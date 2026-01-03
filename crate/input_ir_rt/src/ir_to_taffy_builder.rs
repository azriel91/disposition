use std::borrow::Cow;

use disposition_ir_model::{
    entity::{EntityDescs, EntityType, EntityTypes},
    layout::{NodeLayout, NodeLayouts},
    node::{NodeHierarchy, NodeId, NodeInbuilt, NodeNames},
    IrDiagram,
};
use disposition_model_common::Map;
use disposition_taffy_model::{
    cosmic_text::{Align, Attrs, Buffer, FontSystem, Metrics, Shaping},
    taffy::{
        self,
        style::{FlexDirection, LengthPercentageAuto},
        AlignContent, AlignItems, AvailableSpace, Display, FlexWrap, LengthPercentage, Rect, Size,
        Style, TaffyTree,
    },
    DiagramLod, DimensionAndLod, IrToTaffyError, NodeContext, ProcessesIncluded, TaffyNodeMappings,
};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// Maps an intermediate representation diagram to a `TaffyNodeMappings`.
///
/// # Examples
///
/// ```rust
/// # use disposition_input_ir_rt::IrToTaffyBuilder;
/// # use disposition_ir_model::IrDiagram;
/// # use disposition_taffy_model::DimensionAndLod;
/// #
/// let ir_diagram = IrDiagram::new();
/// let dimension_and_lods = vec![DimensionAndLod::default_lg()];
///
/// let mut taffy_trees = IrToTaffyBuilder::builder()
///     .with_ir_diagram(ir_diagram)
///     .with_dimension_and_lods(dimension_and_lods)
///     .build();
/// ```
#[derive(Clone, Debug, Deserialize, Serialize, TypedBuilder)]
pub struct IrToTaffyBuilder {
    /// The intermediate representation of the diagram to render the taffy trees
    /// for.
    #[builder(setter(prefix = "with_"))]
    ir_diagram: IrDiagram,
    /// The dimensions at which elements should be repositioned.
    #[builder(setter(prefix = "with_"), default = vec![
        DimensionAndLod::default_sm(),
        DimensionAndLod::default_md(),
        DimensionAndLod::default_lg(),
    ])]
    dimension_and_lods: Vec<DimensionAndLod>,
    /// What processes to create diagrams for.
    #[builder(setter(prefix = "with_"), default = ProcessesIncluded::All)]
    processes_included: ProcessesIncluded,
}

impl IrToTaffyBuilder {
    /// Returns an iterator over `TaffyNodeMappings` instances for each
    /// dimension.
    pub fn build(&self) -> Result<impl Iterator<Item = TaffyNodeMappings>, IrToTaffyError> {
        let IrToTaffyBuilder {
            ir_diagram,
            dimension_and_lods,
            processes_included,
        } = self;

        let taffy_node_mappings_iter =
            dimension_and_lods
                .iter()
                .flat_map(move |dimension_and_lod| {
                    Self::build_taffy_trees_for_dimension(
                        ir_diagram,
                        dimension_and_lod,
                        processes_included,
                    )
                });

        Ok(taffy_node_mappings_iter)
    }

