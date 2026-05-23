# Edge Description Containers Plan

## Overview

This plan covers inserting `edge_description_container` taffy nodes -- each
holding an `edge_description` leaf -- interleaved between the existing
`rank_container` nodes in the taffy layout tree. When an edge has a description
in `EntityDescs`, the text is rendered in that container at a position
determined by the ranks of the edge's divergent ancestors at the LCA level.

This is a separate rendering location from the face-label slots described in
`edge_descriptions.md`. Note that `edge_descriptions.md` is currently out of
date: face-label slots no longer render text from `EntityDescs`. `EntityDescs`
is now the exclusive source for edge description containers (this feature).
Step 7.5 covers updating that document.

Edge description containers render the description text as a block positioned
along the edge path, between the source and destination rank containers.

Phases at a glance:

| Phase | Summary |
|-------|---------|
| 1 | New data-model types |
| 2 | Taffy node construction (`EdgeDescriptionBuilder`) |
| 3 | Text measurement and highlighted spans |
| 4 | Edge spacers inside edge_description_containers |
| 5 | SVG rendering |
| 6 | Edge path routing through the new spacers |
| 7 | Documentation updates |


## Background: Position Calculation

Given a `divergent_ancestor_node_rank_from` and a
`divergent_ancestor_node_rank_to`, the insertion position P is:

- **Cycle edge** (`rank_from == rank_to`):
  `P = rank_from - 1`, inserting **before** the shared rank container.
  Special case: if `rank_from == 0`, insert before all rank containers (P = -1,
  represented as `None`).

- **Normal edge** (`rank_from != rank_to`):
  `P = rank_low + (rank_high - rank_low) / 2`
  (integer division, flooring to the lower rank), inserting **after**
  the rank container at rank P and **before** the rank container at rank P+1.

  Examples:
  - ranks 2 and 3: P = 2 + (3-2)/2 = 2 -- inserted between rank 2 and rank 3.
  - ranks 0 and 4: P = 0 + (4-0)/2 = 2 -- inserted between rank 2 and rank 3.
  - ranks 0 and 1: P = 0 + (1-0)/2 = 0 -- inserted between rank 0 and rank 1.

The `edge_description_container` sits between rank containers as a sibling of
them in the parent flex node (e.g., `ThingsContainer`). Its `taffy::Style`
mirrors the rank container style: same `display: Flex` and same `flex_direction`
(the user-configured main axis direction).

Multiple `edge_description_container`s at the same position P are inserted
consecutively, each as its own flex container. When multiple described edges
share the same position P they are ordered ascending by **sibling middle index**
`(nesting_path_from[lca_depth] + nesting_path_to[lca_depth]) / 2`, with
`EdgeId` as a tiebreaker. This mirrors the spatial ordering used for same-level
cross-rank spacer insertion (see `edge_spacers.md`).

Like same-level cross-rank edge spacers (see `edge_spacers.md`), the
`EdgeDescriptionBuilder` must be called once per LCA level (once at the root
level and once per container that is an LCA for at least one described edge), so
that containers are inserted into the correct parent rank-container list.


## Phase 1 -- Data Model Types

### Step 1.1 -- `EdgeDescriptionCtx` context type

Source: `crate/taffy_model/src/taffy_node_ctx/edge_description_ctx.rs`

New file. Used as the context on the `edge_description` leaf node so that
`node_size_measure` can look up the description text and compute the leaf's
dimensions.

```rust
pub struct EdgeDescriptionCtx {
    pub edge_id: EdgeId<'static>,
}
```


### Step 1.2 -- Add `EdgeDescription` variant to `TaffyNodeCtx`

Source: `crate/taffy_model/src/taffy_node_ctx.rs`

Add a new variant alongside the existing `EdgeSpacer` and `EdgeLabel` variants:

```rust
pub enum TaffyNodeCtx {
    // ...existing variants...
    EdgeDescription(EdgeDescriptionCtx),
}
```

The `edge_description_container` taffy node continues to use
`TaffyNodeCtx::None` (structural container, like all rank containers).


### Step 1.3 -- `EdgeDescriptionTaffyNodes` type

Source: `crate/taffy_model/src/edge_description_taffy_nodes.rs`

New file. Stores the two taffy node IDs created for a single described edge at
a single LCA level.

