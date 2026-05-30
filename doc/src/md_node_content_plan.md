# Markdown Node Content Plan

## Overview

This plan covers replacing the simple monospace-measured single `text_node`
in each diagram node with a markdown-aware layout that:

1. Renders markdown text with syntax highlighting (bold, italic, headings,
   inline code, strikethrough, and links).
2. Renders inline images inside nodes.
3. Calculates text positions using per-token taffy nodes, so that wrapping
   respects actual token widths including image dimensions.

The current pipeline collapses the full description string (e.g.
`"# Foo\n\nSome **bold** text"`) into a single measured leaf taffy node, then
post-layout re-wraps it into `EntityHighlightedSpan` entries that each carry
plain text. This plan replaces that leaf with a flex-column container of
per-token leaves, enabling correct mixed-style line wrapping and inline images.

Phases at a glance:

| Phase | Summary |
|-------|---------|
| 1 | New data-model types in `taffy_model` and `svg_model` |
| 2 | Markdown parsing utilities (`MdBlocksParser`, `MdImageSizer`) |
| 3 | Taffy node construction (`MdNodeBuilder`) |
| 4 | Text measurement updates (`node_size_measure`) |
| 5 | Post-layout span computation (`MdSpansComputer`) |
| 6 | SVG rendering (styled text + inline images) |
| 7 | Documentation updates |
| 8 | Extend markdown rendering to edge descriptions |


## Background

### Taffy Node Structure Change

The existing `text_node_id` in every `NodeToTaffyNodeIds` variant is currently
a taffy **leaf** node measured by `node_size_measure`. This plan replaces the
leaf with an `md_content_node` -- a flex-column container -- whose children are
`block_row_node` containers (one per block-level markdown element), each
containing individual token leaf nodes:

```yaml
md_content_node:          # flex-column, replaces old text_node_id leaf
  block_row_0:            # flex-row, flex_wrap: Wrap  (H1 heading block)
    word_leaf_0:          # leaf, TaffyNodeCtx::MdToken("Hello", H1 style)
    word_leaf_1:          # leaf, TaffyNodeCtx::MdToken("World", H1 style)
  block_row_1:            # flex-row, flex_wrap: Wrap  (paragraph block)
    word_leaf_2:          # leaf, TaffyNodeCtx::MdToken("Some", plain)
    word_leaf_3:          # leaf, TaffyNodeCtx::MdToken("bold", Bold style)
    image_leaf_0:         # leaf, TaffyNodeCtx::MdImage(width=80, height=60)
    word_leaf_4:          # leaf, TaffyNodeCtx::MdToken("text", plain)
```

`NodeToTaffyNodeIds` is kept unchanged -- the `text_node_id` field now points
to the `md_content_node_id` container instead of a leaf. Existing code that
reads the layout of `text_node_id` (e.g. for bounding-box calculations) still
works because containers expose the same `Layout` API as leaves.

The old `TaffyNodeCtx::DiagramNode` lookup on `text_node_id` will return
`None` for markdown nodes (a container carries no context), so
`HighlightedSpansComputer::compute` gracefully skips them. Span computation
for markdown nodes is handled exclusively by the new `MdSpansComputer`.


### Scope: `DiagramLod::Normal` with Description Only

The markdown path is only activated when:
- LOD is `DiagramLod::Normal`, **and**
- the node has an entry in `thing_descs`.

At `DiagramLod::Simple`, or at `Normal` with no description, the existing
single-leaf path is kept unchanged.


### Image Sizing Priority

For each `Tag::Image` event from pulldown-cmark:

1. Alt-text annotation `{WxH}` at the end of the alt text (e.g.
   `![Logo {80x60}](data:image/png;base64,...)`) -- highest priority. The
   annotation is stripped from the displayed alt text.
2. Intrinsic size decoded from a base64 PNG data URL
   (`data:image/png;base64,<data>`). Uses the `base64` workspace crate to
   decode and reads the PNG IHDR chunk (bytes 16-23) for width and height.
3. Proportional scaling: if one dimension is known (from step 1 or 2) and the
   other is not, scale the missing dimension to maintain the aspect ratio.
4. Fallback: `100.0 x 100.0`.


### Post-Layout Span Merging

After taffy layout, each token leaf has a computed absolute position. The
merging algorithm groups consecutive token leaves into single
`EntityHighlightedSpan` entries using the following criterion:

- The tokens are from the **same block row** (same `MdBlockTaffyIds`).
- The tokens land on the **same visual line**, defined as sharing the same
  `floor(absolute_y)` value.
- The tokens have **identical `MdStyle`** (bold, italic, heading level, etc.).

Tokens that meet all three criteria are concatenated (space-separated) into one
span, carrying their shared style. Tokens with different styles on the same
line become separate spans.

Image leaves always produce a separate `MdImageSpan` entry and are never merged
with text spans.


### Heading Font Scaling

Heading blocks are given a larger font size by scaling `TEXT_FONT_SIZE`:

| Level | Scale | Effective size (base 14 px) |
|-------|-------|-----------------------------|
| H1    | 2.0x  | 28 px                       |
| H2    | 1.5x  | 21 px                       |
| H3    | 1.25x | 17.5 px                     |
| H4    | 1.0x  | 14 px                       |
| H5    | 1.0x  | 14 px                       |
| H6    | 1.0x  | 14 px                       |

