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
        blocks: &[MdBlock],
        char_width: f32,
    ) -> MdNodeTaffyIds {
        let mut block_taffy_ids = Vec::with_capacity(blocks.len());

        for block in blocks {
            let mut token_node_ids = Vec::with_capacity(block.tokens.len());

            for token in &block.tokens {
                let leaf_id = match token {
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
                        let (width, height) = MdImageSizer::compute_size(token);
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
                token_node_ids.push(leaf_id);
            }

            let block_row_style = Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                gap: Size {
                    width: LengthPercentage::length(char_width),
                    height: LengthPercentage::length(0.0),
                },
                ..Default::default()
            };
            let block_row_node_id = taffy_tree
                .new_with_children(block_row_style, &token_node_ids)
                .expect("Expected to create block_row_node");

            block_taffy_ids.push(MdBlockTaffyIds {
                block_row_node_id,
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
        let block_row_node_ids: Vec<taffy::NodeId> = block_taffy_ids
            .iter()
            .map(|b| b.block_row_node_id)
            .collect();
        let content_node_id = taffy_tree
            .new_with_children(content_node_style, &block_row_node_ids)
            .expect("Expected to create md_content_node");

        MdNodeTaffyIds {
            content_node_id,
            block_taffy_ids,
        }
    }
}
