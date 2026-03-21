use std::{borrow::Cow, collections::BTreeMap};

use disposition_ir_model::{
    entity::{EntityDescs, EntityType, EntityTypes},
    layout::{FlexDirection as ModelFlexDirection, NodeLayout, NodeLayouts},
    node::{
        NodeHierarchy, NodeId, NodeInbuilt, NodeNames, NodeRank, NodeRanks, NodeShape, NodeShapes,
    },
    IrDiagram,
};
use disposition_model_common::{Id, Map};
use disposition_taffy_model::{
    taffy::{
        self,
        style::{FlexDirection, LengthPercentageAuto},
        AlignContent, AlignItems, AvailableSpace, Display, FlexWrap, LengthPercentage, Rect, Size,
        Style, TaffyTree,
    },
    DiagramLod, Dimension, DimensionAndLod, EntityHighlightedSpan, EntityHighlightedSpans,
    IrToTaffyError, NodeContext, NodeToTaffyNodeIds, ProcessesIncluded, TaffyNodeMappings,
    TEXT_FONT_SIZE, TEXT_LINE_HEIGHT,
};
use taffy::prelude::TaffyZero;
use typed_builder::TypedBuilder;

use self::{
    taffy_node_build_context::{NodeMeasureContext, TaffyNodeBuildContext, TaffyWrapperNodeStyles},
    text_measure::{
        compute_text_dimensions, line_width_measure, wrap_text_monospace,
        MONOSPACE_CHAR_WIDTH_RATIO,
    },
};

mod taffy_node_build_context;
mod text_measure;

/// Converts a model [`FlexDirection`](ModelFlexDirection) to a
/// [`taffy::style::FlexDirection`].
fn flex_direction_to_taffy(direction: ModelFlexDirection) -> FlexDirection {
    match direction {
        ModelFlexDirection::Row => FlexDirection::Row,
        ModelFlexDirection::RowReverse => FlexDirection::RowReverse,
        ModelFlexDirection::Column => FlexDirection::Column,
        ModelFlexDirection::ColumnReverse => FlexDirection::ColumnReverse,
    }
}

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
///     .with_ir_diagram(&ir_diagram)
///     .with_dimension_and_lods(dimension_and_lods)
///     .build();
/// ```
#[derive(Debug, TypedBuilder)]
pub struct IrToTaffyBuilder<'builder> {
    /// The intermediate representation of the diagram to render the taffy trees
    /// for.
    #[builder(setter(prefix = "with_"))]
    ir_diagram: &'builder IrDiagram<'static>,
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