The scaled font size affects both the taffy measurement (leaf width and height)
and the SVG `font-size` attribute.


## Phase 1 -- Data Model Types

### Step 1.1 -- `MdHeadingLevel` enum

Source: `crate/taffy_model/src/md_heading_level.rs`

New file. A self-contained heading level type that does not depend on
`pulldown-cmark`. Used in `MdStyle`, `MdTokenCtx`, and SVG rendering.

```rust
pub enum MdHeadingLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl MdHeadingLevel {
    /// Returns the font-size scale factor for this heading level.
    pub fn font_scale(self) -> f32 {
        match self {
            MdHeadingLevel::H1 => 2.0,
            MdHeadingLevel::H2 => 1.5,
            MdHeadingLevel::H3 => 1.25,
            MdHeadingLevel::H4 | MdHeadingLevel::H5 | MdHeadingLevel::H6 => 1.0,
        }
    }
}
```


### Step 1.2 -- `MdStyle` struct

Source: `crate/taffy_model/src/md_style.rs`

New file. Records all inline formatting active at the moment a token is emitted
by the markdown parser. Derives `Default` (all fields `false` / `None`).

```rust
pub struct MdStyle {
    /// Whether the token is inside a `**strong**` / `__strong__` run.
    pub bold: bool,
    /// Whether the token is inside an `*emphasis*` / `_emphasis_` run.
    pub italic: bool,
    /// Whether the token is inside a `~~strikethrough~~` run.
    pub strikethrough: bool,
    /// Whether the token is an inline code fragment (`` `code` ``).
    pub code: bool,
    /// Non-`None` when the token is inside a heading block.
    pub heading_level: Option<MdHeadingLevel>,
    /// Non-`None` when the token is inside a `[link](url)` run.
    /// Contains the destination URL string, e.g. `"https://example.com"`.
    pub link_dest: Option<String>,
}
```

Implement `PartialEq`, `Eq`, `Hash`, `Clone`, `Debug`, `Default`,
`Deserialize`, and `Serialize`.


### Step 1.3 -- `MdTokenCtx` struct

Source: `crate/taffy_model/src/md_token_ctx.rs`

New file. Context placed on a word-token taffy leaf so that `node_size_measure`
can compute its width using the token text and heading-level font scale.

```rust
pub struct MdTokenCtx {
    /// The word or text fragment to measure. Contains no leading/trailing
    /// whitespace.
    ///
    /// Example: `"bold"`, `"hello"`.
    pub text: String,
    /// Inline markdown style active when this token was emitted.
    pub md_style: MdStyle,
}
```


### Step 1.4 -- `MdImageCtx` struct

Source: `crate/taffy_model/src/md_image_ctx.rs`

New file. Context placed on an image leaf node. The node is given a fixed size
during construction so taffy does not call the measure function for it; the
context is retained for SVG rendering.

```rust
pub struct MdImageCtx {
    /// Data URL or relative path of the image.
    ///
    /// Example: `"data:image/png;base64,iVBORw0K..."`, `"diagram.png"`.
    pub src: String,
    /// Alt text for the image.
    pub alt: String,
    /// Rendered width in pixels (already resolved from priority rules).
    pub width: f32,
    /// Rendered height in pixels (already resolved from priority rules).
    pub height: f32,
}
```


### Step 1.5 -- Add `MdToken` and `MdImage` variants to `TaffyNodeCtx`

Source: `crate/taffy_model/src/taffy_node_ctx.rs`

Add two new variants alongside the existing ones:

```rust
pub enum TaffyNodeCtx {
    // ...existing variants...
    MdToken(MdTokenCtx),
    MdImage(MdImageCtx),
}
```

The `md_content_node` and `block_row_*` containers carry `TaffyNodeCtx::None`
(same convention as rank containers and other structural nodes).


### Step 1.6 -- `MdBlockTaffyIds` struct

Source: `crate/taffy_model/src/md_block_taffy_ids.rs`

New file. Stores the taffy node IDs for one block-level markdown element and
the ordered list of token or image leaf node IDs within it.

```rust
pub struct MdBlockTaffyIds {
    /// The flex-row-wrap container node for this block.
    pub block_row_node_id: taffy::NodeId,
    /// Ordered leaf node IDs for each token or image in this block.
    ///
    /// Each ID corresponds to either a `TaffyNodeCtx::MdToken` leaf or a
    /// `TaffyNodeCtx::MdImage` leaf.
    pub token_node_ids: Vec<taffy::NodeId>,
}
```


### Step 1.7 -- `MdNodeTaffyIds` struct

Source: `crate/taffy_model/src/md_node_taffy_ids.rs`

New file. Stores the complete taffy node ID set for a diagram node's markdown
content area.

```rust
pub struct MdNodeTaffyIds {
    /// The flex-column container holding all block rows.
    ///
    /// This is the node stored as `text_node_id` in `NodeToTaffyNodeIds`.
    pub content_node_id: taffy::NodeId,
    /// One entry per block-level element, in source order.
    pub block_taffy_ids: Vec<MdBlockTaffyIds>,
}
```


### Step 1.8 -- `MdImageSpan` struct

Source: `crate/taffy_model/src/md_image_span.rs`

New file. An inline image positioned in the diagram's coordinate space.
Defined in `taffy_model` (not `svg_model`) so it can be stored in
`TaffyNodeMappings` without creating a dependency on `svg_model`.

