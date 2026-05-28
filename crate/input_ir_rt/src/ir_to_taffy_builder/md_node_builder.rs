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
    /// Builds a flex-column `md_content_node` containing one flex-row-wrap
    /// `block_row_node` per `MdBlock`, each holding word and image leaf nodes.
    ///
    /// Returns the `MdNodeTaffyIds` describing the full sub-tree.
    pub(crate) fn build(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        md_blocks: &[MdBlock],
        char_width: f32,
    ) -> MdNodeTaffyIds {
        let mut md_block_taffy_ids_list = Vec::with_capacity(md_blocks.len());

        for md_block in md_blocks {
            let mut md_token_taffy_node_ids = Vec::with_capacity(md_block.tokens.len());

            for md_token_item in &md_block.tokens {
                let md_token_taffy_node_id = match md_token_item {
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
                        // Insert a zero-height line break element that takes full width,
                        // forcing subsequent tokens to wrap to the next line.
                        taffy_tree
                            .new_leaf(Style {
                                size: Size {
                                    width: taffy::style::Dimension::percent(1.0),
                                    height: taffy::style::Dimension::length(0.0),
                                },
                                ..Default::default()
                            })
                            .expect("Expected to create LineBreak leaf")
                    }
                };
                md_token_taffy_node_ids.push(md_token_taffy_node_id);
            }

            let block_row_style = Style {
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
            let block_row_node_id = taffy_tree
                .new_with_children(block_row_style, &md_token_taffy_node_ids)
                .expect("Expected to create block_row_node");

            md_block_taffy_ids_list.push(MdBlockTaffyIds {
                block_row_node_id,
                token_node_ids: md_token_taffy_node_ids,
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
        let block_row_node_ids: Vec<taffy::NodeId> = md_block_taffy_ids_list
            .iter()
            .map(|md_block_taffy_ids| md_block_taffy_ids.block_row_node_id)
            .collect();
        let content_node_id = taffy_tree
            .new_with_children(content_node_style, &block_row_node_ids)
            .expect("Expected to create md_content_node");

        MdNodeTaffyIds {
            content_node_id,
            block_taffy_ids: md_block_taffy_ids_list,
        }
    }
}