```rust
pub struct EdgeDescriptionTaffyNodes {
    /// The flex container interleaved between rank containers.
    pub container_taffy_node_id: taffy::NodeId,
    /// The leaf node inside the container whose size is measured from the
    /// description text.
    pub description_taffy_node_id: taffy::NodeId,
}
```


### Step 1.4 -- Update `TaffyNodeMappings`

Source: wherever `TaffyNodeMappings` is defined (likely
`crate/taffy_model/src/taffy_node_mappings.rs`).

Add a new field:

```rust
pub edge_description_taffy_nodes: Map<EdgeId<'static>, EdgeDescriptionTaffyNodes>,
```


### Step 1.5 -- Update `EdgeSpacerTaffyNodes`

Source: wherever `EdgeSpacerTaffyNodes` is defined (likely
`crate/taffy_model/src/edge_spacer_taffy_nodes.rs`).

Add a third list for spacers placed inside `edge_description_container`s:

```rust
pub struct EdgeSpacerTaffyNodes {
    pub rank_to_spacer_taffy_node_id:               Map<NodeRank, taffy::NodeId>,
    pub cross_container_spacer_taffy_node_ids:       Vec<taffy::NodeId>,
    /// Spacers placed inside edge_description_container nodes.
    pub edge_desc_container_spacer_taffy_node_ids:  Vec<taffy::NodeId>,
}
```


## Phase 2 -- Taffy Node Construction

### Step 2.1 -- `EdgeDescriptionBuilder` module

Source: `crate/input_ir_rt/src/ir_to_taffy_builder/edge_description_builder.rs`

New module, modelled on `EdgeSpacerBuilder`. It exposes a single `build`
function that creates all `edge_description_container` and `edge_description`
nodes for one LCA level and returns them together with their insertion
positions.

Approximate signature:

```rust
impl EdgeDescriptionBuilder {
    pub fn build(
        taffy_tree:          &mut TaffyTree<TaffyNodeCtx>,
        entity_descs:        &EntityDescs,
        edge_groups:         &EdgeGroups,
        node_nesting_infos:  &NodeNestingInfos,
        node_ranks_nested:   &NodeRanksNested,
        entity_types:        &EntityTypes,
        target_entity_type:  &EntityType,
        lca_node_id:         Option<&NodeId>,
        rank_container_style: &taffy::Style,
    ) -> EdgeDescriptionBuildResult
}

pub struct EdgeDescriptionBuildResult {
    /// Maps each edge ID to its newly created taffy nodes.
    pub edge_description_taffy_nodes:
        Map<EdgeId<'static>, EdgeDescriptionTaffyNodes>,

    /// Positions to interleave with rank containers.
    /// Key: `None` means before all rank containers;
    ///       `Some(rank_P)` means after rank_container[rank_P].
    /// Value: ordered list of edge_description_container taffy::NodeIds
    ///        inserted at that position, sorted ascending by sibling middle
    ///        index `(nesting_path_from[lca_depth] + nesting_path_to[lca_depth])
    ///        / 2` (with EdgeId as tiebreaker). Use a
    ///        `BTreeMap<(usize, EdgeId), taffy::NodeId>` internally during
    ///        construction, then flatten to `Vec` before returning.
    pub position_to_container_ids:
        BTreeMap<Option<NodeRank>, Vec<taffy::NodeId>>,
}
```


### Step 2.2 -- Algorithm: `edge_desc_build` (per edge)

For each edge in every edge group, call the private helper
`edge_desc_build`. The helper follows the same LCA-filtering pattern used
by `EdgeSpacerBuilder::edge_spacers_build`:

**Step 2.2.1 -- Filter by entity_descs.**
Look up `entity_descs.get(edge_id)`. Return `None` if the edge has no
description.

**Step 2.2.2 -- Resolve nesting infos.**
Look up `NodeNestingInfo` for `edge.from` and `edge.to`. Return `None` if
either is missing.

**Step 2.2.3 -- Compute LCA depth and divergent ancestors.**

```text
lca_depth      = LcaDepthCalculator::calculate(info_from, info_to)
divergent_from = info_from.ancestor_chain[lca_depth]
divergent_to   = info_to.ancestor_chain[lca_depth]
```

Return `None` if either index is out of bounds (one node is an ancestor of
the other).

**Step 2.2.4 -- Entity type filter.**
Both `divergent_from` and `divergent_to` must match `target_entity_type`.
Return `None` otherwise.

**Step 2.2.5 -- LCA level filter.**
Same rule as `EdgeSpacerBuilder::edge_spacers_build` Step 5:

- `lca_node_id = None` (top-level call): return `None` unless `lca_depth == 0`.
- `lca_node_id = Some(id)` (nested call): return `None` unless `lca_depth > 0`
  and `info_from.ancestor_chain[lca_depth - 1] == id`.

**Step 2.2.6 -- Look up divergent ancestor ranks.**

```text
lca_container  = if lca_depth > 0 { Some(info_from.ancestor_chain[lca_depth - 1]) }
                 else              { None }