```rust
pub struct MdImageSpan {
    /// Absolute x coordinate of the image's top-left corner.
    pub x: f32,
    /// Absolute y coordinate of the image's top-left corner.
    pub y: f32,
    /// Rendered width in pixels.
    pub width: f32,
    /// Rendered height in pixels.
    pub height: f32,
    /// Image source (data URL or path).
    pub src: String,
    /// Alt text.
    pub alt: String,
}
```


### Step 1.9 -- Update `EntityHighlightedSpan`

Source: `crate/taffy_model/src/entity_highlighted_span.rs`

Activate the previously commented-out `style` field, renamed to `md_style`:

```rust
pub struct EntityHighlightedSpan {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub text: String,
    /// Markdown style for this span. `None` for plain/unstyled text spans
    /// produced by the legacy path.
    pub md_style: Option<MdStyle>,
}
```

Default construction still sets `md_style: None` so existing callers in
`HighlightedSpansComputer` compile without change.


### Step 1.10 -- Update `TaffyNodeMappings`

Source: `crate/taffy_model/src/taffy_node_mappings.rs`

Add two new fields:

```rust
pub struct TaffyNodeMappings<'id> {
    // ...existing fields...

    /// Per-token taffy node IDs for diagram nodes that use the markdown
    /// content path (`DiagramLod::Normal` with a description).
    ///
    /// Keyed by diagram `NodeId`. Absent for nodes that use the legacy
    /// single-leaf text path.
    pub md_node_taffy_ids: Map<NodeId<'id>, MdNodeTaffyIds>,

    /// Inline image spans computed after taffy layout for markdown nodes.
    ///
    /// Keyed by diagram `NodeId`. Absent for nodes without inline images.
    pub entity_image_spans: Map<NodeId<'id>, Vec<MdImageSpan>>,
}
```


## Phase 2 -- SVG Model Updates

### Step 2.1 -- `SvgMdStyle` struct

Source: `crate/svg_model/src/svg_md_style.rs`

New file. The SVG-layer representation of `MdStyle`. Mirrors the fields of
`MdStyle` but lives in `svg_model` rather than `taffy_model`, maintaining the
existing crate separation.

```rust
pub struct SvgMdStyle {
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub code: bool,
    /// `1`--`6`, or `0` for non-heading text.
    pub heading_level: u8,
    /// Destination URL when the span is part of a link. `None` otherwise.
    pub link_dest: Option<String>,
}
```


### Step 2.2 -- Update `SvgTextSpan`

Source: `crate/svg_model/src/svg_text_span.rs`

Add an optional `md_style` field:

```rust
pub struct SvgTextSpan {
    pub x: f32,
    pub y: f32,
    /// Height of the span in pixels (equals `effective_line_height` for the
    /// heading scale). Used to size the code-background `<rect>` (see
    /// Step 6.2).
    pub height: f32,
    /// The text content (already XML-escaped).
    pub text: String,
    /// Markdown style for this span. `None` for plain/unstyled text.
    pub md_style: Option<SvgMdStyle>,
}
```

Keep `SvgTextSpan::new` unchanged (pass `0.0` for `height` and `None` for
`md_style`). Add a `SvgTextSpan::new_styled` constructor that also accepts
`height: f32` and `md_style: Option<SvgMdStyle>`.


### Step 2.3 -- `SvgImageSpan` struct

Source: `crate/svg_model/src/svg_image_span.rs`

New file. Information for an inline `<image>` SVG element.

```rust
pub struct SvgImageSpan {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    /// Image source (data URL or path, unescaped).
    pub src: String,
    /// Alt text.
    pub alt: String,
}
```


### Step 2.4 -- Update `SvgNodeInfo`

Source: `crate/svg_model/src/svg_node_info.rs`

Add `image_spans` alongside `text_spans`:

```rust
pub struct SvgNodeInfo<'id> {
    // ...existing fields...
    pub text_spans: Vec<SvgTextSpan>,
    /// Inline image spans computed from markdown content.
    pub image_spans: Vec<SvgImageSpan>,
}
```

Initialise `image_spans` to `Vec::new()` in all construction sites. Also add
`image_spans: Vec<SvgImageSpan>` to `SvgEdgeDescriptionInfo` in the same way,
for edge descriptions that may contain inline images.


## Phase 3 -- Markdown Parsing

### Step 3.1 -- Add `pulldown-cmark` to the workspace

Source: `Cargo.toml` (workspace)

```toml
pulldown-cmark = "0.13"
```

Source: `crate/input_ir_rt/Cargo.toml`

```toml
pulldown-cmark = { workspace = true }
```


### Step 3.2 -- `md_text` module

Source: `crate/input_ir_rt/src/md_text.rs`
Source: `crate/input_ir_rt/src/md_text/`

New module file + subdirectory. Declares the submodules:

```rust
pub(crate) mod md_blocks_parser;
pub(crate) mod md_image_sizer;
```


### Step 3.3 -- `MdBlock` and `MdTokenItem` types

Source: `crate/input_ir_rt/src/md_text/md_blocks_parser.rs`

These are internal types used only during parsing and taffy node construction.
They do not need to be exported from the crate.

