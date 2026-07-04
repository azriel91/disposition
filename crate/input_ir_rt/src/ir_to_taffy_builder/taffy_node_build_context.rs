use std::borrow::Cow;

use disposition_ir_model::{
    edge::{EdgeId, EdgeLabels},
    layout::LeafLayout,
    node::{NodeFace, NodeId},
};
use disposition_model_common::Map;
use disposition_taffy_model::{
    taffy::{
        self, style::FlexDirection, AlignContent, AlignItems, AvailableSpace, Display, FlexWrap,
        Size, Style,
    },
    DiagramLod, MdNodeTaffyIds, TaffyNodeCtx, TEXT_LINE_HEIGHT,
};
use taffy::{LengthPercentage, LengthPercentageAuto, Rect};

use super::{
    taffy_build_ctx::TaffyBuildCtx,
    text_measure::{compute_text_dimensions, md_token_width_measure},
};

/// Layout information for a wrapper node and its text node.
pub(crate) struct TaffyWrapperNodeStyles {
    pub(crate) wrapper_style: Style,
    pub(crate) text_style: Style,
    pub(crate) child_container_style: Style,
}

impl TaffyWrapperNodeStyles {
    pub fn new(leaf_layout: &LeafLayout) -> Self {
        Self {
            wrapper_style: Style {
                display: Display::Flex,
                max_size: Size::auto(),
                flex_direction: FlexDirection::Column,
                flex_wrap: FlexWrap::NoWrap,
                margin: Rect {
                    left: LengthPercentageAuto::length(leaf_layout.margin_left()),
                    right: LengthPercentageAuto::length(leaf_layout.margin_right()),
                    top: LengthPercentageAuto::length(leaf_layout.margin_top()),
                    bottom: LengthPercentageAuto::length(leaf_layout.margin_bottom()),
                },
                padding: Rect {
                    left: LengthPercentage::length(leaf_layout.padding_left()),
                    right: LengthPercentage::length(leaf_layout.padding_right()),
                    top: LengthPercentage::length(leaf_layout.padding_top()),
                    bottom: LengthPercentage::length(leaf_layout.padding_bottom()),
                },
                ..Default::default()
            },
            text_style: Style::default(),
            child_container_style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                align_items: Some(AlignItems::Start),
                justify_items: Some(AlignItems::Start),
                align_content: Some(AlignContent::Start),
                justify_content: Some(AlignContent::Start),
                ..Default::default()
            },
        }
    }
}

impl Default for TaffyWrapperNodeStyles {
    fn default() -> Self {
        Self {
            wrapper_style: Style {
                display: Display::Flex,
                max_size: Size::auto(),
                flex_direction: FlexDirection::Column,
                flex_wrap: FlexWrap::NoWrap,
                ..Default::default()
            },
            text_style: Style::default(),
            child_container_style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                align_items: Some(AlignItems::Start),
                justify_items: Some(AlignItems::Start),
                align_content: Some(AlignContent::Start),
                justify_content: Some(AlignContent::Start),
                ..Default::default()
            },
        }
    }
}

pub(crate) struct NodeMeasureContext<'ctx> {
    /// Immutable build context (node names, descriptions, precomputed text,
    /// character width, and level of detail).
    pub(crate) ctx: TaffyBuildCtx<'ctx>,
    /// Text labels for each edge endpoint.
    pub(crate) edge_labels: &'ctx EdgeLabels<'static>,
    /// Pre-computed lookup from edge ID to its `from` and `to` endpoint node
    /// IDs.
    ///
    /// Used to determine which endpoint text (`from` or `to`) to use when
    /// sizing an edge label slot.
    pub(crate) edge_id_to_endpoint_node_ids:
        &'ctx Map<EdgeId<'static>, (NodeId<'static>, NodeId<'static>)>,
}

