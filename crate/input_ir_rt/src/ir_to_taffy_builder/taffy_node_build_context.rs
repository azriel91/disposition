use disposition_ir_model::{
    edge::EdgeId,
    entity::EntityTypes,
    layout::{LeafLayout, NodeLayouts},
    node::{
        NodeFace, NodeFaceEdges, NodeHierarchy, NodeId, NodeNames, NodeNestingInfos,
        NodeRanksNested, NodeShapes,
    },
};
use disposition_model_common::{entity::EntityDescs, Map, RankDir};
use disposition_taffy_model::{
    taffy::{
        self, style::FlexDirection, AlignContent, AlignItems, Display, FlexWrap, Size, Style,
        TaffyTree,
    },
    DiagramLod, NodeToTaffyNodeIds, TaffyNodeCtx,
};
use taffy::{LengthPercentage, LengthPercentageAuto, Rect};

pub(crate) struct TaffyNodeBuildContext<'ctx> {
    pub(crate) taffy_tree: &'ctx mut TaffyTree<TaffyNodeCtx>,
    pub(crate) nodes: &'ctx NodeNames<'static>,
    pub(crate) node_layouts: &'ctx NodeLayouts<'static>,
    pub(crate) node_hierarchy: &'ctx NodeHierarchy<'static>,
    pub(crate) entity_types: &'ctx EntityTypes<'static>,
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
    pub(crate) entity_descs: &'ctx EntityDescs<'static>,
    /// Monospace character width in pixels.
    pub(crate) char_width: f32,
    /// Level of detail for the diagram.
    pub(crate) lod: &'ctx DiagramLod,
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
