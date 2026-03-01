use disposition_ir_model::{
    entity::EntityTypes,
    layout::NodeLayouts,
    node::{NodeHierarchy, NodeId, NodeNames, NodeShapes},
};
use disposition_model_common::{entity::EntityDescs, Map};
use disposition_taffy_model::{
    taffy::{
        self, style::FlexDirection, AlignContent, AlignItems, Display, FlexWrap, Size, Style,
        TaffyTree,
    },
    DiagramLod, NodeContext, NodeToTaffyNodeIds,
};

pub(crate) struct TaffyNodeBuildContext<'ctx> {
    pub(crate) taffy_tree: &'ctx mut TaffyTree<NodeContext>,
    pub(crate) nodes: &'ctx NodeNames<'static>,
    pub(crate) node_layouts: &'ctx NodeLayouts<'static>,
    pub(crate) node_hierarchy: &'ctx NodeHierarchy<'static>,
    pub(crate) entity_types: &'ctx EntityTypes<'static>,
    pub(crate) node_shapes: &'ctx NodeShapes<'static>,
    pub(crate) node_id_to_taffy: &'ctx mut Map<NodeId<'static>, NodeToTaffyNodeIds>,
    pub(crate) taffy_id_to_node: &'ctx mut Map<taffy::NodeId, NodeId<'static>>,
}

/// Layout information for a wrapper node and its text node.
pub(crate) struct TaffyWrapperNodeStyles {
    pub(crate) wrapper_style: Style,
    pub(crate) text_style: Style,
    pub(crate) child_container_style: Style,
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
