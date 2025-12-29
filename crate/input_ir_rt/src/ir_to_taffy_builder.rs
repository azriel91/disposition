use disposition_ir_model::{
    entity::{EntityType, EntityTypes},
    layout::{NodeLayout, NodeLayouts},
    node::{NodeHierarchy, NodeInbuilt},
    IrDiagram,
};
use disposition_model_common::Map;
use disposition_taffy_model::{
    cosmic_text::FontSystem,
    taffy::{
        self,
        style::{FlexDirection, LengthPercentageAuto},
        AlignContent, AlignItems, AvailableSpace, Display, FlexWrap, LengthPercentage, Rect, Size,
        Style, TaffyTree,
    },
    CosmicTextContext, DimensionAndLod, IrToTaffyError, NodeContext, ProcessesIncluded,
    TaffyTreeAndRoot,
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

        let first_level_taffy_nodes = Self::build_taffy_nodes_for_first_level_nodes(
            &mut taffy_tree,
            node_layouts,
            entity_types,
            node_hierarchy,
            processes_included,
        );
        let thing_taffy_node_ids = first_level_taffy_nodes
            .get(&EntityType::ThingDefault)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let tag_taffy_node_ids = first_level_taffy_nodes
            .get(&EntityType::TagDefault)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let process_taffy_node_ids = first_level_taffy_nodes
            .get(&EntityType::ProcessDefault)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let root = Self::build_taffy_container_nodes(
            &mut taffy_tree,
            node_layouts,
            dimension,
            thing_taffy_node_ids,
            process_taffy_node_ids,
            tag_taffy_node_ids,
        );

        let mut font_system = FontSystem::new();
        taffy_tree
            .compute_layout_with_measure(
                root,
                Size::<AvailableSpace> {
                    width: AvailableSpace::Definite(dimension.width()),
                    height: AvailableSpace::Definite(dimension.height()),
                },
                // Note: this closure is a FnMut closure and can be used to borrow external context
                // for the duration of layout For example, you may wish to borrow a
                // global font registry and pass it into your text measuring
                // function
                |known_dimensions, available_space, _node_id, node_context, _style| {
                    Self::measure_function(
                        known_dimensions,
                        available_space,
                        node_context,
                        &mut font_system,
                    )
                },
            )
            .expect("Expected layout computation to succeed.");

        std::iter::once(TaffyTreeAndRoot { taffy_tree, root })
    }

    /// Adds the inbuilt container nodes to the `TaffyTree`.
    fn build_taffy_container_nodes(
        taffy_tree: &mut TaffyTree<NodeContext>,
        node_layouts: &NodeLayouts,
        dimension: &disposition_taffy_model::Dimension,
        thing_taffy_node_ids: &[taffy::NodeId],
        process_taffy_node_ids: &[taffy::NodeId],
        tag_taffy_node_ids: &[taffy::NodeId],
    ) -> taffy::NodeId {
        let things_container = Self::taffy_container_node(
            taffy_tree,
            &node_layouts,
            NodeInbuilt::ThingsContainer,
            Size::auto(),
            thing_taffy_node_ids,
        );
        let processes_container = Self::taffy_container_node(
            taffy_tree,
            &node_layouts,
            NodeInbuilt::ProcessesContainer,
            Size::auto(),
            process_taffy_node_ids,
        );
        let things_and_processes_container = Self::taffy_container_node(
            taffy_tree,
            &node_layouts,
            NodeInbuilt::ThingsAndProcessesContainer,
            Size::auto(),
            &[processes_container, things_container],
        );
        let tags_container = Self::taffy_container_node(
            taffy_tree,
            &node_layouts,
            NodeInbuilt::TagsContainer,
            Size::auto(),
            tag_taffy_node_ids,
        );
        let root = Self::taffy_container_node(
            taffy_tree,
            &node_layouts,
            NodeInbuilt::Root,
            Size::from_lengths(dimension.width(), dimension.height()),
            &[tags_container, things_and_processes_container],
        );
        root
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
        processes_included: &ProcessesIncluded,
    ) -> Map<EntityType, Vec<taffy::NodeId>> {
        node_hierarchy.iter().fold(
            Map::<EntityType, Vec<taffy::NodeId>>::new(),
            |mut entity_type_to_nodes, (node_id, child_hierarchy)| {
                let node_id: &disposition_model_common::Id = node_id.as_ref();
                let entity_type = entity_types
                    .get(node_id)
                    .map(|entity_types| entity_types.first())
                    .flatten()
                    .unwrap_or_else(|| panic!("`entity_type` not found for {node_id}"));

                if matches!(entity_type, EntityType::ProcessDefault) {
                    match processes_included {
                        ProcessesIncluded::All => {}
                        ProcessesIncluded::Filter { process_ids } => {
                            if process_ids.contains(node_id) {
                                // Don't add this process.
                                return entity_type_to_nodes;
                            }
                        }
                    };
                }

                let cosmic_text_context = CosmicTextContext::new();

                let taffy_node_id = if child_hierarchy.is_empty() {
                    let taffy_style =
                        Self::taffy_container_style(node_layouts, node_id, Size::auto());
                    taffy_tree
                        .new_leaf_with_context(
                            taffy_style,
                            NodeContext {
                                entity_id: node_id.clone(),
                                entity_type: entity_type.clone(),
                                cosmic_text_context,
                            },
                        )
                        .unwrap_or_else(|e| {
                            panic!("Expected to create text leaf node for {node_id}. Error: {e}")
                        })
                } else {
                    let TaffyWrapperNodeStyles {
                        wrapper_style,
                        text_style,
                        child_container_style,
                    } = Self::taffy_wrapper_node_styles(node_layouts, node_id);
                    let taffy_text_node_id = taffy_tree
                        .new_leaf_with_context(
                            text_style,
                            NodeContext {
                                entity_id: node_id.clone(),
                                entity_type: entity_type.clone(),
                            },
                        )
                        .unwrap_or_else(|e| {
                            panic!("Expected to create text leaf node for {node_id}. Error: {e}")
                        });
                    let taffy_children_ids = Self::build_taffy_child_nodes_for_node(
                        taffy_tree,
                        node_layouts,
                        entity_types,
                        child_hierarchy,
                    );
                    let taffy_children_container_id = taffy_tree
                        .new_with_children(child_container_style, &taffy_children_ids)
                        .unwrap_or_else(|e| {
                            panic!("Expected to create text leaf node for {node_id}. Error: {e}")
                        });
                    taffy_tree
                        .new_with_children(
                            wrapper_style,
                            &[taffy_text_node_id, taffy_children_container_id],
                        )
                        .unwrap_or_else(|e| {
                            panic!("Expected to create wrapper node for {node_id}. Error: {e}")
                        })
                };

                entity_type_to_nodes
                    .entry(entity_type.clone())
                    .or_default()
                    .push(taffy_node_id);

                entity_type_to_nodes
            },
        )
    }

    /// Adds the child taffy nodes for a given IR diagram node.
    fn build_taffy_child_nodes_for_node(
        taffy_tree: &mut TaffyTree<NodeContext>,
        node_layouts: &NodeLayouts,
        entity_types: &EntityTypes,
        node_hierarchy: &NodeHierarchy,
    ) -> Vec<taffy::NodeId> {
        node_hierarchy
            .iter()
            .map(|(node_id, child_hierarchy)| {
                let node_id: &disposition_model_common::Id = node_id.as_ref();
                let entity_type = entity_types
                    .get(node_id)
                    .map(|entity_types| entity_types.first())
                    .flatten()
                    .unwrap_or_else(|| panic!("`entity_type` not found for {node_id}"));

                if child_hierarchy.is_empty() {
                    let taffy_style =
                        Self::taffy_container_style(node_layouts, node_id, Size::auto());
                    taffy_tree
                        .new_leaf_with_context(
                            taffy_style,
                            NodeContext {
                                entity_id: node_id.clone(),
                                entity_type: entity_type.clone(),
                            },
                        )
                        .unwrap_or_else(|e| {
                            panic!("Expected to create text leaf node for {node_id}. Error: {e}")
                        })
                } else {
                    let TaffyWrapperNodeStyles {
                        wrapper_style,
                        text_style,
                        child_container_style,
                    } = Self::taffy_wrapper_node_styles(node_layouts, node_id);
                    let taffy_text_node_id = taffy_tree
                        .new_leaf_with_context(
                            text_style,
                            NodeContext {
                                entity_id: node_id.clone(),
                                entity_type: entity_type.clone(),
                            },
                        )
                        .unwrap_or_else(|e| {
                            panic!("Expected to create text leaf node for {node_id}. Error: {e}")
                        });
                    let taffy_children_ids = Self::build_taffy_child_nodes_for_node(
                        taffy_tree,
                        node_layouts,
                        entity_types,
                        child_hierarchy,
                    );
                    let taffy_children_container_id = taffy_tree
                        .new_with_children(child_container_style, &taffy_children_ids)
                        .unwrap_or_else(|e| {
                            panic!("Expected to create text leaf node for {node_id}. Error: {e}")
                        });
                    taffy_tree
                        .new_with_children(
                            wrapper_style,
                            &[taffy_text_node_id, taffy_children_container_id],
                        )
                        .unwrap_or_else(|e| {
                            panic!("Expected to create wrapper node for {node_id}. Error: {e}")
                        })
                }
            })
            .collect::<Vec<taffy::NodeId>>()
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
    /// * `child_node_ids`: IDs of child nodes to add to the container.
    fn taffy_container_node(
        taffy_tree: &mut TaffyTree<NodeContext>,
        node_layouts: &NodeLayouts,
        node_inbuilt: NodeInbuilt,
        max_size: Size<taffy::Dimension>,
        child_node_ids: &[taffy::NodeId],
    ) -> taffy::NodeId {
        let tags_container_style =
            Self::taffy_container_style(node_layouts, &node_inbuilt.id(), max_size);
        taffy_tree
            .new_with_children(tags_container_style, child_node_ids)
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

    /// Returns the `taffy::Style` for a wrapper node and its text node.
    fn taffy_wrapper_node_styles(
        node_layouts: &NodeLayouts,
        node_id: &disposition_model_common::Id,
    ) -> TaffyWrapperNodeStyles {
        node_layouts
            .get(node_id)
            .map(|node_layout| match node_layout {
                NodeLayout::Flex(flex_layout) => {
                    let wrapper_style = Style {
                        display: Display::Flex,
                        max_size: Size::auto(),
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
                        align_items: Some(AlignItems::FlexStart),
                        justify_items: Some(AlignItems::FlexStart),
                        align_content: Some(AlignContent::Center),
                        justify_content: Some(AlignContent::Center),
                        gap: Size::length(flex_layout.gap()),
                        flex_direction: FlexDirection::Column,
                        flex_wrap: FlexWrap::NoWrap,
                        ..Default::default()
                    };
                    // Leaf node doesn't need much difference from wrapper style
                    let text_style = Style::default();
                    let child_container_style = Style {
                        display: Display::Flex,
                        max_size: Size::auto(),
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

                    TaffyWrapperNodeStyles {
                        wrapper_style,
                        text_style,
                        child_container_style,
                    }
                }
                NodeLayout::None => TaffyWrapperNodeStyles::default(),
            })
            .unwrap_or_default()
    }

    fn measure_function(
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        node_context: Option<&mut NodeContext>,
        font_system: &mut FontSystem,
    ) -> Size<f32> {
        if let Size {
            width: Some(width),
            height: Some(height),
        } = known_dimensions
        {
            return Size { width, height };
        }

        match node_context {
            None => Size::zero(),
            Some(NodeContext {
                entity_id,
                entity_type,
                cosmic_text_context,
            }) => cosmic_text_context.measure(known_dimensions, available_space, font_system),
        }
    }
}

/// Layout information for a wrapper node and its text node.
struct TaffyWrapperNodeStyles {
    wrapper_style: Style,
    text_style: Style,
    child_container_style: Style,
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
                align_items: Some(AlignItems::Center),
                justify_items: Some(AlignItems::Center),
                align_content: Some(AlignContent::Center),
                justify_content: Some(AlignContent::Center),
                ..Default::default()
            },
        }
    }
}
