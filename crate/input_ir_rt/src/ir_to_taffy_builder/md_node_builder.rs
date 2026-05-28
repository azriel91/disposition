use disposition_taffy_model::{
    taffy::{self, AlignItems, Display, FlexDirection, FlexWrap, Size, Style, TaffyTree},
    MdBlockTaffyIds, MdImageCtx, MdNodeTaffyIds, MdTokenCtx, TaffyNodeCtx, TEXT_LINE_HEIGHT,
};
use taffy::LengthPercentage;

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

        for md_block in md_blocks {
            let (block_col_node_id, token_node_ids) =
                Self::build_block(taffy_tree, md_block, char_width);

            md_block_taffy_ids_list.push(MdBlockTaffyIds {
                block_col_node_id,
                token_node_ids,
            });
        }

        let content_node_style = Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            flex_wrap: FlexWrap::NoWrap,
            align_items: Some(AlignItems::FlexStart),
            gap: Size {
                width: LengthPercentage::length(0.0),
                height: LengthPercentage::length(TEXT_LINE_HEIGHT),
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

    /// Builds the `block_col_node` for one `MdBlock`.
    ///
    /// Tokens are split at `LineBreak` boundaries into groups. Each group
    /// becomes a `line_row_node` (flex row wrap). All `line_row_nodes` are
    /// children of the returned `block_col_node` (flex column, no gap).
    ///
    /// Returns `(block_col_node_id, all_token_leaf_node_ids)`.
    fn build_block(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        md_block: &MdBlock,
        char_width: f32,
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
