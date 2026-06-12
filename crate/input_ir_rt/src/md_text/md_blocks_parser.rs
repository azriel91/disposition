use disposition_taffy_model::{MdHeadingLevel, MdStyle};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

/// A single block-level element from the markdown source.
pub(crate) struct MdBlock {
    /// Heading level, or `None` for a paragraph.
    pub(crate) heading_level: Option<MdHeadingLevel>,
    /// Ordered inline tokens within this block.
    pub(crate) tokens: Vec<MdTokenItem>,
    /// `None` for non-list blocks (paragraph / heading); `Some(depth)` for a
    /// list item, where `depth` is the 0-based nesting level (top-level list
    /// items are `Some(0)`, items in a once-nested list are `Some(1)`, etc.).
    ///
    /// Used by `MdNodeBuilder` to indent nested items and to stack list items
    /// tightly (no blank line between siblings).
    pub(crate) list_depth: Option<u8>,
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
enum ListState {
    /// Inside an ordered list, tracking the current item number.
    Ordered { current_number: u64 },
    /// Inside an unordered list.
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
        let mut list_stack: Vec<ListState> = Vec::new();
        let mut list_item_prefix_pending: Option<String> = None;

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    Self::block_flush(&mut current_block, &mut blocks);
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
                        list_depth: Self::list_depth_current(&list_stack),
                    });
                }
                Event::End(TagEnd::Heading(_)) => {
                    Self::block_flush(&mut current_block, &mut blocks);
                }
                Event::Start(Tag::Paragraph) => {
                    Self::block_flush(&mut current_block, &mut blocks);
                    current_block = Some(MdBlock {
                        heading_level: None,
                        tokens: vec![],
                        list_depth: Self::list_depth_current(&list_stack),
                    });
                }
                Event::End(TagEnd::Paragraph) => {
                    Self::block_flush(&mut current_block, &mut blocks);
                }
                Event::Start(Tag::List(first_item_number)) => {
                    // Flush the parent list item's text (if any) before descending
                    // into the nested list, otherwise it would be clobbered when
                    // the first nested item creates its own block.
                    Self::block_flush(&mut current_block, &mut blocks);
                    let list_state = if let Some(start_number) = first_item_number {
                        ListState::Ordered {
                            current_number: start_number,
                        }
                    } else {
                        ListState::Unordered
                    };
                    list_stack.push(list_state);
                }
                Event::End(TagEnd::List(_)) => {
                    list_stack.pop();
                }
                Event::Start(Tag::Item) => {
                    Self::block_flush(&mut current_block, &mut blocks);
                    let list_depth = Self::list_depth_current(&list_stack);
                    // Prepare the list item prefix based on the innermost list.
                    let prefix = match list_stack.last_mut() {
                        Some(ListState::Ordered { current_number }) => {
                            let prefix = format!("{}. ", current_number);
                            *current_number += 1;
                            prefix
                        }
                        Some(ListState::Unordered) => "- ".to_string(),
                        None => String::new(),
                    };
                    list_item_prefix_pending = Some(prefix);
                    current_block = Some(MdBlock {
                        heading_level: None,
                        tokens: vec![],
                        list_depth,
                    });
                }
                Event::End(TagEnd::Item) => {
                    Self::block_flush(&mut current_block, &mut blocks);
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
                        // Prepend heading or list item prefix if pending
                        let code_text = if let Some(prefix) = heading_prefix_pending.take() {
                            format!("{}{}", prefix, text)
                        } else if let Some(prefix) = list_item_prefix_pending.take() {
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
                            // Prepend heading or list item prefix to the first word if pending
                            if let Some(prefix) = heading_prefix_pending
                                .take()
                                .or_else(|| list_item_prefix_pending.take())
                            {
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
                                    // No words in text, keep prefix pending
                                    // Note: we can't distinguish which prefix it was, so we
                                    // store it back in heading_prefix_pending as a fallback
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

        blocks
    }

    /// Pushes `current_block` into `blocks` when it holds at least one token,
    /// then clears it.
    ///
    /// Empty blocks are dropped rather than pushed. This matters for "loose"
    /// markdown lists, where `Start(Tag::Item)` creates a block but the item
    /// text arrives inside a nested `Paragraph`; the empty item block is
    /// flushed (and discarded) when that paragraph starts.
    fn block_flush(current_block: &mut Option<MdBlock>, blocks: &mut Vec<MdBlock>) {
        if let Some(block) = current_block.take()
            && !block.tokens.is_empty()
        {
            blocks.push(block);
        }
    }

    /// Returns the 0-based nesting depth of the innermost active list, or
    /// `None` when not currently inside a list.
    fn list_depth_current(list_stack: &[ListState]) -> Option<u8> {
        list_stack
            .len()
            .checked_sub(1)
            .map(|depth| depth.min(u8::MAX as usize) as u8)
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

    #[test]
    fn nested_unordered_list_keeps_parent_item_and_nesting_depth() {
        let markdown = "\
* unordered item 1
* unordered item 2
    - unordered nested item 2.1
";
        let blocks = MdBlocksParser::parse(markdown);

        let summaries = blocks
            .iter()
            .map(|block| (block_text(block), block.list_depth))
            .collect::<Vec<_>>();

        assert_eq!(
            summaries,
            vec![
                ("- unordered item 1".to_string(), Some(0)),
                // The parent item is no longer clobbered by the nested list.
                ("- unordered item 2".to_string(), Some(0)),
                // The nested item keeps its deeper nesting depth.
                ("- unordered nested item 2.1".to_string(), Some(1)),
            ]
        );
    }

    #[test]
    fn nested_ordered_list_restores_parent_numbering_and_depth() {
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
            .map(|block| (block_text(block), block.list_depth))
            .collect::<Vec<_>>();

        assert_eq!(
            summaries,
            vec![
                ("1. item 1".to_string(), Some(0)),
                ("2. item 2".to_string(), Some(0)),
                ("1. nested ordered item 2.1".to_string(), Some(1)),
                ("2. nested ordered item 2.2".to_string(), Some(1)),
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
        assert_eq!(blocks[0].list_depth, None);
        assert_eq!(block_text(&blocks[1]), "The main branch is protected.");
        assert_eq!(blocks[1].list_depth, None);
    }
}
