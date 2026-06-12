use disposition_taffy_model::{MdHeadingLevel, MdStyle};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

/// A single block-level element from the markdown source.
pub(crate) struct MdBlock {
    /// Heading level, or `None` for a paragraph.
    pub(crate) heading_level: Option<MdHeadingLevel>,
    /// Ordered inline tokens within this block.
    ///
    /// For list items the first token is the (plain-styled) marker, e.g. `"*"`
    /// or `"a."`, inserted by `MdBlocksParser::list_markers_apply` after the
    /// whole document is parsed (so ordered markers can be right-aligned).
    pub(crate) tokens: Vec<MdTokenItem>,
    /// List-item metadata when this block is a list item, otherwise `None`
    /// (paragraphs / headings).
    ///
    /// Used by `MdNodeBuilder` to indent nested items (via
    /// [`MdListItem::depth`]) and to stack list items tightly (no blank
    /// line between siblings).
    pub(crate) list_item: Option<MdListItem>,
}

/// Metadata for a list-item [`MdBlock`].
#[derive(Clone)]
pub(crate) struct MdListItem {
    /// 0-based nesting depth (top-level items are `0`).
    pub(crate) depth: u8,
    /// Identifier of the list this item belongs to. Items that share a
    /// `list_id` are siblings in the same list instance, so their ordered
    /// markers are right-aligned against the widest marker among them.
    list_id: u32,
    /// The marker kind / value for this item.
    marker: MdListMarker,
}

/// The marker of a list item, as entered in the markdown source.
#[derive(Clone)]
enum MdListMarker {
    /// An unordered item, keeping the bullet character used in the source
    /// (`'*'`, `'-'`, or `'+'`).
    Unordered { bullet: char },
    /// An ordered item with its 1-based ordinal within the list. The rendered
    /// form depends on nesting depth (decimal, then lowercase alpha, then
    /// lowercase roman).
    Ordered { number: u64 },
}

/// An inline token within a block.
pub(crate) enum MdTokenItem {
    /// A single word (no interior whitespace) with its active inline style.
    Word {
        /// A single word (no interior whitespace).
        text: String,
        /// Active inline style when this word was emitted.
        md_style: MdStyle,
    },
    /// An inline image.
    Image {
        /// The image URL.
        src: String,
        /// Alt text with any trailing `{WxH}` annotation already stripped.
        alt: String,
        /// Width in pixels from a trailing `{WxH}` annotation in the alt text,
        /// e.g. `![Logo {80x60}](url)` yields `explicit_width: Some(80.0)`.
        explicit_width: Option<f32>,
        /// Height in pixels from the same alt-text annotation.
        explicit_height: Option<f32>,
    },
    /// A line break within a block, typically from a soft break in markdown.
    LineBreak,
}

/// Parses a markdown string into an ordered list of [`MdBlock`] values.
pub(crate) struct MdBlocksParser;

struct ImageState {
    src: String,
    alt_buffer: String,
}

struct StyleStack {
    bold_depth: u32,
    italic_depth: u32,
    strikethrough_depth: u32,
    link_dest: Option<String>,
}

/// State for one level of list nesting. The parser keeps a stack of these so
/// that nested lists restore the parent list's numbering when they end.
struct ListLevel {
    /// Whether this list is ordered (and its running item number) or unordered.
    kind: ListKind,
    /// Unique identifier of this list instance, used to group sibling items
    /// for ordered-marker right-alignment.
    list_id: u32,
}

enum ListKind {
    /// An ordered list tracking the next item number.
    Ordered { next_number: u64 },
    /// An unordered list.
    Unordered,
}