```rust
/// One block-level markdown element (heading or paragraph).
pub(crate) struct MdBlock {
    /// Heading level, or `None` for a paragraph.
    pub(crate) heading_level: Option<MdHeadingLevel>,
    /// Ordered inline tokens within this block.
    pub(crate) tokens: Vec<MdTokenItem>,
}

/// One wrappable inline unit inside a block.
pub(crate) enum MdTokenItem {
    Word {
        /// A single word (no interior whitespace).
        text: String,
        /// Active inline style when this word was emitted.
        md_style: MdStyle,
    },
    Image {
        src: String,
        /// Alt text with any trailing `{WxH}` annotation already stripped.
        alt: String,
        /// Width in pixels from a trailing `{WxH}` annotation in the alt text,
        /// e.g. `![Logo {80x60}](url)` yields `explicit_width: Some(80.0)`.
        explicit_width: Option<f32>,
        /// Height in pixels from the same alt-text annotation.
        explicit_height: Option<f32>,
    },
}
```


### Step 3.4 -- `MdBlocksParser::parse`

Source: `crate/input_ir_rt/src/md_text/md_blocks_parser.rs`

```rust
pub(crate) struct MdBlocksParser;

impl MdBlocksParser {
    /// Parses a markdown string into an ordered list of `MdBlock` values.
    pub(crate) fn parse(markdown: &str) -> Vec<MdBlock> {
        // ...
    }
}
```

The function uses `pulldown_cmark::Parser` with `Options::ENABLE_STRIKETHROUGH`
enabled. It maintains a mutable `StyleStack` that tracks the currently active
inline tags (strong, emphasis, strikethrough, link). For each event:

- `Event::Start(Tag::Heading { level, .. })` -- push current heading level.
- `Event::End(TagEnd::Heading(_))` -- pop heading level, finish block.
- `Event::Start(Tag::Paragraph)` -- begin a new `MdBlock` with `heading_level: None`.
- `Event::End(TagEnd::Paragraph)` -- finish block.
- `Event::Start(Tag::Strong)` -- push `bold: true` onto style stack.
- `Event::End(TagEnd::Strong)` -- pop bold.
- `Event::Start(Tag::Emphasis)` -- push `italic: true`.
- `Event::End(TagEnd::Emphasis)` -- pop italic.
- `Event::Start(Tag::Strikethrough)` -- push `strikethrough: true`.
- `Event::Start(Tag::Link { dest_url, .. })` -- push `link_dest: Some(dest_url)`.
- `Event::Code(text)` -- emit a `MdTokenItem::Word { text, md_style: { code: true, ..rest } }`.
- `Event::Text(text)` -- split `text` on ASCII whitespace; emit one `MdTokenItem::Word` per non-empty fragment, each carrying the current style stack snapshot.
- `Event::Start(Tag::Image { dest_url, .. })` -- record `dest_url` and begin
  collecting alt text from any nested `Event::Text` events.
- `Event::End(TagEnd::Image)` -- parse any trailing `{WxH}` annotation from
  the collected alt text (e.g. `"Logo {80x60}"` -> `alt: "Logo"`,
  `explicit_width: Some(80.0)`, `explicit_height: Some(60.0)`), then emit
  `MdTokenItem::Image` with the stripped alt and the resolved dimensions.
- All other events -- ignored.

#### Style stack snapshot

At each `Event::Text` the active style is captured as:

```rust
MdStyle {
    bold: stack.bold_depth > 0,
    italic: stack.italic_depth > 0,
    strikethrough: stack.strikethrough_depth > 0,
    code: false,  // set by Event::Code branch
    heading_level: current_block_heading_level,
    link_dest: stack.link_dest.clone(),
}
```


### Step 3.5 -- `MdImageSizer::compute_size`

Source: `crate/input_ir_rt/src/md_text/md_image_sizer.rs`

```rust
pub(crate) struct MdImageSizer;

impl MdImageSizer {
    /// Returns `(width, height)` in pixels for the given image token item,
    /// using the priority order described in the Background section.
    pub(crate) fn compute_size(item: &MdTokenItem) -> (f32, f32) {
        // ...
    }

    /// Attempts to read the intrinsic pixel dimensions from a base64 PNG data
    /// URL by decoding the IHDR chunk.
    ///
    /// Returns `None` if the URL is not a PNG data URL or decoding fails.
    fn png_intrinsic_size(src: &str) -> Option<(f32, f32)> {
        // ...
    }
}
```

`png_intrinsic_size` extracts width and height from the PNG IHDR chunk:
- Strip the `data:image/png;base64,` prefix.
- Base64-decode using `BASE64_STANDARD` (already imported in `input_ir_rt`).
- Read bytes 16-19 as big-endian `u32` for width, bytes 20-23 for height.


## Phase 4 -- Taffy Node Construction

### Step 4.1 -- `MdNodeBuilder` module

Source: `crate/input_ir_rt/src/ir_to_taffy_builder/md_node_builder.rs`

New module. Builds the `md_content_node` taffy sub-tree from a parsed
`Vec<MdBlock>`.

```rust
pub(crate) struct MdNodeBuilder;

impl MdNodeBuilder {
    /// Builds a flex-column `md_content_node` containing one flex-row-wrap
    /// `block_row_node` per `MdBlock`, each holding word and image leaf nodes.
    ///
    /// Returns the `MdNodeTaffyIds` describing the full sub-tree.
    pub(crate) fn build(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        blocks: &[MdBlock],
        char_width: f32,
    ) -> MdNodeTaffyIds {
        // ...
    }
}
```