rank_from      = node_ranks_nested.ranks_for(lca_container)[divergent_from]
rank_to        = node_ranks_nested.ranks_for(lca_container)[divergent_to]
```

**Step 2.2.7 -- Compute insertion position.**

```text
if rank_from == rank_to {
    // Cycle edge: insert before the shared rank container.
    position = if rank_from > 0 { Some(rank_from - 1) } else { None }
} else {
    // Normal edge: insert after the rank container at the midpoint rank.
    rank_low  = min(rank_from, rank_to)
    rank_high = max(rank_from, rank_to)
    position  = Some(rank_low + (rank_high - rank_low) / 2)
}
```

**Step 2.2.8 -- Create taffy nodes.**

1. Compute the sibling middle index (sort key):
   ```text
   sibling_index_from = nesting_path_from[lca_depth]
   sibling_index_to   = nesting_path_to[lca_depth]
   sibling_middle     = (sibling_index_from + sibling_index_to) / 2
   ```
2. Create an `edge_description` leaf node with `TaffyNodeCtx::EdgeDescription(EdgeDescriptionCtx { edge_id })` and a style appropriate for text measurement (min-size: auto, stretch).
3. Create an `edge_description_container` node (using `rank_container_style`) with the leaf as its sole child. Context: `TaffyNodeCtx::None`.
4. Record `EdgeDescriptionTaffyNodes { container_taffy_node_id, description_taffy_node_id }`.
5. Insert `(sibling_middle, edge_id) -> container_taffy_node_id` into an internal `BTreeMap<(usize, EdgeId), taffy::NodeId>` keyed by `position`. On return, flatten each inner BTreeMap (in key order) into a `Vec<taffy::NodeId>` for `position_to_container_ids`.


### Step 2.3 -- Interleaving rank containers and edge_description_containers

Source: `IrToTaffyBuilder::build_taffy_rank_containers_for_first_level_nodes`
(and the equivalent in `build_taffy_child_nodes_for_node_by_rank` for nested
levels).

After building the rank containers from `rank_to_taffy_ids` and obtaining
`EdgeDescriptionBuildResult`, interleave them in a single pass:

```text
sorted_ranks       = rank_to_taffy_ids.keys() sorted ascending
result_children    = []

// Prepend any containers positioned before all rank containers.
append result_children <- position_to_container_ids[None] (if present)

for rank in sorted_ranks:
    append result_children <- rank_container[rank]
    append result_children <- position_to_container_ids[Some(rank)] (if present)

