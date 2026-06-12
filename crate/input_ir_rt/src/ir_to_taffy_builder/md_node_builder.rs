use disposition_taffy_model::{
    MdBlockTaffyIds, MdImageCtx, MdNodeTaffyIds, MdTokenCtx, TaffyNodeCtx, TEXT_LINE_HEIGHT,
};
use taffy::{
    self, AlignItems, Display, FlexDirection, FlexWrap, LengthPercentage, LengthPercentageAuto,
    Rect, Size, Style, TaffyTree,
};

use crate::md_text::{
    md_blocks_parser::{MdBlock, MdTokenItem},
    md_image_sizer::MdImageSizer,
};

pub(crate) struct MdNodeBuilder;

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
    /// Returns `(line_row_node_id, token_leaf_node_ids)`.
    fn build_block_line_row(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        line_tokens: &[&MdTokenItem],
        char_width: f32,
    ) -> (taffy::NodeId, Vec<taffy::NodeId>) {
        let mut token_node_ids: Vec<taffy::NodeId> = Vec::with_capacity(line_tokens.len());

        for md_token_item in line_tokens {
            let token_node_id = match md_token_item {
                MdTokenItem::Word { text, md_style } => taffy_tree
                    .new_leaf_with_context(
                        Style::default(),
                        TaffyNodeCtx::MdToken(MdTokenCtx {
                            text: text.clone(),
                            md_style: md_style.clone(),
                        }),
                    )
                    .expect("Expected to create MdToken leaf"),
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
            };
            token_node_ids.push(token_node_id);
        }

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
            .new_with_children(line_row_style, &token_node_ids)
            .expect("Expected to create line_row_node");

        (line_row_node_id, token_node_ids)
    }
}