impl NodeMeasureContext<'_> {
    /// Returns the size of a node based on its text content and available
    /// space.
    ///
    /// Called during taffy layout computation as a measure callback. Only
    /// computes sizes -- syntax highlighting is deferred to a separate pass
    /// after layout is complete.
    ///
    /// Nodes without text content (edge spacers, empty wrapper containers)
    /// return zero size immediately so they do not contribute spurious height.
    pub(crate) fn size_measure(
        &mut self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        taffy_node_ctx: Option<&mut TaffyNodeCtx>,
        style: &Style,
    ) -> Size<f32> {
        if let Size {
            width: Some(width),
            height: Some(height),
        } = known_dimensions
        {
            return Size { width, height };
        }

        let NodeMeasureContext {
            ctx,
            edge_labels,
            edge_id_to_endpoint_node_ids,
        } = self;
        let ctx = *ctx;
        let char_width = ctx.char_width;
        let lod = ctx.lod;
        let edge_descs = ctx.edge_descs;

        // MdToken leaves are sized per-token using heading-level font scaling.
        if let Some(md_token_ctx) = taffy_node_ctx.as_ref().and_then(|taffy_node_ctx| {
            if let TaffyNodeCtx::MdToken(md_token_ctx) = taffy_node_ctx {
                Some(md_token_ctx)
            } else {
                None
            }
        }) {
            let effective_char_width = char_width;
            let effective_line_height = TEXT_LINE_HEIGHT;
            let width = md_token_width_measure(&md_token_ctx.text, effective_char_width);
            return Size {
                width,
                height: effective_line_height,
            };
        }

        // Edge spacers, edge labels, and empty wrapper containers (no context)
        // have no text to measure.  Return zero size immediately so that
        // empty face-wrapper rows/columns (e.g. `edge_wrapper_top` when a
        // node has no top-face edges) do not contribute spurious height via
        // the `(line_count + 0.5) * line_height` bias.
        let text = match taffy_node_ctx
            .as_ref()
            .and_then(|taffy_node_ctx| match taffy_node_ctx {
                TaffyNodeCtx::EdgeSpacer(_) => None,
                TaffyNodeCtx::EdgeDescription(edge_desc_ctx) => match lod {
                    DiagramLod::Simple => None,
                    DiagramLod::Normal => {
                        let edge_id = &edge_desc_ctx.edge_id;
                        ctx.edge_id_to_group_id
                            .get(edge_id)
                            .and_then(|edge_group_id| {
                                edge_descs.get_for_edge(edge_id, edge_group_id)
                            })
                            .map(|desc| Cow::Borrowed(desc.as_str()))
                    }
                },
                TaffyNodeCtx::EdgeLabel(edge_label_ctx) => match lod {
                    DiagramLod::Simple => None,
                    DiagramLod::Normal => {
                        let edge_id = &edge_label_ctx.edge_id;
                        let node_id = &edge_label_ctx.node_id;
                        ctx.edge_id_to_group_id
                            .get(edge_id)
                            .and_then(|edge_group_id| {
                                edge_labels.get_for_edge(edge_id, edge_group_id)
                            })
                            .and_then(|edge_label| {
                                // Use the from or to text depending on which
                                // endpoint this label slot is attached to.
                                let is_from_endpoint = edge_id_to_endpoint_node_ids
                                    .get(edge_id)
                                    .map(|(from_node_id, _)| from_node_id == node_id)
                                    .unwrap_or(false);
                                let text = if is_from_endpoint {
                                    edge_label.from.as_str()
                                } else {
                                    edge_label.to.as_str()
                                };
                                if text.is_empty() {
                                    None
                                } else {
                                    Some(Cow::Borrowed(text))
                                }
                            })
                    }
                },
                TaffyNodeCtx::MdToken(_) | TaffyNodeCtx::MdImage(_) => None,
            }) {
            Some(text) => text,
            None => {
                return Size {
                    width: 0.0,
                    height: 0.0,
                }
            }
        };

        // Set width constraint
        let width_constraint = known_dimensions.width.or(match available_space.width {
            AvailableSpace::MinContent => Some(0.0),
            AvailableSpace::MaxContent => None,
            AvailableSpace::Definite(width) => Some(width),
        });

        // Compute layout using simple monospace calculations
        let (line_width_max, line_count) =
            compute_text_dimensions(&text, char_width, width_constraint);

        let line_height = TEXT_LINE_HEIGHT;
        let line_heights = (line_count as f32 + 0.5) * line_height;

        Size {
            width: line_width_max
                + style.border.left.into_raw().value()
                + style.border.right.into_raw().value()
                + style.padding.left.into_raw().value()
                + style.padding.right.into_raw().value(),
            height: line_heights
                + style.border.top.into_raw().value()
                + style.border.bottom.into_raw().value()
                + style.padding.top.into_raw().value()
                + style.padding.bottom.into_raw().value(),
        }
    }
}

/// A single edge label leaf node built during envelope node construction.
///
/// Collected across all envelope nodes so that `edge_label_taffy_nodes` can
/// be populated in `TaffyNodeMappings` at the end of Phase 2 Step 2.5.
pub(crate) struct EdgeLabelLeafBuilt {
    /// The edge ID this label leaf belongs to.
    pub(crate) edge_id: EdgeId<'static>,
    /// The endpoint node this label leaf is attached to.
    pub(crate) node_id: NodeId<'static>,
    /// The face of the endpoint node this label leaf is on.
    // Seems sensible to hold which `NodeFace` the label leaf is on, even if it is not read now.
    #[allow(unused)]
    pub(crate) face: NodeFace,
    /// The taffy node ID of the label slot.
    ///
    /// At [`DiagramLod::Normal`] this is the slot container that wraps the
    /// markdown content node; at [`DiagramLod::Simple`] it is the placeholder
    /// leaf.
    ///
    /// [`DiagramLod::Normal`]: disposition_taffy_model::DiagramLod::Normal
    /// [`DiagramLod::Simple`]: disposition_taffy_model::DiagramLod::Simple
    pub(crate) taffy_node_id: taffy::NodeId,
    /// Markdown sub-tree IDs for this label slot, when built via the markdown
    /// content path ([`DiagramLod::Normal`] with non-empty label text).
    ///
    /// [`DiagramLod::Normal`]: disposition_taffy_model::DiagramLod::Normal
    pub(crate) md_node_taffy_ids: Option<MdNodeTaffyIds>,
}