return result_children
```

The resulting `Vec<taffy::NodeId>` is then set as the children of the parent
container node (e.g., `ThingsContainer` or a diagram container wrapper).


### Step 2.4 -- Wire `EdgeDescriptionBuilder` into `IrToTaffyBuilder`

Source: `crate/input_ir_rt/src/ir_to_taffy_builder.rs`

`EdgeDescriptionBuilder::build` must be called at the same points where
`EdgeSpacerBuilder::build` is called:

1. **Top-level call** inside `build_taffy_trees_for_dimension` (or
   `build_taffy_nodes_for_first_level_nodes`), once per entity type with
   `lca_node_id = None`.

2. **Nested call** inside `build_taffy_nodes_for_node_with_child_hierarchy`,
   once per container node with `lca_node_id = Some(&container_id)`.

Merge the returned `edge_description_taffy_nodes` maps from all calls into
`TaffyNodeMappings::edge_description_taffy_nodes` (same accumulation pattern
as `edge_spacer_taffy_nodes`).


## Phase 3 -- Text Measurement and Highlighted Spans

### Step 3.1 -- `node_size_measure` update

Source: `IrToTaffyBuilder::node_size_measure`

Add a match arm for `TaffyNodeCtx::EdgeDescription(ctx)`:

- Only measure text at `DiagramLod::Normal` (same as face labels).
- Look up `entity_descs.get(ctx.edge_id.as_ref())`.
- If found, compute text width and line count exactly as for face-label nodes,
  returning a `Size<AvailableSpace>` that reflects the wrapped text dimensions.
- If not found (should not occur for well-formed diagrams), return zero size.


### Step 3.2 -- `edge_description_highlighted_spans` computation

Source: `IrToTaffyBuilder::highlighted_spans_compute` (or a new sibling
function `highlighted_spans_compute_edge_desc_containers`).

After taffy layout, for each entry in `edge_description_taffy_nodes`:

1. Look up `entity_descs.get(edge_id.as_ref())`. Skip if absent.
2. Read the taffy layout width of `description_taffy_node_id` as the wrapping
   constraint.
3. Call `wrap_text_monospace` to produce wrapped lines.
4. Build `EntityHighlightedSpan` values (x, y, width, height, text) relative to
   `description_taffy_node_id`'s top-left corner.
5. Store in a new map `edge_description_highlighted_spans: Map<EdgeId, Vec<EntityHighlightedSpan>>` on `TaffyNodeMappings` (separate from the existing `entity_highlighted_spans` used for face labels, to avoid key collisions).


## Phase 4 -- Edge Spacers Inside Edge Description Containers

### Step 4.1 -- Which edges need spacers in an edge_description_container

An `edge_description_container` at position P (after rank container at rank P)
occupies physical space along the rank axis. Any edge whose rank span crosses
from a rank `<= P` to a rank `> P` passes through this space and requires a
spacer inside the container.

Formally, for an edge with `rank_low` and `rank_high` at the LCA level: it
requires a spacer in the container at position P if `rank_low <= P < rank_high`.

Note that the edge **owning** the container (the described edge) does NOT need
a spacer inside its own container -- its path terminates at the description node
itself.


### Step 4.2 -- `EdgeSpacerBuilder::build_edge_desc_container_spacers`

Source: `crate/input_ir_rt/src/ir_to_taffy_builder/edge_spacer_builder.rs`

New function on `EdgeSpacerBuilder`. Called after `EdgeDescriptionBuilder::build`
has produced the `position_to_container_ids` map.

Signature (approximate):

```rust
pub fn build_edge_desc_container_spacers(
    taffy_tree:             &mut TaffyTree<TaffyNodeCtx>,
    edge_groups:            &EdgeGroups,
    node_nesting_infos:     &NodeNestingInfos,
    node_ranks_nested:      &NodeRanksNested,
    entity_types:           &EntityTypes,
    target_entity_type:     &EntityType,
    lca_node_id:            Option<&NodeId>,
    position_to_container_ids: &BTreeMap<Option<NodeRank>, Vec<taffy::NodeId>>,
) -> Map<EdgeId, EdgeSpacerTaffyNodes>
```

Algorithm per edge:

1. Resolve nesting infos, LCA depth, divergent ancestors, and ranks (identical
   to `edge_spacers_build` Steps 1-5).
2. Skip the described edge itself (its container is not a waypoint for its own
   path).
3. For each position P in `position_to_container_ids`:
   - Check: `rank_low <= P < rank_high` (where P is `None` treated as before
     rank 0, i.e., `P_value = -1` conceptually; in practice, only
     `Some(rank_P)` positions with `rank_low <= rank_P.value() < rank_high`
     qualify).
   - Create a spacer leaf node with the standard spacer style and
     `TaffyNodeCtx::EdgeSpacer { edge_id, rank: ??? }`.
     - Since `edge_description_container`s sit between integer ranks, use a
       sentinel or the nearest integer rank. The simplest approach: use the
       rank of the container-at-position-P as the spacer's rank (i.e., the rank
       P value or `rank_low` for the `None` case).
   - Append the spacer as a child of the container at position P
     (`taffy_tree.add_child(container_id, spacer_id)`).
   - Append `spacer_id` to the edge's
     `EdgeSpacerTaffyNodes::edge_desc_container_spacer_taffy_node_ids`.


### Step 4.3 -- Wire into `IrToTaffyBuilder`

Call `EdgeSpacerBuilder::build_edge_desc_container_spacers` immediately after
`EdgeDescriptionBuilder::build` returns, passing the `position_to_container_ids`
from that result. Merge the returned `Map<EdgeId, EdgeSpacerTaffyNodes>` into
the accumulating `edge_spacer_taffy_nodes` map (same merge pattern as the
existing spacer calls).


## Phase 5 -- SVG Rendering

### Step 5.1 -- `SvgEdgeDescriptionInfo` type

Source: wherever SVG info types live (e.g.,
`crate/svg_model/src/svg_edge_description_info.rs`).

```rust
pub struct SvgEdgeDescriptionInfo {
    pub edge_id:    EdgeId<'static>,
    pub x:          f32,
    pub y:          f32,
    pub width:      f32,
    pub height:     f32,
    pub text_spans: Vec<SvgTextSpan>,
}
```


### Step 5.2 -- `SvgEdgeDescriptionsBuilder`

Source: `crate/input_ir_rt/src/taffy_to_svg_elements_mapper/svg_edge_descriptions_builder.rs`

New builder, modelled on `SvgEdgeLabelsBuilder`. Iterates over
`edge_description_taffy_nodes` and `edge_description_highlighted_spans`:

1. For each edge ID, look up `edge_description_highlighted_spans.get(edge_id)`.
   Skip if absent or empty.
2. Compute the absolute SVG position of `description_taffy_node_id` by walking
   up the taffy tree (same `taffy_node_absolute_xy_compute` helper used by
   `SvgEdgeLabelsBuilder`).
3. Offset each span's (x, y) by the node's absolute position to produce
   `SvgTextSpan` values with diagram-level coordinates.
4. Produce one `SvgEdgeDescriptionInfo` per edge.


### Step 5.3 -- `SvgElements` update

Add:

```rust
pub edge_description_infos: Vec<SvgEdgeDescriptionInfo>,
```

Wire `SvgEdgeDescriptionsBuilder::build` into `TaffyToSvgElementsMapper::map`
(same call site as `SvgEdgeLabelsBuilder::build`).


### Step 5.4 -- `SvgElementsToSvgMapper` update

Source: `crate/input_ir_rt/src/svg_elements_to_svg_mapper.rs`

Add `render_edge_descriptions` that iterates `edge_description_infos` and
writes:

```svg
<g id="{edge_id}__desc" class="edge-description">
  <text x="..." y="..." stroke-width="0">line text</text>
  ...
