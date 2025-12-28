use disposition_ir_model::{
    entity::{EntityType, EntityTypes},
    layout::{NodeLayout, NodeLayouts},
    node::{NodeHierarchy, NodeInbuilt},
    IrDiagram,
};
use disposition_taffy_model::{
    taffy::{
        self,
        style::{FlexDirection, LengthPercentageAuto},
        AlignContent, AlignItems, Display, FlexWrap, LengthPercentage, Rect, Size, Style,
        TaffyTree,
    },
    DimensionAndLod, IrToTaffyError, NodeContext, ProcessesIncluded, TaffyTreeAndRoot,
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
    /// The intermediate representation of the diagram to render the taffy trees
    /// for.
    #[builder(setter(prefix = "with_"))]
    ir_diagram: IrDiagram,
    /// The dimensions at which elements should be repositioned.
    #[builder(setter(prefix = "with_"))]
    dimension_and_lods: Vec<DimensionAndLod>,
    /// What processes to create diagrams for.
    #[builder(setter(prefix = "with_"))]
    processes_included: ProcessesIncluded,
}

impl IrToTaffyBuilder {
    /// Returns an iterator over `TaffyTreeAndRoot` instances for each
    /// dimension.
    pub fn build(&self) -> Result<impl Iterator<Item = TaffyTreeAndRoot>, IrToTaffyError> {
        let IrToTaffyBuilder {
            ir_diagram,
            dimension_and_lods,
            processes_included,
        } = self;

        let taffy_tree_and_root_iter =
            dimension_and_lods
                .iter()
                .flat_map(move |dimension_and_lod| {
                    Self::build_taffy_trees_for_dimension(
                        ir_diagram,
                        dimension_and_lod,
                        processes_included,
                    )
                });

        Ok(taffy_tree_and_root_iter)
    }

    /// Returns a `TaffyTreeAndRoot` with all processes as part of the diagram.
    ///
    /// This includes the processes container. Clicking on each process node
    /// reveals the process steps.
    fn build_taffy_trees_for_dimension(
        ir_diagram: &IrDiagram,
        dimension_and_lod: &DimensionAndLod,
        processes_included: &ProcessesIncluded,
    ) -> impl Iterator<Item = TaffyTreeAndRoot> {
        let IrDiagram {
            nodes,
            node_copy_text,
            node_hierarchy,
            edge_groups,
            entity_descs,
            entity_types,
            tailwind_classes,
            node_layouts,
            css,
        } = ir_diagram;

        // TODO: use `lod` to determine whether text is rendered, which affects the
        // layout calculation.
        let DimensionAndLod { dimension, lod: _ } = dimension_and_lod;

        let mut taffy_tree = TaffyTree::new();
        let (root, tags_container, things_container, processes_container) =
            Self::build_taffy_container_nodes(node_layouts, dimension, &mut taffy_tree);

        Self::build_taffy_nodes_for_first_level_nodes(
            &mut taffy_tree,
            node_layouts,
            entity_types,
            node_hierarchy,
            tags_container,
            things_container,
            processes_container,
        );

        std::iter::once(TaffyTreeAndRoot { taffy_tree, root })
    }