impl IrToTaffyBuilder<'_> {
    /// Returns an iterator over `TaffyNodeMappings` instances for each
    /// dimension.
    pub fn build(
        &self,
    ) -> Result<impl Iterator<Item = TaffyNodeMappings<'static>>, IrToTaffyError> {
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
        ir_diagram: &IrDiagram<'static>,
        dimension_and_lod: &DimensionAndLod,
        processes_included: &ProcessesIncluded,
    ) -> impl Iterator<Item = TaffyNodeMappings<'static>> {
        let IrDiagram {
            nodes,
            node_copy_text: _,
            node_hierarchy,
            node_ordering: _,
            edge_groups: _,
            entity_descs,
            entity_tooltips: _,
            entity_types,
            tailwind_classes: _,
            node_layouts,
            node_ranks,
            node_shapes,
            process_step_entities: _,
            css: _,
        } = ir_diagram;

        let DimensionAndLod { dimension, lod } = dimension_and_lod;

        let mut taffy_tree = TaffyTree::new();
        let mut node_id_to_taffy = Map::new();
        let mut taffy_id_to_node = Map::new();

        let taffy_node_build_context = TaffyNodeBuildContext {
            taffy_tree: &mut taffy_tree,
            nodes,
            node_layouts,
            node_hierarchy,
            entity_types,
            node_shapes,
            node_ranks,
            node_id_to_taffy: &mut node_id_to_taffy,
            taffy_id_to_node: &mut taffy_id_to_node,
        };
        let first_level_taffy_nodes = Self::build_taffy_nodes_for_first_level_nodes(
            taffy_node_build_context,
            processes_included,
        );
        let thing_rank_to_taffy_ids = first_level_taffy_nodes
            .get(&EntityType::ThingDefault)
            .cloned()
            .unwrap_or_default();
        let tag_taffy_node_ids: Vec<taffy::NodeId> = first_level_taffy_nodes
            .get(&EntityType::TagDefault)
            .into_iter()
            .flat_map(|rank_map| rank_map.values().flatten().copied())
            .collect();
        let process_taffy_node_ids: Vec<taffy::NodeId> = first_level_taffy_nodes
            .get(&EntityType::ProcessDefault)
            .into_iter()
            .flat_map(|rank_map| rank_map.values().flatten().copied())
            .collect();

        // Create rank sub-containers for top-level thing nodes, mirroring the
        // rank-based child container logic used inside
        // `build_taffy_nodes_for_node_with_child_hierarchy`.
        let things_container_style = Self::taffy_container_style(
            node_layouts,
            &NodeInbuilt::ThingsContainer.id(),
            Size::auto(),
        );
        let thing_rank_container_ids: Vec<taffy::NodeId> = thing_rank_to_taffy_ids
            .into_iter()
            .map(|(_rank, taffy_ids)| {
                taffy_tree
                    .new_with_children(things_container_style.clone(), &taffy_ids)
                    .expect("Expected to create rank container node for top-level things.")
            })
            .collect();

        let node_inbuilt_to_taffy = Self::build_taffy_container_nodes(
            &mut taffy_tree,
            &mut taffy_id_to_node,
            node_layouts,
            dimension,
            &thing_rank_container_ids,
            &process_taffy_node_ids,
            &tag_taffy_node_ids,
        );

        let Some(root) = node_inbuilt_to_taffy.get(&NodeInbuilt::Root).copied() else {
            panic!("`root` node not present in `node_inbuilt_to_taffy`.");
        };

        // Precompute monospace character width
        let char_width = TEXT_FONT_SIZE * MONOSPACE_CHAR_WIDTH_RATIO;

        // Compute layout (size measurement only, no syntax highlighting)
        let mut node_measure_context = NodeMeasureContext {
            nodes,
            entity_descs,
            char_width,
            lod,
        };

        taffy_tree
            .compute_layout_with_measure(
                root,
                Size::<AvailableSpace> {
                    width: AvailableSpace::Definite(dimension.width()),
                    height: AvailableSpace::Definite(dimension.height()),
                },
                |known_dimensions, available_space, _taffy_node_id, node_context, style| {
                    Self::node_size_measure(
                        &mut node_measure_context,
                        known_dimensions,
                        available_space,
                        node_context,
                        style,
                    )
                },
            )
            .expect("Expected layout computation to succeed.");

        // Compute highlighted spans *after* layout is complete.
        //
        // This is done once per node instead of multiple times during layout
        // measurement
        let entity_highlighted_spans = Self::highlighted_spans_compute(
            &taffy_tree,
            &node_id_to_taffy,
            nodes,
            entity_descs,
            char_width,
            lod,
        );

        std::iter::once(TaffyNodeMappings {
            taffy_tree,
            node_inbuilt_to_taffy,
            node_id_to_taffy,
            taffy_id_to_node,
            entity_highlighted_spans,
        })
    }

    /// Compute highlighted spans for all nodes after layout is complete.
    /// This is much more efficient than doing it during measure() which gets
    /// called multiple times.
    fn highlighted_spans_compute(
        taffy_tree: &TaffyTree<NodeContext>,
        node_id_to_taffy: &Map<NodeId<'static>, NodeToTaffyNodeIds>,
        nodes: &NodeNames<'static>,
        entity_descs: &EntityDescs<'static>,
        char_width: f32,
        lod: &DiagramLod,
    ) -> EntityHighlightedSpans<'static> {
        let mut entity_highlighted_spans =
            EntityHighlightedSpans::with_capacity(node_id_to_taffy.len());

        let line_height = TEXT_LINE_HEIGHT;

        node_id_to_taffy
            .iter()
            .for_each(|(node_id, &taffy_node_ids)| {
                let (wrapper_node_layout, text_node_layout, node_context) = match taffy_node_ids {
                    NodeToTaffyNodeIds::Leaf { text_node_id } => {
                        let Ok(text_node_layout) = taffy_tree.layout(text_node_id) else {
                            return;
                        };
                        let Some(node_context) = taffy_tree.get_node_context(text_node_id) else {
                            return;
                        };
                        (text_node_layout, text_node_layout, node_context)
                    }
                    NodeToTaffyNodeIds::Wrapper {
                        wrapper_node_id,
                        text_node_id,
                    }
                    | NodeToTaffyNodeIds::LeafWithCircle {
                        wrapper_node_id,
                        circle_node_id: _,
                        text_node_id,
                    }
                    | NodeToTaffyNodeIds::WrapperCircle {
                        wrapper_node_id,
                        label_wrapper_node_id: _,
                        circle_node_id: _,
                        text_node_id,
                    } => {
                        let Ok(wrapper_node_layout) = taffy_tree.layout(wrapper_node_id) else {
                            return;
                        };
                        let Ok(text_node_layout) = taffy_tree.layout(text_node_id) else {
                            return;
                        };
                        let Some(node_context) = taffy_tree.get_node_context(text_node_id) else {
                            return;
                        };

                        (wrapper_node_layout, text_node_layout, node_context)
                    }
                };
                let text_label_offset = match taffy_node_ids {
                    NodeToTaffyNodeIds::Leaf { .. } | NodeToTaffyNodeIds::Wrapper { .. } => 0.0f32,
                    NodeToTaffyNodeIds::LeafWithCircle {
                        wrapper_node_id: _,
                        circle_node_id,
                        text_node_id: _,
                    }
                    | NodeToTaffyNodeIds::WrapperCircle {
                        wrapper_node_id: _,
                        label_wrapper_node_id: _,
                        circle_node_id,
                        text_node_id: _,
                    } => taffy_tree
                        .layout(circle_node_id)
                        .map(|circle_node_layout| {
                            // This could be:
                            //
                            // ```rust
                            // circle_node_layout.size.width + gap
                            // ```
                            //
                            // but we don't have the gap value
                            text_node_layout.location.x - circle_node_layout.location.x
                        })
                        .unwrap_or_default(),
                };

                let entity_id = &node_context.entity_id;

                // Build the text content
                let node_name = nodes
                    .get(entity_id)
                    .map(String::as_str)
                    .unwrap_or_else(|| entity_id.as_str());

                let text: Cow<'_, str> = match lod {
                    DiagramLod::Simple => Cow::Borrowed(node_name),
                    DiagramLod::Normal => {
                        let node_desc = entity_descs.get(entity_id).map(String::as_str);
                        match node_desc {
                            Some(desc) => Cow::Owned(format!("# {node_name}\n\n{desc}")),
                            None => Cow::Borrowed(node_name),
                        }
                    }
                };

                if text.is_empty() {
                    return;
                }

                // Use the computed layout width as constraint
                let max_width = text_node_layout.size.width;

                // Compute line wrapping using simple monospace calculation
                let wrapped_lines = wrap_text_monospace(&text, char_width, max_width);

                // Get style info for padding calculations
                let padding_left = text_node_layout.padding.left;
                let padding_top = wrapper_node_layout.padding.top;

                // Note: we shift the text by half a character width because even though we have
                // padding, the text still reaches the left and right edges of the node.
                //
                // The half a character width (at each end) is added to the node's width in
                // `line_width_measure`.
                let text_leftmost_x = text_label_offset + padding_left + 0.5 * char_width;

                let highlighted_spans: Vec<EntityHighlightedSpan> = {
                    wrapped_lines
                        .iter()
                        .enumerate()
                        .flat_map(|(line_index, line)| {
                            let x = text_leftmost_x;
                            let y = (line_index + 1) as f32 * line_height + padding_top;
                            let width = line_width_measure(line, char_width);

                            let entity_highlighted_span = EntityHighlightedSpan {
                                x,
                                y,
                                width,
                                height: line_height,
                                // style,
                                text: line.to_string(),
                            };

                            vec![entity_highlighted_span]
                        })
                        .collect()
                };

                entity_highlighted_spans.insert(node_id.as_ref().clone(), highlighted_spans);
            });

        entity_highlighted_spans
    }

    /// Adds the inbuilt container nodes to the `TaffyTree`.
    fn build_taffy_container_nodes(
        taffy_tree: &mut TaffyTree<NodeContext>,
        taffy_id_to_node: &mut Map<taffy::NodeId, NodeId>,
        node_layouts: &NodeLayouts,
        dimension: &disposition_taffy_model::Dimension,
        thing_rank_container_ids: &[taffy::NodeId],
        process_taffy_node_ids: &[taffy::NodeId],
        tag_taffy_node_ids: &[taffy::NodeId],
    ) -> Map<NodeInbuilt, taffy::NodeId> {
        // The things container's children are rank sub-containers (each of
        // which uses the `_things_container` row/wrap style internally).
        // The things container itself uses a column layout to stack rank
        // groups vertically.
        let things_container_base_style = Self::taffy_container_style(
            node_layouts,
            &NodeInbuilt::ThingsContainer.id(),
            Size::auto(),
        );
        let things_container_style = Style {
            flex_direction: FlexDirection::Column,
            flex_wrap: FlexWrap::NoWrap,
            ..things_container_base_style
        };
        let things_container = taffy_tree
            .new_with_children(things_container_style, thing_rank_container_ids)
            .expect("`TaffyTree::new_with_children` should be infallible.");
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
            match dimension {
                Dimension::NoLimit => Size::auto(),
                _ => Size::from_lengths(dimension.width(), dimension.height()),
            },
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

        taffy_id_to_node.insert(
            things_container,
            NodeId::from(NodeInbuilt::ThingsContainer.id()),
        );
        taffy_id_to_node.insert(
            processes_container,
            NodeId::from(NodeInbuilt::ProcessesContainer.id()),
        );
        taffy_id_to_node.insert(
            things_and_processes_container,
            NodeId::from(NodeInbuilt::ThingsAndProcessesContainer.id()),
        );
        taffy_id_to_node.insert(
            tags_container,
            NodeId::from(NodeInbuilt::TagsContainer.id()),
        );
        taffy_id_to_node.insert(root, NodeId::from(NodeInbuilt::Root.id()));

        node_inbuilt_to_taffy
    }

    /// Adds the tags, things, and process nodes to the taffy tree.
    ///
    /// This is different from `build_taffy_nodes_for_node` in that the parent
    /// node is one of the container nodes.
    ///
    /// Returns a map from `EntityType` to a `BTreeMap<NodeRank, Vec<NodeId>>`,
    /// so that callers can create rank-based sub-containers for each entity
    /// type (e.g. grouping top-level thing nodes by rank).
    fn build_taffy_nodes_for_first_level_nodes(
        taffy_node_build_context: TaffyNodeBuildContext<'_>,
        processes_included: &ProcessesIncluded,
    ) -> Map<EntityType, BTreeMap<NodeRank, Vec<taffy::NodeId>>> {
        let TaffyNodeBuildContext {
            nodes,
            taffy_tree,
            node_layouts,
            node_hierarchy,
            entity_types,
            node_shapes,
            node_ranks,
            node_id_to_taffy,
            taffy_id_to_node,
        } = taffy_node_build_context;

        node_hierarchy.iter().fold(
            Map::<EntityType, BTreeMap<NodeRank, Vec<taffy::NodeId>>>::new(),
            |mut entity_type_to_nodes, (node_id, child_hierarchy)| {
                let node_id: &Id = node_id.as_ref();
                let Some(entity_type) = entity_types
                    .get(node_id)
                    .and_then(|entity_types| entity_types.first())
                else {
                    // Skip nodes without an entity type -- probably something extra in the
                    // hierarchy without a node name.
                    return entity_type_to_nodes;
                };

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

                let wrapper_node_id = if child_hierarchy.is_empty() {
                    Self::build_taffy_nodes_for_node_without_child_hierarchy(
                        taffy_tree,
                        node_layouts,
                        node_shapes,
                        node_id_to_taffy,
                        taffy_id_to_node,
                        node_id,
                        entity_type,
                    )
                } else {
                    Self::build_taffy_nodes_for_node_with_child_hierarchy(
                        nodes,
                        taffy_tree,
                        node_layouts,
                        node_shapes,
                        entity_types,
                        node_ranks,
                        node_id_to_taffy,
                        taffy_id_to_node,
                        child_hierarchy,
                        node_id,
                        entity_type,
                    )
                };

                let ir_node_id = NodeId::from(node_id.clone());
                let rank = node_ranks
                    .get(&ir_node_id)
                    .copied()
                    .unwrap_or(NodeRank::new(0));

                entity_type_to_nodes
                    .entry(entity_type.clone())
                    .or_default()
                    .entry(rank)
                    .or_default()
                    .push(wrapper_node_id);

                entity_type_to_nodes
            },
        )
    }

    /// Adds the child taffy nodes for a given IR diagram node, grouped by rank.
    ///
    /// Returns a `BTreeMap` from `NodeRank` to the list of taffy node IDs at
    /// that rank. This allows the caller to create separate child containers
    /// for each rank level.
    fn build_taffy_child_nodes_for_node_by_rank(
        taffy_node_build_context: TaffyNodeBuildContext<'_>,
    ) -> BTreeMap<NodeRank, Vec<taffy::NodeId>> {
        let TaffyNodeBuildContext {
            nodes,
            taffy_tree,
            node_layouts,
            node_hierarchy,
            entity_types,
            node_shapes,
            node_ranks,
            node_id_to_taffy,
            taffy_id_to_node,
        } = taffy_node_build_context;

        let mut rank_to_taffy_ids: BTreeMap<NodeRank, Vec<taffy::NodeId>> = BTreeMap::new();

        for (node_id, child_hierarchy) in node_hierarchy.iter() {
            let node_id: &Id = node_id.as_ref();
            let Some(entity_type) = entity_types
                .get(node_id)
                .and_then(|entity_types| entity_types.first())
            else {
                // Skip nodes without an entity type -- probably something extra in the
                // hierarchy without a node name.
                continue;
            };

            let taffy_node_id = if child_hierarchy.is_empty() {
                Self::build_taffy_nodes_for_node_without_child_hierarchy(
                    taffy_tree,
                    node_layouts,
                    node_shapes,
                    node_id_to_taffy,
                    taffy_id_to_node,
                    node_id,
                    entity_type,
                )
            } else {
                Self::build_taffy_nodes_for_node_with_child_hierarchy(
                    nodes,
                    taffy_tree,
                    node_layouts,
                    node_shapes,
                    entity_types,
                    node_ranks,
                    node_id_to_taffy,
                    taffy_id_to_node,
                    child_hierarchy,
                    node_id,
                    entity_type,
                )
            };

            let ir_node_id = NodeId::from(node_id.clone());
            let rank = node_ranks
                .get(&ir_node_id)
                .copied()
                .unwrap_or(NodeRank::new(0));

            rank_to_taffy_ids
                .entry(rank)
                .or_default()
                .push(taffy_node_id);
        }

        rank_to_taffy_ids
    }

    fn build_taffy_nodes_for_node_without_child_hierarchy(
        taffy_tree: &mut TaffyTree<NodeContext>,
        node_layouts: &NodeLayouts<'static>,
        node_shapes: &NodeShapes<'static>,
        node_id_to_taffy: &mut Map<NodeId<'static>, NodeToTaffyNodeIds>,
        taffy_id_to_node: &mut Map<taffy::NodeId, NodeId<'static>>,
        node_id: &Id<'static>,
        entity_type: &EntityType,
    ) -> taffy::NodeId {
        let ir_node_id = NodeId::from(node_id.clone());
        let node_shape = node_shapes
            .get(&ir_node_id)
            .unwrap_or_else(|| panic!("There was no node shape for {ir_node_id}."));
        match node_shape {
            NodeShape::Rect(_node_shape_rect) => {
                let taffy_style = Self::taffy_container_style(node_layouts, node_id, Size::auto());
                let taffy_text_node_id = taffy_tree
                    .new_leaf_with_context(
                        taffy_style,
                        NodeContext {
                            entity_id: node_id.clone(),
                            entity_type: entity_type.clone(),
                        },
                    )
                    .unwrap_or_else(|e| {
                        panic!("Expected to create text leaf node for {node_id}. Error: {e}")
                    });

                node_id_to_taffy.insert(
                    ir_node_id.clone(),
                    NodeToTaffyNodeIds::Leaf {
                        text_node_id: taffy_text_node_id,
                    },
                );
                taffy_id_to_node.insert(taffy_text_node_id, ir_node_id);

                taffy_text_node_id
            }
            NodeShape::Circle(node_shape_circle) => {
                // Circle leaf:
                //
                // ```yaml
                // label_wrapper_node: # flex row
                //   - circle_node
                //   - text_node
                // ```
                let circle_radius = node_shape_circle.radius();
                let circle_diameter = circle_radius * 2.0;

                let circle_node_id = taffy_tree
                    .new_leaf(Style {
                        size: Size {
                            width: taffy::style::Dimension::length(circle_diameter),
                            height: taffy::style::Dimension::length(circle_diameter),
                        },
                        flex_shrink: 0.0,
                        ..Default::default()
                    })
                    .unwrap_or_else(|e| {
                        panic!("Expected to create circle leaf node for {node_id}. Error: {e}")
                    });

                let text_style = Style::default();
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

                let label_wrapper_style =
                    Self::taffy_container_style(node_layouts, node_id, Size::auto());

                // Override to flex row for circle + text side by side
                let label_wrapper_style = Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: Some(AlignItems::Center),
                    gap: Size::length(4.0),
                    ..label_wrapper_style
                };

                let wrapper_node_id = taffy_tree
                    .new_with_children(label_wrapper_style, &[circle_node_id, taffy_text_node_id])
                    .unwrap_or_else(|e| {
                        panic!("Expected to create label wrapper node for {node_id}. Error: {e}")
                    });

                node_id_to_taffy.insert(
                    ir_node_id.clone(),
                    NodeToTaffyNodeIds::LeafWithCircle {
                        wrapper_node_id,
                        circle_node_id,
                        text_node_id: taffy_text_node_id,
                    },
                );
                taffy_id_to_node.insert(wrapper_node_id, ir_node_id);

                wrapper_node_id
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn build_taffy_nodes_for_node_with_child_hierarchy(
        nodes: &NodeNames<'static>,
        taffy_tree: &mut TaffyTree<NodeContext>,
        node_layouts: &NodeLayouts<'static>,
        node_shapes: &NodeShapes<'static>,
        entity_types: &EntityTypes<'static>,
        node_ranks: &NodeRanks<'static>,
        node_id_to_taffy: &mut Map<NodeId<'static>, NodeToTaffyNodeIds>,
        taffy_id_to_node: &mut Map<taffy::NodeId, NodeId<'static>>,
        child_hierarchy: &NodeHierarchy<'static>,
        node_id: &Id<'static>,
        entity_type: &EntityType,
    ) -> taffy::NodeId {
        let ir_node_id = NodeId::from(node_id.clone());

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
            node_shapes,
            node_ranks,
            node_id_to_taffy,
            taffy_id_to_node,
        };
        let rank_to_taffy_ids =
            Self::build_taffy_child_nodes_for_node_by_rank(taffy_node_build_context);

        // === Build Rank-Based Child Containers === //
        //
        // Instead of a single child container with all children, we create one
        // child container per rank level. This causes higher-ranked nodes to be
        // positioned further along the wrapper's flex direction (down for
        // column, right for row).
        //
        // ```yaml
        // wrapper_node:
        //   text_node: 'node text'
        //   child_container_0: {} # nodes with rank n
        //   child_container_1: {} # nodes with rank n + 1
        //   child_container_2: {} # nodes with rank n + 2
        // ```
        let rank_container_ids: Vec<taffy::NodeId> = rank_to_taffy_ids
            .into_iter()
            .map(|(_rank, taffy_ids)| {
                taffy_tree
                    .new_with_children(child_container_style.clone(), &taffy_ids)
                    .unwrap_or_else(|e| {
                        panic!(
                            "Expected to create rank child container node for {node_id}. \
                             Error: {e}"
                        )
                    })
            })
            .collect();

        let node_shape = node_shapes
            .get(&ir_node_id)
            .unwrap_or_else(|| panic!("There was no node shape for {ir_node_id}."));

        match node_shape {
            NodeShape::Rect(_node_shape_rect) => {
                let mut wrapper_children = vec![taffy_text_node_id];
                wrapper_children.extend(rank_container_ids);

                let wrapper_node_id = taffy_tree
                    .new_with_children(wrapper_style, &wrapper_children)
                    .unwrap_or_else(|e| {
                        panic!("Expected to create wrapper node for {node_id}. Error: {e}")
                    });

                node_id_to_taffy.insert(
                    ir_node_id.clone(),
                    NodeToTaffyNodeIds::Wrapper {
                        wrapper_node_id,
                        text_node_id: taffy_text_node_id,
                    },
                );
                taffy_id_to_node.insert(wrapper_node_id, ir_node_id);

                wrapper_node_id
            }
            NodeShape::Circle(node_shape_circle) => {
                // Circle wrapper:
                //
                // ```yaml
                // wrapper_node:
                //   - label_wrapper_node: # flex row
                //     - circle_node
                //     - text_node
                //   - child_container_0  # rank n
                //   - child_container_1  # rank n + 1
                // ```
                let circle_radius = node_shape_circle.radius();
                let circle_diameter = circle_radius * 2.0;

                let circle_node_id = taffy_tree
                    .new_leaf(Style {
                        size: Size {
                            width: taffy::style::Dimension::length(circle_diameter),
                            height: taffy::style::Dimension::length(circle_diameter),
                        },
                        flex_shrink: 0.0,
                        ..Default::default()
                    })
                    .unwrap_or_else(|e| {
                        panic!("Expected to create circle leaf node for {node_id}. Error: {e}")
                    });

                let label_wrapper_style = Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: Some(AlignItems::Center),
                    gap: Size::length(4.0),
                    ..Default::default()
                };

                let label_wrapper_node_id = taffy_tree
                    .new_with_children(label_wrapper_style, &[circle_node_id, taffy_text_node_id])
                    .unwrap_or_else(|e| {
                        panic!("Expected to create label wrapper node for {node_id}. Error: {e}")
                    });

                let mut wrapper_children = vec![label_wrapper_node_id];
                wrapper_children.extend(rank_container_ids);

                let wrapper_node_id = taffy_tree
                    .new_with_children(wrapper_style, &wrapper_children)
                    .unwrap_or_else(|e| {
                        panic!("Expected to create wrapper node for {node_id}. Error: {e}")
                    });

                node_id_to_taffy.insert(
                    ir_node_id.clone(),
                    NodeToTaffyNodeIds::WrapperCircle {
                        wrapper_node_id,
                        label_wrapper_node_id,
                        circle_node_id,
                        text_node_id: taffy_text_node_id,
                    },
                );
                taffy_id_to_node.insert(wrapper_node_id, ir_node_id);

                wrapper_node_id
            }
        }
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
        node_id: &Id,
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
                    // We use `AlignItems::Start` because we want coordinates to be as close to the
                    // top-left corner as possible. If we use `AlignItems::Center`, the coordinates
                    // may be negative when the content width exceeds the diagram dimension.
                    align_items: Some(AlignItems::Start),
                    justify_items: Some(AlignItems::Start),
                    align_content: Some(AlignContent::Start),
                    justify_content: Some(AlignContent::Start),
                    gap: Size::length(flex_layout.gap()),
                    flex_direction: flex_direction_to_taffy(flex_layout.direction()),
                    flex_wrap: if flex_layout.wrap() {
                        FlexWrap::Wrap
                    } else {
                        FlexWrap::NoWrap
                    },
                    ..Default::default()
                },
                NodeLayout::Leaf(leaf_layout) => Style {
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
            })
            .unwrap_or_default()
    }

    /// Returns the `taffy::Style` for a wrapper node and its text node.
    fn taffy_wrapper_node_styles(
        node_layouts: &NodeLayouts,
        node_id: &Id,
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
                        align_content: Some(AlignContent::FlexStart),
                        justify_content: Some(AlignContent::FlexStart),
                        flex_direction: FlexDirection::Column,
                        flex_wrap: FlexWrap::NoWrap,
                        ..Default::default()
                    };
                    // Leaf node doesn't need much difference from wrapper style
                    let text_style = Style {
                        padding: Rect {
                            left: LengthPercentage::length(flex_layout.padding_left()),
                            right: LengthPercentage::length(flex_layout.padding_right()),
                            top: LengthPercentage::ZERO,
                            bottom: LengthPercentage::ZERO,
                        },
                        ..Default::default()
                    };
                    let child_container_style = Style {
                        display: Display::Flex,
                        max_size: Size::auto(),
                        gap: Size::length(flex_layout.gap()),
                        flex_direction: flex_direction_to_taffy(flex_layout.direction()),
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
                NodeLayout::Leaf(leaf_layout) => TaffyWrapperNodeStyles::new(leaf_layout),
            })
            .unwrap_or_default()
    }

    /// Returns the size of a node based on its layout and available space.
    /// This is called during layout computation and only computes sizes.
    /// Syntax highlighting is deferred to a separate pass after layout.
    fn node_size_measure(
        node_measure_context: &mut NodeMeasureContext<'_>,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        node_context: Option<&mut NodeContext>,
        style: &taffy::Style,
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
            entity_descs,
            char_width,
            lod,
        } = node_measure_context;

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
                            Some(desc) => Cow::Owned(format!("# {node_name}\n\n{desc}")),
                            None => Cow::Borrowed(node_name),
                        }
                    }
                }
            })
            .unwrap_or(Cow::Borrowed(""));

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

        taffy::Size {
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