</g>
```

Call `render_edge_descriptions` in the same render pass as
`render_edge_labels`.


## Phase 6 -- Edge Path Routing

### Step 6.1 -- Include `edge_desc_container_spacer_taffy_node_ids` in spacer sorting

Source: `crate/input_ir_rt/src/taffy_to_svg_elements_mapper/svg_edge_infos_builder.rs`
-- `spacer_coordinates_from_spacers` function.

Currently this function collects rank-based and cross-container spacer IDs into
a single list sorted by their main-axis coordinate. Extend it to also include
`edge_desc_container_spacer_taffy_node_ids`:

```text
all_spacer_ids = rank_spacer_ids
              ++ cross_container_spacer_ids
              ++ edge_desc_container_spacer_ids
sort by main-axis absolute coordinate (entry_y for TopToBottom/BottomToTop,
                                       entry_x for LeftToRight/RightToLeft)
```

No other changes to protrusion or path-building logic are needed; once sorted
into the common spacer list, the new spacers are treated identically to
existing cross-container spacers.


### Step 6.2 -- `EdgeSpacerCoordinatesCalculator` -- no change required

`EdgeSpacerCoordinatesCalculator` already computes entry/exit coordinates for
any `taffy::NodeId` by walking up the tree. The new spacer nodes are plain taffy
leaf nodes with the same style as all other spacers, so no changes are needed
there.


## Phase 7 -- Documentation Updates

### Step 7.1 -- Update `taffy_node_hierarchy.md`

Add a new section **Edge Description Containers** describing:

- Where `edge_description_container` nodes sit in the tree (as siblings of rank
  containers within `ThingsContainer`, `ProcessesContainer`, etc., or within a
  diagram container wrapper).
- The style of `edge_description_container` (mirrors rank container style).
- The `edge_description` leaf node inside it and its `TaffyNodeCtx::EdgeDescription`
  context.
- The new `TaffyNodeCtx::EdgeDescription(EdgeDescriptionCtx)` variant in the
  `TaffyNodeCtx` variants table.


### Step 7.2 -- Update `edge_spacers.md`

Add a **third kind of spacer** subsection under **Two Kinds of Spacer**:

- **3. Edge Description Container Spacers** -- built by
  `EdgeSpacerBuilder::build_edge_desc_container_spacers`.
  Used when an edge's rank span crosses the position of an
  `edge_description_container` that belongs to a different described edge.
  A single spacer is inserted as a child of that container, providing a
  coordinate waypoint so the crossing edge's path routes through the container's
  visual space.

Also update:

- **Data Produced** section: note the new
  `edge_desc_container_spacer_taffy_node_ids` field in `EdgeSpacerTaffyNodes`.
- **When Spacer Building is Triggered**: add the new
  `build_edge_desc_container_spacers` call after `build` and
  `build_cross_container_spacers` in each stage.


### Step 7.3 -- Update `diagram_generation.md`

In the step that describes taffy node construction (the `IrToTaffyBuilder`
stage), add:

- A note that `EdgeDescriptionBuilder::build` is called at the same three
  trigger points as `EdgeSpacerBuilder::build` (top-level and per-container),
  inserting `edge_description_container` nodes into the rank-container list.
- A note that `EdgeSpacerBuilder::build_edge_desc_container_spacers` is called
  immediately after, inserting spacers for crossing edges.

In the SVG-rendering stage (`TaffyToSvgElementsMapper`), add a note that
`SvgEdgeDescriptionsBuilder` produces `SvgEdgeDescriptionInfo` values placed
in `SvgElements::edge_description_infos`.


### Step 7.4 -- Update `CLAUDE.md`

Add a reference under **Additional Context**:

```
9. See `<@doc/src/edge_description_containers_plan.md>` for the step-by-step
   plan to render edge descriptions as container nodes interleaved between rank
   containers.
