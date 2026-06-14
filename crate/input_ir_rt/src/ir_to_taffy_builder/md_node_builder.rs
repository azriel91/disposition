use disposition_taffy_model::{
    MdBlockTaffyIds, MdImageCtx, MdNodeTaffyIds, MdTokenCtx, TaffyNodeCtx, TEXT_LINE_HEIGHT,
};
use taffy::{
    self, AlignItems, Display, FlexDirection, FlexWrap, LengthPercentage, LengthPercentageAuto,
    Rect, Size, Style, TaffyTree,
};

use crate::{
    ir_to_taffy_builder::text_measure::md_token_width_measure,
    md_text::{
        md_blocks_parser::{MdBlock, MdTokenItem},
        md_image_sizer::MdImageSizer,
    },
};

pub(crate) struct MdNodeBuilder;

/// Padding around the whole markdown content node.
const MD_CONTENT_NODE_PADDING: f32 = 3.0;

impl MdNodeBuilder {
    /// Builds a flex-column `md_content_node` containing one flex-column
    /// `block_col_node` per `MdBlock`, each holding one flex-row-wrap
    /// `line_row_node` per logical line (split at `LineBreak` tokens).
    ///
    /// `LineBreak` tokens are NOT inserted as taffy leaf nodes. Instead they
    /// act as boundaries that start a new `line_row_node`. This ensures that
    /// each row's max-content width reflects only its own tokens, so the
    /// container is sized to the widest line rather than the sum of all lines.
    ///
    /// Returns the `MdNodeTaffyIds` describing the full sub-tree.
    pub(crate) fn build(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        md_blocks: &[MdBlock],
        char_width: f32,
    ) -> MdNodeTaffyIds {
        let mut md_block_taffy_ids_list = Vec::with_capacity(md_blocks.len());

        let mut prev_block: Option<&MdBlock> = None;
        for md_block in md_blocks {
            // Vertical spacing is applied per-block as a top margin (see
            // `block_margin_top`) rather than as a uniform container gap, so
            // that consecutive list items stack tightly.
            let margin_top = Self::block_margin_top(prev_block, md_block);
            let margin_left = Self::block_margin_left(md_block, char_width);
            let (block_col_node_id, token_node_ids) =
                Self::build_block(taffy_tree, md_block, char_width, margin_top, margin_left);

            md_block_taffy_ids_list.push(MdBlockTaffyIds {
                block_col_node_id,
                token_node_ids,
            });
            prev_block = Some(md_block);
        }

        let content_node_style = Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            flex_wrap: FlexWrap::NoWrap,
            align_items: Some(AlignItems::FlexStart),
            // No container gap: inter-block spacing is the top margin on each
            // `block_col_node`.
            gap: Size {
                width: LengthPercentage::length(0.0),
                height: LengthPercentage::length(0.0),
            },
            padding: Rect {
                left: LengthPercentage::length(MD_CONTENT_NODE_PADDING),
                right: LengthPercentage::length(MD_CONTENT_NODE_PADDING),
                top: LengthPercentage::length(MD_CONTENT_NODE_PADDING),
                bottom: LengthPercentage::length(MD_CONTENT_NODE_PADDING),
            },
            ..Default::default()
        };
        let block_col_node_ids: Vec<taffy::NodeId> = md_block_taffy_ids_list
            .iter()
            .map(|md_block_taffy_ids| md_block_taffy_ids.block_col_node_id)
            .collect();
        let content_node_id = taffy_tree
            .new_with_children(content_node_style, &block_col_node_ids)
            .expect("Expected to create md_content_node");