#### Block row style

Each `block_row_node` uses:
- `display: Flex`
- `flex_direction: Row`
- `flex_wrap: Wrap`
- `gap: Size { width: LengthPercentage::Length(char_width), height: LengthPercentage::ZERO }`

The horizontal gap approximates the width of one space character. No explicit
padding or border on block rows.

#### Word leaf style

Each word leaf node uses:
- `display: Block` (or `Flex` with no children)
- Size set to `max_content` (measured by `node_size_measure`)
- `TaffyNodeCtx::MdToken(MdTokenCtx { text, md_style })`

#### Image leaf style

Each image leaf node uses:
- `display: Block`
- Fixed size: `Size { width: Points(width), height: Points(height) }` from `MdImageSizer::compute_size`
- `TaffyNodeCtx::MdImage(MdImageCtx { src, alt, width, height })`

#### `md_content_node` style

```
display: Flex
flex_direction: Column
flex_wrap: NoWrap
align_items: FlexStart
```

No border, padding, or margin (those are already on the surrounding
`wrapper_node` or `text_node` padding inherited from node layout).


### Step 4.2 -- Update `node_size_measure`

Source: `crate/input_ir_rt/src/ir_to_taffy_builder/taffy_node_build_context.rs`
or `ir_to_taffy_builder.rs` -- whichever file contains `node_size_measure`.

Add a new match arm for `TaffyNodeCtx::MdToken`:

```rust
Some(TaffyNodeCtx::MdToken(ctx)) => {
    let font_scale = ctx.md_style
        .heading_level
        .map(MdHeadingLevel::font_scale)
        .unwrap_or(1.0);
    let effective_char_width = char_width * font_scale;
    let effective_line_height = TEXT_LINE_HEIGHT * font_scale;
    let width = line_width_measure(&ctx.text, effective_char_width);
    Size {
        width: length(width),
        height: length(effective_line_height),
    }
}
```

`TaffyNodeCtx::MdImage` leaves have a fixed size set at construction time;
the measure function is not called for them.


### Step 4.3 -- Wire `MdNodeBuilder` into `TaffyDiagramNodeBuilder`

Source: `crate/input_ir_rt/src/ir_to_taffy_builder/taffy_diagram_node_builder.rs`

In the function that creates the `text_node_id` leaf for a diagram node,
add a branch:

```
if lod == DiagramLod::Normal AND node has description:
    let markdown = format!("{node_name}\n\n{desc}");
    let blocks = MdBlocksParser::parse(&markdown);
    let md_node_taffy_ids = MdNodeBuilder::build(taffy_tree, &blocks, char_width);
    let text_node_id = md_node_taffy_ids.content_node_id;
    // store md_node_taffy_ids in a local accumulator to later populate
    // TaffyNodeMappings::md_node_taffy_ids
else:
    // existing single-leaf creation path (unchanged)
```

The returned `text_node_id` is used in `NodeToTaffyNodeIds` as before.
No change to the `NodeToTaffyNodeIds` variants is needed.

After all nodes are built, collect the accumulated `MdNodeTaffyIds` entries
into `TaffyNodeMappings::md_node_taffy_ids`.


## Phase 5 -- Post-Layout Span Computation

### Step 5.1 -- `MdSpansComputer` module

Source: `crate/input_ir_rt/src/ir_to_taffy_builder/md_spans_computer.rs`

New module. Computes `EntityHighlightedSpan` and `MdImageSpan` entries for
nodes that used the markdown content path.

```rust
pub(crate) struct MdSpansComputer;

impl MdSpansComputer {
    /// Computes highlighted text spans and image spans for all nodes that have
    /// `MdNodeTaffyIds` entries in `md_node_taffy_ids`.
    pub(crate) fn compute(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        md_node_taffy_ids: &Map<NodeId<'static>, MdNodeTaffyIds>,
        char_width: f32,
    ) -> (
        EntityHighlightedSpans<'static>,
        Map<NodeId<'static>, Vec<MdImageSpan>>,
    ) {
        // ...
    }
}
```

#### Per-node algorithm

For each `(node_id, md_node_taffy_ids)` in the input map:

1. Compute the absolute top-left `(base_x, base_y)` of `content_node_id` by
   calling `SvgNodeInfoBuilder::node_absolute_xy_coordinates` (already
   available in scope via the existing helper).

2. For each `MdBlockTaffyIds` in `block_taffy_ids`:

   a. Initialise a `pending: Vec<(f32, f32, &TaffyNodeCtx)>` (x, y, context).

   b. For each `taffy_node_id` in `token_node_ids`:
      - Get `layout = taffy_tree.layout(taffy_node_id)`.
      - Compute `abs_x = base_x + layout.location.x`,
        `abs_y = base_y + layout.location.y`.
      - Push `(abs_x, abs_y, context)` onto `pending`.

   c. Group `pending` entries by `floor(abs_y)` (same visual line).
      Within each group, sort by `abs_x`.

   d. Within each visual-line group, merge consecutive `MdToken` entries that
      share the same `MdStyle` into one `EntityHighlightedSpan`:
      - `x` = smallest `abs_x` in the run.
      - `y` = `abs_y + effective_line_height` (one line-height below the top,
        consistent with the existing text baseline convention).
      - `width` = sum of individual token widths.
      - `height` = `effective_line_height` for this style's heading scale.
      - `text` = words joined by `" "`.
      - `md_style` = the shared `MdStyle`.

   e. For each `MdImage` entry in the visual-line group, emit a `MdImageSpan`
      with absolute `(x, y)`, the stored `width` and `height`, `src`, and `alt`.