```


### Step 7.5 -- Update `edge_descriptions.md`

`edge_descriptions.md` currently documents that face-label slots render their
text from `EntityDescs`. This is no longer accurate. Update the document to
reflect the current state:

1. **Remove** the `EntityDescs` data-flow sections that show description text
   flowing into face-label taffy nodes and SVG `<text>` elements via
   `node_size_measure` / `highlighted_spans_compute` / `SvgEdgeLabelsBuilder`.

2. **Clarify the current role of face-label slots**: they are purely structural
   taffy leaf nodes used for edge contact-point positioning and face-offset
   calculations (see `edge_paths.md` -- Offset Calculation). They measure as
   zero size when no other sizing applies and carry no rendered text.

3. **Add a cross-reference** to `edge_description_containers_plan.md`
   (this document) as the authoritative source for how `EntityDescs` text is
   rendered in the diagram.

4. Update the **Data Flow Summary** diagram to remove the `EntityDescs` ->
   face-label path and add a note pointing to the edge-description-container
   pipeline.


## Open Questions

### OQ1 -- Spacer rank sentinel for edge_description_container spacers

Spacers inside `edge_description_container`s are not at an integer rank but
between ranks. The `EdgeSpacerCtx` currently stores a `NodeRank`. Options:

- Use the rank of the position P (i.e., `rank_P` from `Some(rank_P)`) as the
  spacer rank. This is a valid integer and unambiguous for sorting.
- Introduce a new context type for these spacers (analogous to how
  `cross_container_spacer_taffy_node_ids` is a separate list).

Either approach works since the spacers are already segregated into the
`edge_desc_container_spacer_taffy_node_ids` list and sorted by coordinate.
Confirm which is cleaner during implementation.

### OQ2 -- Cycle edges at rank 0

For a cycle edge whose divergent ancestors share rank 0, the computed position
is `None` (before all rank containers). If there are no rank containers yet
(e.g., a container with only cycle-rank-0 nodes), the container list is just
the `edge_description_container`s. Confirm this is acceptable visually.