        MdNodeTaffyIds {
            content_node_id,
            block_taffy_ids: md_block_taffy_ids_list,
        }
    }

    /// Returns the top margin (blank-line spacing) to place above `md_block`.
    ///
    /// Only consecutive list items at the *same* nesting depth stack tightly
    /// (no blank line). Every other adjacency gets a single `TEXT_LINE_HEIGHT`
    /// blank line: between non-list blocks, between a list and a non-list
    /// block, when entering a deeper nesting level, and when leaving one (a
    /// dedent to a shallower item always gets exactly one blank line, never
    /// one per level popped).
    fn block_margin_top(prev_block: Option<&MdBlock>, md_block: &MdBlock) -> f32 {
        let Some(prev_block) = prev_block else {
            return 0.0;
        };
        match (
            Self::block_list_depth(prev_block),
            Self::block_list_depth(md_block),
        ) {
            (Some(prev_depth), Some(curr_depth)) if prev_depth == curr_depth => 0.0,
            _ => TEXT_LINE_HEIGHT,
        }
    }

    /// Returns the left margin (indentation) for `md_block`, indenting nested
    /// list items by 4 character widths per nesting level.
    fn block_margin_left(md_block: &MdBlock, char_width: f32) -> f32 {
        Self::block_list_depth(md_block).map_or(0.0, |list_depth| {
            (char_width * 4.0).round() * f32::from(list_depth)
        })
    }

    /// Returns the list-nesting depth of `md_block`, or `None` for non-list
    /// blocks (paragraphs / headings).
    fn block_list_depth(md_block: &MdBlock) -> Option<u8> {
        md_block.list_item.as_ref().map(|list_item| list_item.depth)
    }

    /// Builds the `block_col_node` for one `MdBlock`.
    ///
    /// Tokens are split at `LineBreak` boundaries into groups. Each group
    /// becomes a `line_row_node` (flex row wrap). All `line_row_nodes` are
    /// children of the returned `block_col_node` (flex column, no gap).
    ///
    /// `margin_top` provides inter-block spacing and `margin_left` provides
    /// list-item indentation.
    ///
    /// Returns `(block_col_node_id, all_token_leaf_node_ids)`.
    fn build_block(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        md_block: &MdBlock,
        char_width: f32,
        margin_top: f32,
        margin_left: f32,
    ) -> (taffy::NodeId, Vec<taffy::NodeId>) {
        // Partition tokens into line groups at every LineBreak boundary.
        let mut line_groups: Vec<Vec<&MdTokenItem>> = vec![Vec::new()];
        for token in &md_block.tokens {
            if matches!(token, MdTokenItem::LineBreak) {
                line_groups.push(Vec::new());
            } else {
                line_groups
                    .last_mut()
                    .expect("line_groups is never empty")
                    .push(token);
            }
        }

        let mut all_token_node_ids: Vec<taffy::NodeId> = Vec::with_capacity(md_block.tokens.len());
        let mut line_row_node_ids: Vec<taffy::NodeId> = Vec::with_capacity(line_groups.len());

        for line_tokens in line_groups {
            let (line_row_node_id, token_node_ids) =
                Self::build_block_line_row(taffy_tree, &line_tokens, char_width);
            all_token_node_ids.extend_from_slice(&token_node_ids);
            line_row_node_ids.push(line_row_node_id);
        }

        let block_col_style = Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            flex_wrap: FlexWrap::NoWrap,
            align_items: Some(AlignItems::FlexStart),
            margin: Rect {
                left: LengthPercentageAuto::length(margin_left),
                right: LengthPercentageAuto::length(0.0),
                top: LengthPercentageAuto::length(margin_top),
                bottom: LengthPercentageAuto::length(0.0),
            },
            ..Default::default()
        };
        let block_col_node_id = taffy_tree
            .new_with_children(block_col_style, &line_row_node_ids)
            .expect("Expected to create block_col_node");

        (block_col_node_id, all_token_node_ids)
    }

    /// Builds one `line_row_node` (flex-row-wrap) from a slice of
    /// non-`LineBreak` tokens.
    ///
    /// Tokens are grouped into "words": a run of glued tokens (`glue_prev`,
    /// i.e. no whitespace between them in the source) forms one word. Each
    /// word is a child of the `line_row_node`, separated from the next word
    /// by the `char_width` flex gap. A single-token word is added as the
    /// leaf itself; a multi-token word is wrapped in a `gap: 0` container
    /// so its tokens abut with no gap (e.g. the code span and `,` in ``
    /// `git clone`, ``).
    ///
    /// Returns `(line_row_node_id, token_leaf_node_ids)`. The returned leaf ids
    /// are every token leaf in source order (flattened across word containers),
    /// as `MdSpansComputer` reads token positions from them.
    fn build_block_line_row(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        line_tokens: &[&MdTokenItem],
        char_width: f32,
    ) -> (taffy::NodeId, Vec<taffy::NodeId>) {
        let mut token_node_ids: Vec<taffy::NodeId> = Vec::with_capacity(line_tokens.len());
        // Children of the line row: single token leaves or word containers.
        let mut line_row_child_ids: Vec<taffy::NodeId> = Vec::new();
        // Token leaves accumulated for the current word group.
        let mut word_leaf_ids: Vec<taffy::NodeId> = Vec::new();

        for (idx, md_token_item) in line_tokens.iter().enumerate() {
            let token_node_id = Self::token_leaf_build(taffy_tree, md_token_item, char_width);
            token_node_ids.push(token_node_id);

            // The first token of the line always starts a new word; otherwise a
            // glued token continues the current word and a non-glued token
            // starts a new one.
            let starts_new_word = idx == 0 || !md_token_item.glue_prev();
            if starts_new_word {
                Self::word_group_flush(taffy_tree, &mut word_leaf_ids, &mut line_row_child_ids);
            }
            word_leaf_ids.push(token_node_id);
        }
        Self::word_group_flush(taffy_tree, &mut word_leaf_ids, &mut line_row_child_ids);

        let line_row_style = Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            // TODO: This should be correct, but when it is set, text items around an image are:
            // * pushed to the next line
            // * bunched together instead of to the left and right of the image.
            //
            // align_items: Some(AlignItems::End),
            gap: Size {
                // Round to the nearest integer pixel so that gap * n_gaps is always an exact
                // integer. This makes every term in taffy's flex-wrap line-length comparison an
                // exact f32 integer, preventing floating-point non-associativity from pushing
                // the last item onto the next line.
                width: LengthPercentage::length(char_width.round()),
                height: LengthPercentage::length(0.0),
            },
            ..Default::default()
        };
        let line_row_node_id = taffy_tree
            .new_with_children(line_row_style, &line_row_child_ids)
            .expect("Expected to create line_row_node");

        (line_row_node_id, token_node_ids)
    }

    /// Flushes the accumulated `word_leaf_ids` into `line_row_child_ids`.
    ///
    /// A single leaf is added directly as a `line_row` child (the common case,
    /// keeping the tree flat). Multiple glued leaves are wrapped in a flex-row
    /// `gap: 0` container so they abut with no inter-token gap. Clears
    /// `word_leaf_ids`.
    fn word_group_flush(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        word_leaf_ids: &mut Vec<taffy::NodeId>,
        line_row_child_ids: &mut Vec<taffy::NodeId>,
    ) {
        match word_leaf_ids.len() {
            0 => {}
            1 => line_row_child_ids.push(word_leaf_ids[0]),
            _ => {
                let word_group_style = Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::NoWrap,
                    gap: Size {
                        width: LengthPercentage::length(0.0),
                        height: LengthPercentage::length(0.0),
                    },
                    ..Default::default()
                };
                let word_group_node_id = taffy_tree
                    .new_with_children(word_group_style, word_leaf_ids)
                    .expect("Expected to create word_group_node");
                line_row_child_ids.push(word_group_node_id);
            }
        }
        word_leaf_ids.clear();
    }

    /// Builds a single token leaf node for `md_token_item`.
    ///
    /// `Word` tokens become `MdToken` leaves. A marker word (with `align_cols`)
    /// is given a left margin so its text is right-aligned within the marker
    /// column, lining up ordered-list digits without padding the text with
    /// spaces. `Image` tokens become fixed-size `MdImage` leaves.
    fn token_leaf_build(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        md_token_item: &MdTokenItem,
        char_width: f32,
    ) -> taffy::NodeId {
        match md_token_item {
            MdTokenItem::Word {
                text,
                md_style,
                align_cols,
                ..
            } => {
                let style = match align_cols {
                    Some(align_cols) => {
                        // Right-align the marker within an `align_cols`-wide
                        // column: the left margin is the column width minus the
                        // marker's own width, so the marker's right edge (and the
                        // body text after the gap) is uniform across the list.
                        let column_width = (f32::from(*align_cols) * char_width).ceil();
                        let marker_width = md_token_width_measure(text, char_width);
                        let margin_left = (column_width - marker_width).max(0.0);
                        Style {
                            margin: Rect {
                                left: LengthPercentageAuto::length(margin_left),
                                right: LengthPercentageAuto::length(0.0),
                                top: LengthPercentageAuto::length(0.0),
                                bottom: LengthPercentageAuto::length(0.0),
                            },
                            ..Default::default()
                        }
                    }
                    None => Style::default(),
                };
                taffy_tree
                    .new_leaf_with_context(
                        style,
                        TaffyNodeCtx::MdToken(MdTokenCtx {
                            text: text.clone(),
                            md_style: md_style.clone(),
                        }),
                    )
                    .expect("Expected to create MdToken leaf")
            }
            MdTokenItem::Image { src, alt, .. } => {
                let (width, height) = MdImageSizer::compute_size(md_token_item);
                taffy_tree
                    .new_leaf_with_context(
                        Style {
                            size: Size {
                                width: taffy::style::Dimension::length(width),
                                height: taffy::style::Dimension::length(height),
                            },
                            ..Default::default()
                        },
                        TaffyNodeCtx::MdImage(MdImageCtx {
                            src: src.clone(),
                            alt: alt.clone(),
                            width,
                            height,
                        }),
                    )
                    .expect("Expected to create MdImage leaf")
            }
            MdTokenItem::LineBreak => {
                unreachable!("LineBreak tokens are filtered before build_block_line_row")
            }
        }
    }
}
