use disposition_ir_model::{
    layout::{NodeLayout, NodeLayouts},
    node::NodeInbuilt,
};
use disposition_taffy_model::{
    taffy::{
        self,
        style::{FlexDirection, LengthPercentageAuto},
        AlignContent, AlignItems, Display, FlexWrap, LengthPercentage, Rect, Size, Style,
        TaffyTree,
    },
    DimensionAndLod, IrToTaffyError, NodeContext, TaffyTreeAndRoot,
};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// Maps an intermediate representation diagram to a `TaffyTreeAndRoot`.
///
/// # Examples
///
/// ```rust
/// use disposition_input_ir_rt::IrToTaffyBuilder;
///
/// let mut taffy_trees = IrToTaffyBuilder::default()
///     .node_layouts(node_layouts)
///     .build();
/// ```
#[derive(Clone, Debug, Deserialize, Serialize, TypedBuilder)]
pub struct IrToTaffyBuilder {
    /// The dimensions at which elements should be repositioned.
    #[builder(setter(prefix = "with_"))]
    dimension_and_lods: Vec<DimensionAndLod>,
    /// Node layouts for the diagram.
    #[builder(setter(prefix = "with_"))]
    node_layouts: NodeLayouts,
}

impl IrToTaffyBuilder {
    /// Returns an iterator over `TaffyTreeAndRoot` instances for each
    /// dimension.
    pub fn build(self) -> Result<impl Iterator<Item = TaffyTreeAndRoot>, IrToTaffyError> {
        let IrToTaffyBuilder {
            dimension_and_lods,
            node_layouts,
        } = self;

        let taffy_tree_and_root_iter =
            dimension_and_lods
                .into_iter()
                .map(move |dimension_and_lod| {
                    Self::build_taffy_tree_for_dimension(dimension_and_lod, &node_layouts)
                });

        Ok(taffy_tree_and_root_iter)
    }

    /// Returns a `TaffyTreeAndRoot` with all processes as part of the diagram.
    ///
    /// This includes the processes container. Clicking on each process node
    /// reveals the process steps.
    fn build_taffy_tree_for_dimension(
        dimension_and_lod: DimensionAndLod,
        node_layouts: &NodeLayouts,
    ) -> TaffyTreeAndRoot {
        // TODO: use `lod` to determine whether text is rendered, which affects the
        // layout calculation.
        let DimensionAndLod { dimension, lod: _ } = dimension_and_lod;

        let mut taffy_tree = TaffyTree::new();
        let root = Self::taffy_container_node(
            &mut taffy_tree,
            &node_layouts,
            NodeInbuilt::Root,
            Size::from_lengths(dimension.width(), dimension.height()),
        );
        let tags_container = Self::taffy_container_node(
            &mut taffy_tree,
            &node_layouts,
            NodeInbuilt::TagsContainer,
            Size::auto(),
        );
        let things_and_processes_container = Self::taffy_container_node(
            &mut taffy_tree,
            &node_layouts,
            NodeInbuilt::ThingsAndProcessesContainer,
            Size::auto(),
        );
        let things_container = Self::taffy_container_node(
            &mut taffy_tree,
            &node_layouts,
            NodeInbuilt::ThingsContainer,
            Size::auto(),
        );
        let processes_container = Self::taffy_container_node(
            &mut taffy_tree,
            &node_layouts,
            NodeInbuilt::ProcessesContainer,
            Size::auto(),
        );

        // Order is important here because of flex direction.
        taffy_tree.add_child(root, tags_container).expect(
            "`taffy_tree.add_child(root, tags_container)` failed, but should be infallible.",
        );
        taffy_tree
            .add_child(root, things_and_processes_container)
            .expect("`taffy_tree.add_child(root, things_and_processes_container)` failed, but should be infallible.");
        taffy_tree
            .add_child(things_and_processes_container, processes_container)
            .expect("`taffy_tree.add_child(things_and_processes_container, processes_container)` failed, but should be infallible.");
        taffy_tree
            .add_child(things_and_processes_container, things_container)
            .expect("`taffy_tree.add_child(things_and_processes_container, things_container)` failed, but should be infallible.");

        TaffyTreeAndRoot { taffy_tree, root }
    }

    /// Adds a container node to the `TaffyTree` and returns its ID.
    ///
    /// # Parameters
    ///
    /// * `taffy_tree`: `TaffyTree` to add the node to.
    /// * `node_layouts`: Flex layout / none computed when mapping the
    ///   `InputDiagram` to the `IrDiagram`.
    /// * `node_inbuilt`: The `NodeInbuilt` struct representing the node.
    /// * `max_size`: Maximum size of the node.
    fn taffy_container_node(
        taffy_tree: &mut TaffyTree<NodeContext>,
        node_layouts: &NodeLayouts,
        node_inbuilt: NodeInbuilt,
        max_size: Size<taffy::Dimension>,
    ) -> taffy::NodeId {
        let tags_container_style =
            Self::taffy_container_style(node_layouts, node_inbuilt.id(), max_size);
        taffy_tree
            .new_leaf_with_context(
                tags_container_style,
                NodeContext {
                    entity_id: node_inbuilt.id(),
                    entity_type: node_inbuilt.entity_type(),
                },
            )
            .expect("`TaffyTree::new_leaf_with_context` should be infallible.")
    }

    /// Returns the `taffy::Style` for container nodes.
    fn taffy_container_style(
        node_layouts: &NodeLayouts,
        node_id: disposition_model_common::Id,
        max_size: Size<taffy::Dimension>,
    ) -> Style {
        let style = node_layouts
            .get(&node_id)
            .map(|node_layout| match node_layout {
                NodeLayout::Flex(flex_layout) => {
                    let flex_style = Style {
                        display: Display::Flex,
                        max_size,
                        margin: Rect {
                            left: LengthPercentageAuto::length(flex_layout.margin_left()),
                            right: LengthPercentageAuto::length(flex_layout.margin_right()),
                            top: LengthPercentageAuto::length(flex_layout.margin_top()),
                            bottom: LengthPercentageAuto::length(flex_layout.margin_bottom()),
                        },
                        padding: Rect {
                            left: LengthPercentage::length(flex_layout.padding_left()),
                            right: LengthPercentage::length(flex_layout.padding_right()),
                            top: LengthPercentage::length(flex_layout.padding_top()),
                            bottom: LengthPercentage::length(flex_layout.padding_bottom()),
                        },
                        border: Rect::length(1.0f32),
                        align_items: Some(AlignItems::Center),
                        justify_items: Some(AlignItems::Center),
                        align_content: Some(AlignContent::Center),
                        justify_content: Some(AlignContent::Center),
                        gap: Size::length(flex_layout.gap()),
                        flex_direction: FlexDirection::from(flex_layout.direction()),
                        flex_wrap: if flex_layout.wrap() {
                            FlexWrap::Wrap
                        } else {
                            FlexWrap::NoWrap
                        },
                        ..Default::default()
                    };
                    flex_style
                }
                NodeLayout::None => Style::default(),
            })
            .unwrap_or_default();
        style
    }
}
