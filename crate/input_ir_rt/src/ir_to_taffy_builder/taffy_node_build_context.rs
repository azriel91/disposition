use std::borrow::Cow;

use disposition_ir_model::{
    edge::{EdgeId, EdgeLabels},
    entity::EntityTypes,
    layout::{LeafLayout, NodeLayouts},
    node::{
        NodeFace, NodeFaceEdges, NodeHierarchy, NodeId, NodeNames, NodeNestingInfos,
        NodeRanksNested, NodeShapes,
    },
};
use disposition_model_common::{edge::EdgeDescs, thing::ThingDescs, Map, RankDir};
use disposition_taffy_model::{
    taffy::{
        self, style::FlexDirection, AlignContent, AlignItems, AvailableSpace, Display, FlexWrap,
        Size, Style, TaffyTree,
    },
    DiagramLod, MdHeadingLevel, MdNodeTaffyIds, NodeToTaffyNodeIds, TaffyNodeCtx, TEXT_LINE_HEIGHT,
};
use taffy::{LengthPercentage, LengthPercentageAuto, Rect};

use super::text_measure::{compute_text_dimensions, line_width_measure};

pub(crate) struct TaffyNodeBuildContext<'ctx> {
    pub(crate) taffy_tree: &'ctx mut TaffyTree<TaffyNodeCtx>,
    pub(crate) nodes: &'ctx NodeNames<'static>,
    pub(crate) node_layouts: &'ctx NodeLayouts<'static>,
    pub(crate) node_hierarchy: &'ctx NodeHierarchy<'static>,
    pub(crate) entity_types: &'ctx EntityTypes<'static>,
    pub(crate) thing_descs: &'ctx ThingDescs<'static>,
    pub(crate) edge_descs: &'ctx EdgeDescs<'static>,
    pub(crate) node_shapes: &'ctx NodeShapes<'static>,
    pub(crate) node_ranks_nested: &'ctx NodeRanksNested<'static>,
    pub(crate) node_nesting_infos: &'ctx NodeNestingInfos<'static>,
    pub(crate) node_id_to_taffy: &'ctx mut Map<NodeId<'static>, NodeToTaffyNodeIds>,
    pub(crate) taffy_id_to_node: &'ctx mut Map<taffy::NodeId, NodeId<'static>>,
    /// Per-node face-to-edge-IDs mapping used to build envelope label slots.
    pub(crate) node_face_edges: &'ctx NodeFaceEdges<'static>,
    /// Map from each diagram node ID to its envelope taffy node ID.
    ///
    /// Populated incrementally as each node's envelope is built.
    pub(crate) node_id_to_envelope_taffy_node: &'ctx mut Map<NodeId<'static>, taffy::NodeId>,
    /// Accumulator for edge label leaf nodes built across all envelope nodes.
    ///
    /// After all nodes are built, merged into `edge_label_taffy_nodes` in
    /// `TaffyNodeMappings`.
    pub(crate) edge_label_leaves: &'ctx mut Vec<EdgeLabelLeafBuilt>,
    /// Direction of edges in the diagram.
    ///
    /// Used to compute face-specific padding for edge label leaf nodes.
    pub(crate) rank_dir: RankDir,
    /// Level of detail for this diagram build.
    pub(crate) lod: DiagramLod,
    /// Monospace character width in pixels.
    pub(crate) char_width: f32,
    /// Accumulator for md node taffy IDs built across all diagram nodes.
    pub(crate) md_node_taffy_ids: &'ctx mut Map<NodeId<'static>, MdNodeTaffyIds>,
}

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
    pub(crate) nodes: &'ctx NodeNames<'static>,
    pub(crate) thing_descs: &'ctx ThingDescs<'static>,
    pub(crate) edge_descs: &'ctx EdgeDescs<'static>,
    /// Text labels for each edge endpoint.
    pub(crate) edge_labels: &'ctx EdgeLabels<'static>,
    /// Pre-computed lookup from edge ID to its `from` and `to` endpoint node
    /// IDs.
    ///
    /// Used to determine which endpoint text (`from` or `to`) to use when
    /// sizing an edge label slot.
    pub(crate) edge_id_to_endpoint_node_ids:
        &'ctx Map<EdgeId<'static>, (NodeId<'static>, NodeId<'static>)>,
    /// Monospace character width in pixels.
    pub(crate) char_width: f32,
    /// Level of detail for the diagram.
    pub(crate) lod: &'ctx DiagramLod,
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
            nodes,
            thing_descs,
            edge_descs,
            edge_labels,
            edge_id_to_endpoint_node_ids,
            char_width,
            lod,
        } = self;

        // MdToken leaves are sized per-token using heading-level font scaling.
        if let Some(ctx) = taffy_node_ctx.as_ref().and_then(|n| {
            if let TaffyNodeCtx::MdToken(ctx) = n {
                Some(ctx)
            } else {
                None
            }
        }) {
            let font_scale = ctx
                .md_style
                .heading_level
                .map(MdHeadingLevel::font_scale)
                .unwrap_or(1.0);
            let effective_char_width = *char_width * font_scale;
            let effective_line_height = TEXT_LINE_HEIGHT * font_scale;
            let width = line_width_measure(&ctx.text, effective_char_width);
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
                TaffyNodeCtx::DiagramNode(diagram_node_ctx) => {
                    let entity_id = &diagram_node_ctx.entity_id;
                    let node_name = nodes
                        .get(entity_id)
                        .map(String::as_str)
                        .unwrap_or_else(|| entity_id.as_str());

                    match lod {
                        DiagramLod::Simple => Some(Cow::Borrowed(node_name)),
                        DiagramLod::Normal => {
                            let node_desc = thing_descs.get(entity_id).map(String::as_str);

                            match node_desc {
                                Some(desc) => Some(Cow::Owned(format!("# {node_name}\n\n{desc}"))),
                                None => Some(Cow::Borrowed(node_name)),
                            }
                        }
                    }
                }
                TaffyNodeCtx::EdgeSpacer(_) => None,
                TaffyNodeCtx::EdgeDescription(ctx) => match lod {
                    DiagramLod::Simple => None,
                    DiagramLod::Normal => {
                        let edge_id = &ctx.edge_id;
                        edge_descs
                            .get(edge_id.as_ref())
                            .map(|desc| Cow::Borrowed(desc.as_str()))
                    }
                },
                TaffyNodeCtx::EdgeLabel(ctx) => match lod {
                    DiagramLod::Simple => None,
                    DiagramLod::Normal => {
                        let edge_id = &ctx.edge_id;
                        let node_id = &ctx.node_id;
                        edge_labels.get(edge_id).and_then(|edge_label| {
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
            compute_text_dimensions(&text, *char_width, width_constraint);

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
    /// The taffy node ID of the label leaf.
    pub(crate) taffy_node_id: taffy::NodeId,
}