3. Collect all `EntityHighlightedSpan` entries for this node into a `Vec` and
   insert into `EntityHighlightedSpans` under `node_id`.

4. If any `MdImageSpan` entries were produced for this node, insert them into
   the image-spans map under `node_id`.


### Step 5.2 -- Wire `MdSpansComputer` into `IrToTaffyBuilder`

Source: `crate/input_ir_rt/src/ir_to_taffy_builder.rs`

After the existing `HighlightedSpansComputer::compute` call, call
`MdSpansComputer::compute` and merge the results into `entity_highlighted_spans`
and the new `entity_image_spans`:

```rust
let (md_entity_spans, entity_image_spans) = MdSpansComputer::compute(
    &taffy_tree,
    &md_node_taffy_ids,
    char_width,
);

// Merge md spans into the main spans map (they are keyed by node_id and
// do not overlap with the legacy spans, because markdown nodes have no
// TaffyNodeCtx::DiagramNode on their text_node_id and are therefore skipped
// by HighlightedSpansComputer::compute).
for (node_id, spans) in md_entity_spans {
    entity_highlighted_spans.insert(node_id, spans);
}
```


## Phase 6 -- SVG Rendering

### Step 6.1 -- Update `SvgNodeInfoBuilder`

Source: `crate/input_ir_rt/src/taffy_to_svg_elements_mapper/svg_node_info_builder.rs`

When converting `EntityHighlightedSpan` to `SvgTextSpan`, map `md_style`:

```rust
let text_spans: Vec<SvgTextSpan> = entity_highlighted_spans
    .get(node_id.as_ref())
    .map(|spans| {
        spans
            .iter()
            .map(|span| {
                SvgTextSpan::new_styled(
                    span.x,
                    span.y,
                    StringXmlEscaper::escape(&span.text),
                    span.md_style.as_ref().map(svg_md_style_from),
                )
            })
            .collect()
    })
    .unwrap_or_default();
```

Also populate `image_spans` from `entity_image_spans`:

```rust
let image_spans: Vec<SvgImageSpan> = entity_image_spans
    .get(node_id.as_ref())
    .map(|spans| {
        spans
            .iter()
            .map(|s| SvgImageSpan {
                x: s.x,
                y: s.y,
                width: s.width,
                height: s.height,
                src: s.src.clone(),
                alt: s.alt.clone(),
            })
            .collect()
    })
    .unwrap_or_default();
```

Add a `svg_md_style_from(md_style: &MdStyle) -> SvgMdStyle` helper in the same
file:

```rust
fn svg_md_style_from(md_style: &MdStyle) -> SvgMdStyle {
    SvgMdStyle {
        bold: md_style.bold,
        italic: md_style.italic,
        strikethrough: md_style.strikethrough,
        code: md_style.code,
        heading_level: md_style.heading_level.map(|h| h as u8 + 1).unwrap_or(0),
        link_dest: md_style.link_dest.clone(),
    }
}
```

Update `SvgNodeInfoBuildContext` to pass `entity_image_spans` from
`TaffyNodeMappings`.


### Step 6.2 -- Update `SvgElementsToSvgMapper`

Source: `crate/input_ir_rt/src/svg_elements_to_svg_mapper.rs`

**Styled text rendering** (`render_nodes`):

For each `SvgTextSpan`, emit additional SVG presentation attributes when
`md_style` is `Some`:

```rust
svg_node_info.text_spans.iter().for_each(|span| {
    let text_x = span.x;
    let text_y = span.y;
    let text_content = &span.text;

    let style_attrs = span.md_style.as_ref().map(|s| {
        let font_size = if s.heading_level > 0 {
            let scale = match s.heading_level {
                1 => 2.0f32, 2 => 1.5, 3 => 1.25, _ => 1.0,
            };
            format!(" font-size=\"{}\"", TEXT_FONT_SIZE * scale)
        } else {
            String::new()
        };
        let font_weight = if s.bold { " font-weight=\"bold\"" } else { "" };
        let font_style  = if s.italic { " font-style=\"italic\"" } else { "" };
        let text_deco   = if s.strikethrough {
            " text-decoration=\"line-through\""
        } else if s.link_dest.is_some() {
            " text-decoration=\"underline\""
        } else {
            ""
        };
        format!("{font_size}{font_weight}{font_style}{text_deco}")
    }).unwrap_or_default();

    // Emit a background rect before code spans.
    if span.md_style.as_ref().is_some_and(|s| s.code) {
        let rect_y = text_y - span.height;
        let rect_w = span.width;
        let rect_h = span.height;
        write!(
            content_buffer,
            "<rect x=\"{text_x}\" y=\"{rect_y}\" width=\"{rect_w}\" \
                height=\"{rect_h}\" class=\"md-code-bg\" />",
        ).unwrap();
    }

    write!(
        content_buffer,
        "<text x=\"{text_x}\" y=\"{text_y}\" stroke-width=\"0\"{style_attrs}>\
            {text_content}</text>",
    ).unwrap();
});
```