impl MdBlocksParser {
    /// Parses a markdown string into an ordered list of [`MdBlock`] values.
    pub(crate) fn parse(markdown: &str) -> Vec<MdBlock> {
        let options = Options::ENABLE_STRIKETHROUGH;
        let parser = Parser::new_ext(markdown, options);

        let mut blocks: Vec<MdBlock> = Vec::new();
        let mut current_block: Option<MdBlock> = None;
        let mut style_stack = StyleStack {
            bold_depth: 0,
            italic_depth: 0,
            strikethrough_depth: 0,
            link_dest: None,
        };
        let mut image_state: Option<ImageState> = None;
        let mut heading_prefix_pending: Option<String> = None;
        // Stack of active list nesting levels. Empty when not inside a list.
        let mut list_stack: Vec<ListLevel> = Vec::new();
        // Monotonic id assigned to each list instance, for marker alignment.
        let mut next_list_id: u32 = 0;
        // The list-item metadata to attach to the next block that receives text.
        // Set when an item starts; cleared once a non-empty block has claimed it
        // (so loose-list items, whose text arrives in a nested paragraph, still
        // get their marker, while later paragraphs in the same item do not).
        let mut pending_list_item: Option<MdListItem> = None;

        // `into_offset_iter` gives each event's source byte range, used to read
        // the unordered bullet character (`*`, `-`, or `+`) as it was entered.
        for (event, range) in parser.into_offset_iter() {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    Self::block_flush(&mut current_block, &mut blocks, &mut pending_list_item);
                    let heading_level = Self::heading_level_from(level);
                    // Prepare the heading prefix (e.g., "# " for H1, "## " for H2)
                    let prefix_count = match level {
                        pulldown_cmark::HeadingLevel::H1 => 1,
                        pulldown_cmark::HeadingLevel::H2 => 2,
                        pulldown_cmark::HeadingLevel::H3 => 3,
                        pulldown_cmark::HeadingLevel::H4 => 4,
                        pulldown_cmark::HeadingLevel::H5 => 5,
                        pulldown_cmark::HeadingLevel::H6 => 6,
                    };
                    heading_prefix_pending = Some(format!("{} ", "#".repeat(prefix_count)));
                    current_block = Some(MdBlock {
                        heading_level: Some(heading_level),
                        tokens: vec![],
                        list_item: pending_list_item.clone(),
                    });
                }
                Event::End(TagEnd::Heading(_)) => {
                    Self::block_flush(&mut current_block, &mut blocks, &mut pending_list_item);
                }
                Event::Start(Tag::Paragraph) => {
                    Self::block_flush(&mut current_block, &mut blocks, &mut pending_list_item);
                    current_block = Some(MdBlock {
                        heading_level: None,
                        tokens: vec![],
                        list_item: pending_list_item.clone(),
                    });
                }
                Event::End(TagEnd::Paragraph) => {
                    Self::block_flush(&mut current_block, &mut blocks, &mut pending_list_item);
                }
                Event::Start(Tag::List(first_item_number)) => {
                    // Flush the parent list item's text (if any) before descending
                    // into the nested list, otherwise it would be clobbered when
                    // the first nested item creates its own block.
                    Self::block_flush(&mut current_block, &mut blocks, &mut pending_list_item);
                    let kind = if let Some(start_number) = first_item_number {
                        ListKind::Ordered {
                            next_number: start_number,
                        }
                    } else {
                        ListKind::Unordered
                    };
                    let list_id = next_list_id;
                    next_list_id += 1;
                    list_stack.push(ListLevel { kind, list_id });
                }
                Event::End(TagEnd::List(_)) => {
                    list_stack.pop();
                }
                Event::Start(Tag::Item) => {
                    Self::block_flush(&mut current_block, &mut blocks, &mut pending_list_item);
                    let depth = Self::list_depth_current(&list_stack).unwrap_or(0);
                    if let Some(list_level) = list_stack.last_mut() {
                        let list_id = list_level.list_id;
                        let marker = match &mut list_level.kind {
                            ListKind::Ordered { next_number } => {
                                let marker = MdListMarker::Ordered {
                                    number: *next_number,
                                };
                                *next_number += 1;
                                marker
                            }
                            ListKind::Unordered => MdListMarker::Unordered {
                                bullet: Self::bullet_char_at(markdown, range.start),
                            },
                        };
                        pending_list_item = Some(MdListItem {
                            depth,
                            list_id,
                            marker,
                        });
                    }
                    current_block = Some(MdBlock {
                        heading_level: None,
                        tokens: vec![],
                        list_item: pending_list_item.clone(),
                    });
                }
                Event::End(TagEnd::Item) => {
                    Self::block_flush(&mut current_block, &mut blocks, &mut pending_list_item);
                }
                Event::Start(Tag::Strong) => {
                    style_stack.bold_depth += 1;
                }
                Event::End(TagEnd::Strong) => {
                    style_stack.bold_depth -= 1;
                }
                Event::Start(Tag::Emphasis) => {
                    style_stack.italic_depth += 1;
                }
                Event::End(TagEnd::Emphasis) => {
                    style_stack.italic_depth -= 1;
                }
                Event::Start(Tag::Strikethrough) => {
                    style_stack.strikethrough_depth += 1;
                }
                Event::End(TagEnd::Strikethrough) => {
                    style_stack.strikethrough_depth -= 1;
                }
                Event::Start(Tag::Link { dest_url, .. }) => {
                    style_stack.link_dest = Some(String::from(dest_url));
                }
                Event::End(TagEnd::Link) => {
                    style_stack.link_dest = None;
                }
                Event::Code(text) => {
                    let heading_level = current_block
                        .as_ref()
                        .and_then(|current_block| current_block.heading_level);
                    let md_style = MdStyle {
                        code: true,
                        bold: style_stack.bold_depth > 0,
                        italic: style_stack.italic_depth > 0,
                        strikethrough: style_stack.strikethrough_depth > 0,
                        heading_level,
                        link_dest: style_stack.link_dest.clone(),
                    };
                    if let Some(block) = current_block.as_mut() {
                        // Prepend the heading prefix if pending. List markers are
                        // inserted later by `list_markers_apply`.
                        let code_text = if let Some(prefix) = heading_prefix_pending.take() {
                            format!("{}{}", prefix, text)
                        } else {
                            String::from(text)
                        };
                        block.tokens.push(MdTokenItem::Word {
                            text: code_text,
                            md_style,
                        });
                    }
                }
                Event::Text(text) => {
                    if let Some(state) = image_state.as_mut() {
                        state.alt_buffer.push_str(&text);
                    } else {
                        let heading_level = current_block
                            .as_ref()
                            .and_then(|current_block| current_block.heading_level);
                        let md_style = MdStyle {
                            bold: style_stack.bold_depth > 0,
                            italic: style_stack.italic_depth > 0,
                            strikethrough: style_stack.strikethrough_depth > 0,
                            code: false,
                            heading_level,
                            link_dest: style_stack.link_dest.clone(),
                        };
                        if let Some(block) = current_block.as_mut() {
                            let mut words: Vec<&str> = text.split_ascii_whitespace().collect();
                            // Prepend the heading prefix to the first word if pending.
                            // List markers are inserted later by `list_markers_apply`.
                            if let Some(prefix) = heading_prefix_pending.take() {
                                if let Some(first_word) = words.first_mut() {
                                    let prefixed_word = format!("{}{}", prefix, first_word);
                                    block.tokens.push(MdTokenItem::Word {
                                        text: prefixed_word,
                                        md_style: md_style.clone(),
                                    });
                                    // Add remaining words
                                    for word in &words[1..] {
                                        block.tokens.push(MdTokenItem::Word {
                                            text: word.to_string(),
                                            md_style: md_style.clone(),
                                        });
                                    }
                                } else {
                                    // No words in text, keep prefix pending.
                                    heading_prefix_pending = Some(prefix);
                                }
                            } else {
                                // No prefix pending, add words normally
                                for word in words {
                                    block.tokens.push(MdTokenItem::Word {
                                        text: word.to_string(),
                                        md_style: md_style.clone(),
                                    });
                                }
                            }
                        }
                    }
                }
                Event::Start(Tag::Image { dest_url, .. }) => {
                    image_state = Some(ImageState {
                        src: String::from(dest_url),
                        alt_buffer: String::new(),
                    });
                }
                Event::End(TagEnd::Image) => {
                    if let Some(state) = image_state.take() {
                        let (alt, explicit_width, explicit_height) =
                            Self::parse_alt_annotation(&state.alt_buffer);
                        if let Some(block) = current_block.as_mut() {
                            block.tokens.push(MdTokenItem::Image {
                                src: state.src,
                                alt,
                                explicit_width,
                                explicit_height,
                            });
                        }
                    }
                }
                Event::HardBreak => {
                    if let Some(block) = current_block.as_mut() {
                        block.tokens.push(MdTokenItem::LineBreak);
                    }
                }
                _ => {}
            }
        }

        Self::list_markers_apply(&mut blocks);

        blocks
    }

    /// Pushes `current_block` into `blocks` when it holds at least one token,
    /// then clears it.
    ///
    /// Empty blocks are dropped rather than pushed. This matters for "loose"
    /// markdown lists, where `Start(Tag::Item)` creates a block but the item
    /// text arrives inside a nested `Paragraph`; the empty item block is
    /// flushed (and discarded) when that paragraph starts.
    ///
    /// When a non-empty list-item block is pushed, `pending_list_item` is
    /// cleared so that any further paragraphs in the same item are not given a
    /// duplicate marker.
    fn block_flush(
        current_block: &mut Option<MdBlock>,
        blocks: &mut Vec<MdBlock>,
        pending_list_item: &mut Option<MdListItem>,
    ) {
        if let Some(block) = current_block.take()
            && !block.tokens.is_empty()
        {
            if block.list_item.is_some() {
                *pending_list_item = None;
            }
            blocks.push(block);
        }
    }

    /// Returns the 0-based nesting depth of the innermost active list, or
    /// `None` when not currently inside a list.
    fn list_depth_current(list_stack: &[ListLevel]) -> Option<u8> {
        list_stack
            .len()
            .checked_sub(1)
            .map(|depth| depth.min(u8::MAX as usize) as u8)
    }

    /// Returns the unordered-list bullet character (`'*'`, `'-'`, or `'+'`) at
    /// the given source byte offset, defaulting to `'-'` if none is found.
    fn bullet_char_at(markdown: &str, item_start: usize) -> char {
        markdown
            .get(item_start..)
            .and_then(|rest| rest.chars().find(|c| !c.is_whitespace()))
            .filter(|c| matches!(c, '*' | '-' | '+'))
            .unwrap_or('-')
    }

    /// Inserts a plain-styled marker token at the front of each list-item
    /// block.
    ///
    /// Markers are computed after the whole document is parsed so that ordered
    /// markers can be right-aligned: within each list (grouped by `list_id`),
    /// the widest marker's character count is used to left-pad the others with
    /// spaces, so that the trailing `.` and the following text line up.
    ///
    /// Marker style by nesting depth (cycling every three levels): decimal
    /// (`1.`), then lowercase alpha (`a.`), then lowercase roman (`i.`).
    /// Unordered items keep the bullet entered in the source (`*`, `-`, `+`).
    fn list_markers_apply(blocks: &mut [MdBlock]) {
        use std::collections::BTreeMap;

        // First pass: the rendered marker "core" (without padding or trailing
        // `.`) per block, and the widest core per list.
        let marker_cores: Vec<Option<String>> = blocks
            .iter()
            .map(|block| {
                block
                    .list_item
                    .as_ref()
                    .map(|list_item| match &list_item.marker {
                        MdListMarker::Ordered { number } => {
                            Self::ordered_marker_format(*number, list_item.depth)
                        }
                        MdListMarker::Unordered { bullet } => bullet.to_string(),
                    })
            })
            .collect();

        let mut list_core_width_max: BTreeMap<u32, usize> = BTreeMap::new();
        for (block, marker_core) in blocks.iter().zip(marker_cores.iter()) {
            if let (Some(list_item), Some(marker_core)) = (block.list_item.as_ref(), marker_core) {
                let width = list_core_width_max.entry(list_item.list_id).or_insert(0);
                *width = (*width).max(marker_core.chars().count());
            }
        }

        // Second pass: build and prepend the marker token.
        for (block, marker_core) in blocks.iter_mut().zip(marker_cores) {
            let (Some(list_item), Some(marker_core)) = (block.list_item.as_ref(), marker_core)
            else {
                continue;
            };

            let core_width_max = list_core_width_max
                .get(&list_item.list_id)
                .copied()
                .unwrap_or_else(|| marker_core.chars().count());
            let pad = " ".repeat(core_width_max.saturating_sub(marker_core.chars().count()));

            let marker_text = match &list_item.marker {
                MdListMarker::Ordered { .. } => format!("{pad}{marker_core}."),
                MdListMarker::Unordered { .. } => format!("{pad}{marker_core}"),
            };

            block.tokens.insert(
                0,
                MdTokenItem::Word {
                    text: marker_text,
                    // Plain style so the marker is never struck through / bold /
                    // italic with the item text, and so it forms its own span.
                    md_style: MdStyle::default(),
                },
            );
        }
    }

    /// Formats an ordered-list item `number` for the given nesting `depth`:
    /// decimal at depth 0, lowercase alpha at depth 1, lowercase roman at depth
    /// 2, cycling every three levels. Returns just the value (no trailing `.`).
    fn ordered_marker_format(number: u64, depth: u8) -> String {
        match depth % 3 {
            0 => number.to_string(),
            1 => Self::alpha_lower(number),
            _ => Self::roman_lower(number),
        }
    }

    /// Converts a 1-based `number` to a lowercase bijective base-26 string
    /// (`1 -> "a"`, `26 -> "z"`, `27 -> "aa"`).
    fn alpha_lower(mut number: u64) -> String {
        if number == 0 {
            return String::from("0");
        }
        let mut chars = Vec::new();
        while number > 0 {
            number -= 1;
            chars.push((b'a' + (number % 26) as u8) as char);
            number /= 26;
        }
        chars.iter().rev().collect()
    }

    /// Converts a 1-based `number` to a lowercase roman numeral
    /// (`1 -> "i"`, `4 -> "iv"`, `9 -> "ix"`).
    fn roman_lower(mut number: u64) -> String {
        if number == 0 {
            return String::from("0");
        }
        const ROMAN_VALUES: [(u64, &str); 13] = [
            (1000, "m"),
            (900, "cm"),
            (500, "d"),
            (400, "cd"),
            (100, "c"),
            (90, "xc"),
            (50, "l"),
            (40, "xl"),
            (10, "x"),
            (9, "ix"),
            (5, "v"),
            (4, "iv"),
            (1, "i"),
        ];
        let mut roman = String::new();
        for (value, symbol) in ROMAN_VALUES {
            while number >= value {
                roman.push_str(symbol);
                number -= value;
            }
        }
        roman
    }

    fn heading_level_from(level: pulldown_cmark::HeadingLevel) -> MdHeadingLevel {
        match level {
            pulldown_cmark::HeadingLevel::H1 => MdHeadingLevel::H1,
            pulldown_cmark::HeadingLevel::H2 => MdHeadingLevel::H2,
            pulldown_cmark::HeadingLevel::H3 => MdHeadingLevel::H3,
            pulldown_cmark::HeadingLevel::H4 => MdHeadingLevel::H4,
            pulldown_cmark::HeadingLevel::H5 => MdHeadingLevel::H5,
            pulldown_cmark::HeadingLevel::H6 => MdHeadingLevel::H6,
        }
    }

    /// Strips a trailing `{WxH}` annotation from alt text.
    ///
    /// Returns `(clean_alt, explicit_width, explicit_height)`. The annotation
    /// is case-insensitive on the `x` separator, e.g. `{80x60}` or `{80X60}`.
    ///
    /// # Examples
    ///
    /// - `"Logo {80x60}"` yields `("Logo", Some(80.0), Some(60.0))`
    /// - `"Logo"` yields `("Logo", None, None)`
    fn parse_alt_annotation(alt: &str) -> (String, Option<f32>, Option<f32>) {
        let trimmed = alt.trim_end();
        if let Some(brace_start) = trimmed.rfind('{') {
            let annotation = &trimmed[brace_start..];
            if annotation.ends_with('}') {
                let brace_content = &annotation[1..annotation.len() - 1];
                let lower = brace_content.to_ascii_lowercase();
                if let Some(x_pos) = lower.find('x') {
                    let w_str = &brace_content[..x_pos];
                    let h_str = &brace_content[x_pos + 1..];
                    if let (Ok(w), Ok(h)) =
                        (w_str.trim().parse::<f32>(), h_str.trim().parse::<f32>())
                    {
                        let clean_alt = alt[..brace_start].trim_end().to_string();
                        return (clean_alt, Some(w), Some(h));
                    }
                }
            }
        }
        (alt.to_string(), None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::{MdBlock, MdBlocksParser, MdTokenItem};
    use disposition_taffy_model::MdStyle;

    /// Joins a block's `Word` tokens with single spaces (ignoring images and
    /// line breaks) so list-item text can be compared in tests.
    fn block_text(md_block: &MdBlock) -> String {
        md_block
            .tokens
            .iter()
            .filter_map(|token| match token {
                MdTokenItem::Word { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Returns the nesting depth of a block, or `None` for non-list blocks.
    fn block_depth(md_block: &MdBlock) -> Option<u8> {
        md_block.list_item.as_ref().map(|list_item| list_item.depth)
    }

    #[test]
    fn nested_unordered_list_keeps_parent_item_bullet_and_depth() {
        // The top level uses `*`; the nested level uses `-`. Each marker is kept
        // as entered, and it is the first token of the item block.
        let markdown = "\
* unordered item 1
* unordered item 2
    - unordered nested item 2.1
";
        let blocks = MdBlocksParser::parse(markdown);

        let summaries = blocks
            .iter()
            .map(|block| (block_text(block), block_depth(block)))
            .collect::<Vec<_>>();

        assert_eq!(
            summaries,
            vec![
                ("* unordered item 1".to_string(), Some(0)),
                // The parent item is no longer clobbered by the nested list.
                ("* unordered item 2".to_string(), Some(0)),
                // The nested item keeps its deeper depth and its `-` bullet.
                ("- unordered nested item 2.1".to_string(), Some(1)),
            ]
        );
    }

    #[test]
    fn marker_token_is_plain_styled_so_it_is_not_struck_through() {
        let markdown = "* ~~struck~~ item\n";
        let blocks = MdBlocksParser::parse(markdown);

        let MdTokenItem::Word { text, md_style } = &blocks[0].tokens[0] else {
            panic!("expected first token to be the marker word");
        };
        assert_eq!(text, "*");
        // The marker carries no inline styling even though the text is struck.
        assert_eq!(md_style, &MdStyle::default());

        // The following word is the struck-through text.
        let MdTokenItem::Word {
            text: struck_text,
            md_style: struck_style,
        } = &blocks[0].tokens[1]
        else {
            panic!("expected struck text token");
        };
        assert_eq!(struck_text, "struck");
        assert!(struck_style.strikethrough);
    }

    #[test]
    fn nested_ordered_list_uses_decimal_then_alpha_by_depth() {
        // The blank line before the nested list makes this a "loose" list, so
        // item text arrives inside nested paragraphs.
        let markdown = "\
1. item 1
2. item 2

    1. nested ordered item 2.1
    2. nested ordered item 2.2
";
        let blocks = MdBlocksParser::parse(markdown);

        let summaries = blocks
            .iter()
            .map(|block| (block_text(block), block_depth(block)))
            .collect::<Vec<_>>();

        assert_eq!(
            summaries,
            vec![
                ("1. item 1".to_string(), Some(0)),
                ("2. item 2".to_string(), Some(0)),
                // Depth 1 ordered items render as lowercase alpha.
                ("a. nested ordered item 2.1".to_string(), Some(1)),
                ("b. nested ordered item 2.2".to_string(), Some(1)),
            ]
        );
    }

    #[test]
    fn deeply_nested_ordered_list_uses_roman_right_aligned() {
        // Depth 2 ordered items render as lowercase roman numerals, right
        // aligned: the widest is `viii` (4 chars), so shorter ones are padded
        // with leading spaces so the trailing `.` lines up.
        let markdown = "\
1. one

    1. a

        1. r1
        2. r2
        3. r3
        4. r4
        5. r5
        6. r6
        7. r7
        8. r8
";
        let blocks = MdBlocksParser::parse(markdown);

        let roman_markers = blocks
            .iter()
            .filter(|block| block_depth(*block) == Some(2))
            .map(|block| match &block.tokens[0] {
                MdTokenItem::Word { text, .. } => text.clone(),
                _ => panic!("expected marker word"),
            })
            .collect::<Vec<_>>();

        assert_eq!(
            roman_markers,
            vec![
                "   i.".to_string(),
                "  ii.".to_string(),
                " iii.".to_string(),
                "  iv.".to_string(),
                "   v.".to_string(),
                "  vi.".to_string(),
                " vii.".to_string(),
                "viii.".to_string(),
            ]
        );
    }

    #[test]
    fn paragraphs_and_headings_have_no_list_depth() {
        let markdown = "\
### Source

The main branch is protected.
";
        let blocks = MdBlocksParser::parse(markdown);

        assert_eq!(blocks.len(), 2);
        assert_eq!(block_text(&blocks[0]), "### Source");
        assert_eq!(block_depth(&blocks[0]), None);
        assert_eq!(block_text(&blocks[1]), "The main branch is protected.");
        assert_eq!(block_depth(&blocks[1]), None);
    }
}