    /// Adds the inbuilt container nodes to the `TaffyTree`.
    fn build_taffy_container_nodes(
        node_layouts: &NodeLayouts,
        dimension: &disposition_taffy_model::Dimension,
        taffy_tree: &mut TaffyTree<NodeContext>,
    ) -> (taffy::NodeId, taffy::NodeId, taffy::NodeId, taffy::NodeId) {
        let root = Self::taffy_container_node(
            taffy_tree,
            &node_layouts,
            NodeInbuilt::Root,
            Size::from_lengths(dimension.width(), dimension.height()),
        );
        let tags_container = Self::taffy_container_node(
            taffy_tree,
            &node_layouts,
            NodeInbuilt::TagsContainer,
            Size::auto(),
        );
        let things_and_processes_container = Self::taffy_container_node(
            taffy_tree,
            &node_layouts,
            NodeInbuilt::ThingsAndProcessesContainer,
            Size::auto(),
        );
        let things_container = Self::taffy_container_node(
            taffy_tree,
            &node_layouts,
            NodeInbuilt::ThingsContainer,
            Size::auto(),
        );
        let processes_container = Self::taffy_container_node(
            taffy_tree,
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
        (root, tags_container, things_container, processes_container)
    }

    /// Adds the tags, things, and process nodes to the taffy tree.
    ///
    /// This is different from `build_taffy_nodes_for_node` in that the parent
    /// node is one of the container nodes.
    fn build_taffy_nodes_for_first_level_nodes(
        taffy_tree: &mut TaffyTree<NodeContext>,
        node_layouts: &NodeLayouts,
        entity_types: &EntityTypes,
        node_hierarchy: &NodeHierarchy,
        tags_container: taffy::NodeId,
        things_container: taffy::NodeId,
        processes_container: taffy::NodeId,
    ) {
        node_hierarchy
            .iter()
            .for_each(|(node_id, child_hierarchy)| {
                let node_id: &disposition_model_common::Id = node_id.as_ref();
                let entity_type = entity_types
                    .get(node_id)
                    .map(|entity_types| entity_types.first())
                    .flatten()
                    .unwrap_or_else(|| panic!("`entity_type` not found for {node_id}"));

                let taffy_parent_id = match entity_type {
                    EntityType::ThingDefault => things_container,
                    EntityType::TagDefault => tags_container,
                    EntityType::ProcessDefault => processes_container,
                    EntityType::ContainerInbuilt
                    | EntityType::ProcessStepDefault
                    | EntityType::DependencyEdgeSequenceDefault
                    | EntityType::DependencyEdgeCyclicDefault
                    | EntityType::DependencyEdgeSymmetricDefault
                    | EntityType::DependencyEdgeSequenceForwardDefault
                    | EntityType::DependencyEdgeCyclicForwardDefault
                    | EntityType::DependencyEdgeSymmetricForwardDefault
                    | EntityType::DependencyEdgeSymmetricReverseDefault
                    | EntityType::InteractionEdgeSequenceDefault
                    | EntityType::InteractionEdgeCyclicDefault
                    | EntityType::InteractionEdgeSymmetricDefault
                    | EntityType::InteractionEdgeSequenceForwardDefault
                    | EntityType::InteractionEdgeCyclicForwardDefault
                    | EntityType::InteractionEdgeSymmetricForwardDefault
                    | EntityType::InteractionEdgeSymmetricReverseDefault
                    | EntityType::Custom(_) => {
                        unreachable!("First entity type for {node_id} cannot be {entity_type}")
                    }
                };

                let taffy_style = Self::taffy_container_style(node_layouts, node_id, Size::auto());
                let taffy_node_id = taffy_tree
                    .new_leaf_with_context(
                        taffy_style,
                        NodeContext {
                            entity_id: node_id.clone(),
                            entity_type: entity_type.clone(),
                        },
                    )
                    .unwrap_or_else(|e| {
                        panic!("Expected to create a leaf node for {node_id}. Error: {e}")
                    });
                taffy_tree
                    .add_child(taffy_parent_id, taffy_node_id)
                    .expect("Failed to add child node");

                if !child_hierarchy.is_empty() {
                    Self::build_taffy_child_nodes_for_node(
                        taffy_tree,
                        node_layouts,
                        entity_types,
                        taffy_node_id,
                        child_hierarchy,
                    );
                }
            });
    }

    /// Adds the child taffy nodes for a given IR diagram node.
    fn build_taffy_child_nodes_for_node(
        taffy_tree: &mut TaffyTree<NodeContext>,
        node_layouts: &NodeLayouts,
        entity_types: &EntityTypes,
        taffy_parent_id: taffy::NodeId,
        node_hierarchy: &NodeHierarchy,
    ) {
        node_hierarchy
            .iter()
            .for_each(|(node_id, child_hierarchy)| {
                let node_id: &disposition_model_common::Id = node_id.as_ref();
                let entity_type = entity_types
                    .get(node_id)
                    .map(|entity_types| entity_types.first())
                    .flatten()
                    .unwrap_or_else(|| panic!("`entity_type` not found for {node_id}"));

                let taffy_style = Self::taffy_container_style(node_layouts, node_id, Size::auto());
                let taffy_node_id = taffy_tree
                    .new_leaf_with_context(
                        taffy_style,
                        NodeContext {
                            entity_id: node_id.clone(),
                            entity_type: entity_type.clone(),
                        },
                    )
                    .unwrap_or_else(|e| {
                        panic!("Expected to create a leaf node for {node_id}. Error: {e}")
                    });
                taffy_tree
                    .add_child(taffy_parent_id, taffy_node_id)
                    .expect("Failed to add child node");

                if !child_hierarchy.is_empty() {
                    Self::build_taffy_child_nodes_for_node(
                        taffy_tree,
                        node_layouts,
                        entity_types,
                        taffy_node_id,
                        child_hierarchy,
                    );
                }
            });
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
            Self::taffy_container_style(node_layouts, &node_inbuilt.id(), max_size);
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
        node_id: &disposition_model_common::Id,
        max_size: Size<taffy::Dimension>,
    ) -> Style {
        let style = node_layouts
            .get(node_id)
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