    /// Returns a `TaffyNodeMappings` with all processes as part of the diagram.
    ///
    /// This includes the processes container. Clicking on each process node
    /// reveals the process steps.
    fn build_taffy_trees_for_dimension(
        ir_diagram: &IrDiagram,
        dimension_and_lod: &DimensionAndLod,
        processes_included: &ProcessesIncluded,
    ) -> impl Iterator<Item = TaffyNodeMappings> {
        let IrDiagram {
            nodes,
            node_copy_text: _,
            node_hierarchy,
            edge_groups: _,
            entity_descs,
            entity_types,
            tailwind_classes: _,
            node_layouts,
            css: _,
        } = ir_diagram;

        // In theory this could be shared for all diagram generation, but it isn't
        // straightforward to satisfy Rust's constraints when returning an iterator.
        let mut cosmic_text_context = CosmicTextContext::new();
        let cosmic_text_context = &mut cosmic_text_context;

        let DimensionAndLod { dimension, lod } = dimension_and_lod;

        let mut taffy_tree = TaffyTree::new();
        let mut node_id_to_taffy = Map::new();

        let taffy_node_build_context = TaffyNodeBuildContext {
            taffy_tree: &mut taffy_tree,
            nodes,
            node_layouts,
            node_hierarchy,
            entity_types,
            node_id_to_taffy: &mut node_id_to_taffy,
        };
        let first_level_taffy_nodes = Self::build_taffy_nodes_for_first_level_nodes(
            taffy_node_build_context,
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
        let node_inbuilt_to_taffy = Self::build_taffy_container_nodes(
            &mut taffy_tree,
            node_layouts,
            dimension,
            thing_taffy_node_ids,
            process_taffy_node_ids,
            tag_taffy_node_ids,
        );

        let Some(root) = node_inbuilt_to_taffy.get(&NodeInbuilt::Root).copied() else {
            panic!("`root` node not present in `node_inbuilt_to_taffy`.");
        };

        taffy_tree
            .compute_layout_with_measure(
                root,
                Size::<AvailableSpace> {
                    width: AvailableSpace::Definite(dimension.width()),
                    height: AvailableSpace::Definite(dimension.height()),
                },
                |known_dimensions, available_space, _taffy_node_id, node_context, _style| {
                    Self::node_size_measure(
                        nodes,
                        entity_descs,
                        cosmic_text_context,
                        lod,
                        known_dimensions,
                        available_space,
                        node_context,
                    )
                },
            )
            .expect("Expected layout computation to succeed.");

        std::iter::once(TaffyNodeMappings {
            taffy_tree,
            node_inbuilt_to_taffy,
            node_id_to_taffy,
        })
    }

    /// Adds the inbuilt container nodes to the `TaffyTree`.
    fn build_taffy_container_nodes(
        taffy_tree: &mut TaffyTree<NodeContext>,
        node_layouts: &NodeLayouts,
        dimension: &disposition_taffy_model::Dimension,
        thing_taffy_node_ids: &[taffy::NodeId],
        process_taffy_node_ids: &[taffy::NodeId],
        tag_taffy_node_ids: &[taffy::NodeId],
    ) -> Map<NodeInbuilt, taffy::NodeId> {
        let things_container = Self::taffy_container_node(
            taffy_tree,
            node_layouts,
            NodeInbuilt::ThingsContainer,
            Size::auto(),
            thing_taffy_node_ids,
        );
        let processes_container = Self::taffy_container_node(
            taffy_tree,
            node_layouts,
            NodeInbuilt::ProcessesContainer,
            Size::auto(),
            process_taffy_node_ids,
        );
        let things_and_processes_container = Self::taffy_container_node(
            taffy_tree,
            node_layouts,
            NodeInbuilt::ThingsAndProcessesContainer,
            Size::auto(),
            &[processes_container, things_container],
        );
        let tags_container = Self::taffy_container_node(
            taffy_tree,
            node_layouts,
            NodeInbuilt::TagsContainer,
            Size::auto(),
            tag_taffy_node_ids,
        );

        let root = Self::taffy_container_node(
            taffy_tree,
            node_layouts,
            NodeInbuilt::Root,
            Size::from_lengths(dimension.width(), dimension.height()),
            &[tags_container, things_and_processes_container],
        );

        let mut node_inbuilt_to_taffy = Map::new();
        node_inbuilt_to_taffy.insert(NodeInbuilt::ThingsContainer, things_container);
        node_inbuilt_to_taffy.insert(NodeInbuilt::ProcessesContainer, processes_container);
        node_inbuilt_to_taffy.insert(
            NodeInbuilt::ThingsAndProcessesContainer,
            things_and_processes_container,
        );
        node_inbuilt_to_taffy.insert(NodeInbuilt::TagsContainer, tags_container);
        node_inbuilt_to_taffy.insert(NodeInbuilt::Root, root);

        node_inbuilt_to_taffy
    }

    /// Adds the tags, things, and process nodes to the taffy tree.
    ///
    /// This is different from `build_taffy_nodes_for_node` in that the parent
    /// node is one of the container nodes.
    fn build_taffy_nodes_for_first_level_nodes(
        taffy_node_build_context: TaffyNodeBuildContext<'_>,
        processes_included: &ProcessesIncluded,
    ) -> Map<EntityType, Vec<taffy::NodeId>> {
        let TaffyNodeBuildContext {
            nodes,
            taffy_tree,
            node_layouts,
            node_hierarchy,
            entity_types,
            node_id_to_taffy,
        } = taffy_node_build_context;

        node_hierarchy.iter().fold(
            Map::<EntityType, Vec<taffy::NodeId>>::new(),
            |mut entity_type_to_nodes, (node_id, child_hierarchy)| {
                let node_id: &disposition_model_common::Id = node_id.as_ref();
                let entity_type = entity_types
                    .get(node_id)
                    .and_then(|entity_types| entity_types.first())
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

                let taffy_node_id = if child_hierarchy.is_empty() {
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
                    let taffy_node_build_context = TaffyNodeBuildContext {
                        nodes,
                        taffy_tree,
                        node_layouts,
                        node_hierarchy: child_hierarchy,
                        entity_types,
                        node_id_to_taffy,
                    };
                    let taffy_children_ids =
                        Self::build_taffy_child_nodes_for_node(taffy_node_build_context);
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

                node_id_to_taffy.insert(NodeId::from(node_id.clone()), taffy_node_id);

                entity_type_to_nodes
            },
        )
    }

    /// Adds the child taffy nodes for a given IR diagram node.
    fn build_taffy_child_nodes_for_node(
        taffy_node_build_context: TaffyNodeBuildContext<'_>,
    ) -> Vec<taffy::NodeId> {
        let TaffyNodeBuildContext {
            nodes,
            taffy_tree,
            node_layouts,
            node_hierarchy,
            entity_types,
            node_id_to_taffy,
        } = taffy_node_build_context;

        node_hierarchy
            .iter()
            .map(|(node_id, child_hierarchy)| {
                let node_id: &disposition_model_common::Id = node_id.as_ref();
                let entity_type = entity_types
                    .get(node_id)
                    .and_then(|entity_types| entity_types.first())
                    .unwrap_or_else(|| panic!("`entity_type` not found for {node_id}"));

                let taffy_node_id = if child_hierarchy.is_empty() {
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
                    let taffy_node_build_context = TaffyNodeBuildContext {
                        nodes,
                        taffy_tree,
                        node_layouts,
                        node_hierarchy: child_hierarchy,
                        entity_types,
                        node_id_to_taffy,
                    };
                    let taffy_children_ids =
                        Self::build_taffy_child_nodes_for_node(taffy_node_build_context);
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

                node_id_to_taffy.insert(NodeId::from(node_id.clone()), taffy_node_id);

                taffy_node_id
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
        node_layouts
            .get(node_id)
            .map(|node_layout| match node_layout {
                NodeLayout::Flex(flex_layout) => Style {
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
                },
                NodeLayout::None => Style::default(),
            })
            .unwrap_or_default()
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

    /// Returns the size of a node based on its layout and available space.
    fn node_size_measure(
        nodes: &NodeNames,
        entity_descs: &EntityDescs,
        cosmic_text_context: &mut CosmicTextContext<'_>,
        lod: &DiagramLod,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        node_context: Option<&mut NodeContext>,
    ) -> Size<f32> {
        if let Size {
            width: Some(width),
            height: Some(height),
        } = known_dimensions
        {
            return Size { width, height };
        }

        let CosmicTextContext {
            font_system,
            buffer,
            font_attrs,
        } = cosmic_text_context;
        let text = node_context
            .as_ref()
            .map(|node_context| {
                let entity_id = &node_context.entity_id;
                let node_name = nodes
                    .get(entity_id)
                    .map(String::as_str)
                    .unwrap_or_else(|| entity_id.as_str());

                match lod {
                    DiagramLod::Simple => Cow::Borrowed(node_name),
                    DiagramLod::Normal => {
                        let node_desc = entity_descs.get(entity_id).map(String::as_str);

                        match node_desc {
                            Some(desc) => Cow::Owned(format!("{node_name}\n\n{desc}")),
                            None => Cow::Borrowed(node_name),
                        }
                    }
                }
            })
            .unwrap_or(Cow::Borrowed(""));
        buffer.set_text(
            font_system,
            &text,
            font_attrs,
            Shaping::Advanced,
            Some(Align::Left),
        );

        // Set width constraint
        let width_constraint = known_dimensions.width.or(match available_space.width {
            AvailableSpace::MinContent => Some(0.0),
            AvailableSpace::MaxContent => None,
            AvailableSpace::Definite(width) => Some(width),
        });
        buffer.set_size(font_system, width_constraint, None);

        // Compute layout
        buffer.shape_until_scroll(font_system, false);

        // Determine measured size of text
        let (width, line_count) = buffer.layout_runs().fold(
            (0.0, 1.0f32),
            |(line_width_max_so_far, line_count), layout_run| {
                let line_width_max = layout_run.line_w.max(line_width_max_so_far);
                (line_width_max, line_count + 1.0)
            },
        );
        let height = line_count * buffer.metrics().line_height;

        taffy::Size { width, height }
    }
}

struct TaffyNodeBuildContext<'ctx> {
    taffy_tree: &'ctx mut TaffyTree<NodeContext>,
    nodes: &'ctx NodeNames,
    node_layouts: &'ctx NodeLayouts,
    node_hierarchy: &'ctx NodeHierarchy,
    entity_types: &'ctx EntityTypes,
    node_id_to_taffy: &'ctx mut Map<NodeId, taffy::NodeId>,
}

#[derive(Debug)]
struct CosmicTextContext<'ctx> {
    font_system: FontSystem,
    buffer: Buffer,
    font_attrs: Attrs<'ctx>,
}

impl CosmicTextContext<'_> {
    fn new() -> Self {
        let mut font_system = FontSystem::new();
        let font_attrs = Attrs::new();
        let font_metrics = Metrics {
            font_size: 11.0f32,
            line_height: 13.0f32,
        };
        let mut buffer = Buffer::new_empty(font_metrics);
        buffer.set_size(&mut font_system, None, None);

        Self {
            font_system,
            buffer,
            font_attrs,
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