**Image rendering** (new `render_node_images` function):

```rust
fn render_node_images(content_buffer: &mut String, svg_node_info: &SvgNodeInfo) {
    svg_node_info.image_spans.iter().for_each(|span| {
        let x = span.x;
        let y = span.y;
        let w = span.width;
        let h = span.height;
        let src = &span.src;
        let alt = StringXmlEscaper::escape(&span.alt);
        write!(
            content_buffer,
            "<image x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" \
                href=\"{src}\" alt=\"{alt}\" />",
        ).unwrap();
    });
}
```

Call `render_node_images` immediately after the `text_spans` loop inside each
`<g>` node group in `render_nodes`.

**CSS update** (`map_svg` -- inline `<style>` block):

Add a rule for the code background rect:

```css
.md-code-bg {
    fill: var(--md-code-bg, #e8e8e8);
    rx: 2;
}
```

`--md-code-bg` is a CSS custom property that themes can override. The default
`#e8e8e8` is a light grey that works on both light and dark backgrounds when
used with the existing node fill. `rx: 2` gives the rect slightly rounded
corners to match typical code-span styling.


## Phase 7 -- Documentation Updates

### Step 7.1 -- Update `taffy_node_hierarchy.md`

Add a new **Markdown Content Nodes** section after "Leaf Diagram Nodes (Rect Shape)" describing the `md_content_node` sub-tree layout. Update the **Text Measurement** section to note that `MdToken` leaves are measured individually by `node_size_measure` and that `MdImage` leaves have fixed sizes.

### Step 7.2 -- Update `diagram_generation.md`

In step 3 (taffy layout), mention that when `DiagramLod::Normal` and a
description is present, `MdBlocksParser` + `MdNodeBuilder` replace the single
`text_node` with a per-token sub-tree. In the post-layout paragraph, add
`MdSpansComputer::compute` alongside `HighlightedSpansComputer::compute`.

### Step 7.3 -- Update `CLAUDE.md`

Add a reference to `md_node_content_plan.md` under **Additional Context**:

```
10. See `<@doc/src/md_node_content_plan.md>` for the step-by-step plan to render
    node and edge description text as syntax-highlighted markdown with inline images.
```


## Phase 8 -- Edge Description Markdown Rendering

This phase extends the markdown content path to `edge_description` leaf nodes
so that `EdgeDescs` text is also rendered with syntax highlighting and inline
images, using the same `MdBlocksParser`, `MdNodeBuilder`, and `MdSpansComputer`
built in Phases 2-6.


### Step 8.1 -- Update `EdgeDescriptionTaffyNodes`

Source: `crate/taffy_model/src/edge_description_taffy_nodes.rs`

Add an optional `MdNodeTaffyIds` field. When `None` the legacy single-leaf
path is active; when `Some` the markdown path is active.

```rust
pub struct EdgeDescriptionTaffyNodes {
    /// The flex container interleaved between rank containers.
    pub container_taffy_node_id: taffy::NodeId,
    /// The leaf node (legacy) or `md_content_node` (markdown path) whose
    /// layout position is used to place the description in the SVG.
    pub description_taffy_node_id: taffy::NodeId,
    /// Populated at `DiagramLod::Normal`. When `Some`, `description_taffy_node_id`
    /// points to the `md_content_node` container rather than a bare leaf.
    pub md_node_taffy_ids: Option<MdNodeTaffyIds>,
}
```


### Step 8.2 -- Update `EdgeDescriptionBuilder::build`

Source: `crate/input_ir_rt/src/ir_to_taffy_builder/edge_description_builder.rs`

At `DiagramLod::Normal`, replace the single `edge_description` leaf creation
with a call to `MdNodeBuilder::build`:

```
if lod == DiagramLod::Normal:
    let markdown = edge_descs.get(edge_id);
    let blocks = MdBlocksParser::parse(markdown);
    let md_node_taffy_ids = MdNodeBuilder::build(taffy_tree, &blocks, char_width);
    let description_taffy_node_id = md_node_taffy_ids.content_node_id;
    // Add description_taffy_node_id as child of container_taffy_node_id.
    EdgeDescriptionTaffyNodes {
        container_taffy_node_id,
        description_taffy_node_id,
        md_node_taffy_ids: Some(md_node_taffy_ids),
    }
else:
    // existing single-leaf creation path (unchanged)
    EdgeDescriptionTaffyNodes {
        container_taffy_node_id,
        description_taffy_node_id,
        md_node_taffy_ids: None,
    }
```


### Step 8.3 -- Update `HighlightedSpansComputer::compute_edge_desc_containers`

Source: `crate/input_ir_rt/src/ir_to_taffy_builder/highlighted_spans_computer.rs`

Skip edges whose `EdgeDescriptionTaffyNodes::md_node_taffy_ids` is `Some`;
those are handled by `MdSpansComputer::compute_edge_descs` in the next step.

```rust
edge_description_taffy_nodes
    .iter()
    .filter(|(_, nodes)| nodes.md_node_taffy_ids.is_none())
    .filter_map(|(edge_id, edge_desc_taffy_nodes)| {
        // ...existing logic unchanged...
    })
    .collect()
```


### Step 8.4 -- Add `MdSpansComputer::compute_edge_descs`

Source: `crate/input_ir_rt/src/ir_to_taffy_builder/md_spans_computer.rs`

New method on `MdSpansComputer`. Mirrors `MdSpansComputer::compute` but
operates over `edge_description_taffy_nodes` and keys results by `EdgeId`
rather than `NodeId`.

```rust
impl MdSpansComputer {
    /// Computes highlighted text spans and image spans for all edge
    /// descriptions that used the markdown content path.
    pub(crate) fn compute_edge_descs(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        edge_description_taffy_nodes: &Map<EdgeId<'static>, EdgeDescriptionTaffyNodes>,
        char_width: f32,
    ) -> (
        Map<EdgeId<'static>, Vec<EntityHighlightedSpan>>,
        Map<EdgeId<'static>, Vec<MdImageSpan>>,
    ) {
        // ...same per-block algorithm as compute(), keyed by EdgeId...
    }
}
```

The per-block merging algorithm is identical to the one in `compute()` --
group tokens by visual line, merge consecutive tokens with the same `MdStyle`.


### Step 8.5 -- Update `TaffyNodeMappings`

Source: `crate/taffy_model/src/taffy_node_mappings.rs`

Add a field for edge description image spans:

```rust
pub struct TaffyNodeMappings<'id> {
    // ...existing fields...
    /// Inline image spans for edge descriptions that used the markdown path.
    pub edge_description_image_spans: Map<EdgeId<'id>, Vec<MdImageSpan>>,
}
```


### Step 8.6 -- Wire `MdSpansComputer::compute_edge_descs` into `IrToTaffyBuilder`

Source: `crate/input_ir_rt/src/ir_to_taffy_builder.rs`

After `HighlightedSpansComputer::compute_edge_desc_containers`, call
`MdSpansComputer::compute_edge_descs` and merge results:

```rust
let (md_edge_desc_spans, edge_description_image_spans) =
    MdSpansComputer::compute_edge_descs(
        &taffy_tree,
        &edge_description_taffy_nodes,
        char_width,
    );

// Merge into edge_description_highlighted_spans (disjoint key sets --
// HighlightedSpansComputer skips markdown edges, MdSpansComputer handles them).
for (edge_id, spans) in md_edge_desc_spans {
    edge_description_highlighted_spans.insert(edge_id, spans);
}
```


### Step 8.7 -- Update `SvgEdgeDescriptionInfo`

Source: `crate/svg_model/src/svg_edge_description_info.rs`

Add `image_spans`:

```rust
pub struct SvgEdgeDescriptionInfo<'id> {
    // ...existing fields...
    pub text_spans: Vec<SvgTextSpan>,
    /// Inline image spans for edge descriptions with markdown images.
    pub image_spans: Vec<SvgImageSpan>,
}
```

Initialise `image_spans` to `Vec::new()` at all construction sites.


### Step 8.8 -- Update `SvgEdgeDescriptionsBuilder::build`

Source: `crate/input_ir_rt/src/taffy_to_svg_elements_mapper/svg_edge_descriptions_builder.rs`

Pass `edge_description_image_spans` from `TaffyNodeMappings` into `build` and
apply the same `md_style` mapping as `SvgNodeInfoBuilder`:

```rust
pub(super) fn build<'id>(
    taffy_tree: &TaffyTree<TaffyNodeCtx>,
    edge_description_taffy_nodes: &Map<EdgeId<'id>, EdgeDescriptionTaffyNodes>,
    edge_description_highlighted_spans: &Map<EdgeId<'id>, Vec<EntityHighlightedSpan>>,
    edge_description_image_spans: &Map<EdgeId<'id>, Vec<MdImageSpan>>,
) -> Vec<SvgEdgeDescriptionInfo<'id>>
```

Convert `EntityHighlightedSpan` -> `SvgTextSpan` using `SvgTextSpan::new_styled`
(with `md_style` mapped via `svg_md_style_from`). Populate `image_spans` from
`edge_description_image_spans` with the same `x + span.x` absolute offset
applied.


### Step 8.9 -- Update `SvgElementsToSvgMapper::render_edge_descriptions`

Source: `crate/input_ir_rt/src/svg_elements_to_svg_mapper.rs`

Apply the same styled-text, code-background-rect, and image rendering as in
`render_nodes` (Step 6.2). Extract the shared rendering logic into a helper
function (e.g. `render_text_and_images`) called from both `render_nodes` and
`render_edge_descriptions` to avoid duplication:

```rust
fn render_text_and_images(
    content_buffer: &mut String,
    text_spans: &[SvgTextSpan],
    image_spans: &[SvgImageSpan],
) {
    // emit code-bg rects + <text> elements
    // emit <image> elements
}
```


### Step 8.10 -- Documentation updates

**`diagram_generation.md`** -- In step 3 (taffy layout), add that
`EdgeDescriptionBuilder` also calls `MdNodeBuilder` at `DiagramLod::Normal`.
In the post-layout paragraph, add `MdSpansComputer::compute_edge_descs`
alongside `HighlightedSpansComputer::compute_edge_desc_containers`.

**`edge_descriptions.md`** -- Update the "Text Measurement" section to note
that at `DiagramLod::Normal` the `edge_description` leaf is replaced by an
`md_content_node` sub-tree, and that spans are produced by
`MdSpansComputer::compute_edge_descs` rather than
`HighlightedSpansComputer::compute_edge_desc_containers`.
