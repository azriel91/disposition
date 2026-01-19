use std::borrow::Cow;

use disposition_ir_model::{
    entity::{EntityDescs, EntityType, EntityTypes},
    layout::{NodeLayout, NodeLayouts},
    node::{NodeHierarchy, NodeId, NodeInbuilt, NodeNames},
    IrDiagram,
};
use disposition_model_common::Map;
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
use unicode_segmentation::UnicodeSegmentation;

/// Monospace character width as a ratio of font size.
/// For Noto Sans Mono at 11px, the character width is approximately 6.6px (0.6
/// * 11).
const MONOSPACE_CHAR_WIDTH_RATIO: f32 = 0.6;
const EMOJI_CHAR_WIDTH: f32 = 2.29;

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
            node_id_to_taffy: &mut node_id_to_taffy,
            taffy_id_to_node: &mut taffy_id_to_node,
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
            &mut taffy_id_to_node,
            node_layouts,
            dimension,
            thing_taffy_node_ids,
            process_taffy_node_ids,
            tag_taffy_node_ids,
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
                let (layout, node_context) = match taffy_node_ids {
                    NodeToTaffyNodeIds::Leaf { text_node_id }
                    | NodeToTaffyNodeIds::Wrapper {
                        wrapper_node_id: _,
                        text_node_id,
                    } => {
                        let Ok(layout) = taffy_tree.layout(text_node_id) else {
                            return;
                        };
                        let Some(node_context) = taffy_tree.get_node_context(text_node_id) else {
                            return;
                        };

                        (layout, node_context)
                    }
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
                let max_width = layout.size.width;

                // Compute line wrapping using simple monospace calculation
                let wrapped_lines = wrap_text_monospace(&text, char_width, max_width);

                // Get style info for padding calculations
                let padding_left = layout.padding.left;
                let padding_top = layout.padding.top;

                // Note: we shift the text by half a character width because even though we have
                // padding, the text still reaches the left and right edges of the node.
                //
                // The half a character width (at each end) is added to the node's width in
                // `line_width_measure`.
                let text_leftmost_x = padding_left + 0.5 * char_width;

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
            taffy_id_to_node,
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

                let wrapper_node_id = if child_hierarchy.is_empty() {
                    let taffy_style =
                        Self::taffy_container_style(node_layouts, node_id, Size::auto());
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
                        NodeId::from(node_id.clone()),
                        NodeToTaffyNodeIds::Leaf {
                            text_node_id: taffy_text_node_id,
                        },
                    );
                    taffy_id_to_node.insert(taffy_text_node_id, NodeId::from(node_id.clone()));

                    taffy_text_node_id
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
                        taffy_id_to_node,
                    };
                    let taffy_children_ids =
                        Self::build_taffy_child_nodes_for_node(taffy_node_build_context);
                    let taffy_children_container_id = taffy_tree
                        .new_with_children(child_container_style, &taffy_children_ids)
                        .unwrap_or_else(|e| {
                            panic!("Expected to create text leaf node for {node_id}. Error: {e}")
                        });

                    let wrapper_node_id = taffy_tree
                        .new_with_children(
                            wrapper_style,
                            &[taffy_text_node_id, taffy_children_container_id],
                        )
                        .unwrap_or_else(|e| {
                            panic!("Expected to create wrapper node for {node_id}. Error: {e}")
                        });

                    node_id_to_taffy.insert(
                        NodeId::from(node_id.clone()),
                        NodeToTaffyNodeIds::Wrapper {
                            wrapper_node_id,
                            text_node_id: taffy_text_node_id,
                        },
                    );
                    taffy_id_to_node.insert(wrapper_node_id, NodeId::from(node_id.clone()));

                    wrapper_node_id
                };

                entity_type_to_nodes
                    .entry(entity_type.clone())
                    .or_default()
                    .push(wrapper_node_id);

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
            taffy_id_to_node,
        } = taffy_node_build_context;

        node_hierarchy
            .iter()
            .map(|(node_id, child_hierarchy)| {
                let node_id: &disposition_model_common::Id = node_id.as_ref();
                let entity_type = entity_types
                    .get(node_id)
                    .and_then(|entity_types| entity_types.first())
                    .unwrap_or_else(|| panic!("`entity_type` not found for {node_id}"));

                if child_hierarchy.is_empty() {
                    let taffy_style =
                        Self::taffy_container_style(node_layouts, node_id, Size::auto());
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
                        NodeId::from(node_id.clone()),
                        NodeToTaffyNodeIds::Leaf {
                            text_node_id: taffy_text_node_id,
                        },
                    );
                    taffy_id_to_node.insert(taffy_text_node_id, NodeId::from(node_id.clone()));

                    taffy_text_node_id
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
                        taffy_id_to_node,
                    };
                    let taffy_children_ids =
                        Self::build_taffy_child_nodes_for_node(taffy_node_build_context);
                    let taffy_children_container_id = taffy_tree
                        .new_with_children(child_container_style, &taffy_children_ids)
                        .unwrap_or_else(|e| {
                            panic!("Expected to create text leaf node for {node_id}. Error: {e}")
                        });

                    let wrapper_node_id = taffy_tree
                        .new_with_children(
                            wrapper_style,
                            &[taffy_text_node_id, taffy_children_container_id],
                        )
                        .unwrap_or_else(|e| {
                            panic!("Expected to create wrapper node for {node_id}. Error: {e}")
                        });

                    node_id_to_taffy.insert(
                        NodeId::from(node_id.clone()),
                        NodeToTaffyNodeIds::Wrapper {
                            wrapper_node_id,
                            text_node_id: taffy_text_node_id,
                        },
                    );
                    taffy_id_to_node.insert(wrapper_node_id, NodeId::from(node_id.clone()));

                    wrapper_node_id
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

/// Compute text dimensions using simple monospace character width calculation.
/// Returns (max_line_width, line_count).
fn compute_text_dimensions(text: &str, char_width: f32, max_width: Option<f32>) -> (f32, usize) {
    if text.is_empty() {
        return (0.0, 0);
    }

    let max_chars_per_line = max_width.map(|w| (w / char_width).floor() as usize);

    let mut line_width_max: f32 = 0.0;
    let mut line_count: usize = 0;

    text.lines().for_each(|line| {
        let line_char_count = line.chars().count();

        match max_chars_per_line {
            Some(max_chars) if max_chars > 0 && line_char_count > max_chars => {
                // Word wrap this line
                let wrapped = wrap_line_monospace(line, max_chars);
                wrapped.into_iter().for_each(|wrapped_line| {
                    // Note: Ideally we can get a library to measure all kinds of graphemes.
                    //
                    // I tried this:
                    //
                    // ```rust
                    // let width = unicode_width::UnicodeWidthStr::width_cjk(wrapped_line) as f32 * char_width;
                    // ```
                    //
                    // but it didn't count emoji widths correctly.
                    //
                    // Also tried `string-width`:
                    //
                    // ```rust
                    // let width = string_width::string_width(wrapped_line) as f32 * char_width;
                    // ```

                    let width = line_width_measure(wrapped_line, char_width);
                    line_width_max = line_width_max.max(width);
                    line_count += 1;
                });
            }
            _ => {
                // let width = string_width::string_width(line) as f32 * char_width;
                let width = line_width_measure(line, char_width);
                line_width_max = line_width_max.max(width);
                line_count += 1;
            }
        }
    });

    (line_width_max, line_count)
}

/// Returns the width in pixels to display the given line of text.
fn line_width_measure(line: &str, char_width: f32) -> f32 {
    if line.is_empty() {
        return 0.0;
    }

    let mut line_char_column_count = line
        .graphemes(true)
        .map(|grapheme| match emojis::get(grapheme).is_some() {
            true => EMOJI_CHAR_WIDTH,
            false => 1.0f32,
        })
        .sum::<f32>();

    // Add one character width
    //
    // Without this, even with node padding, the text characters reach to both ends
    // of the node, and sometimes the last character wraps down.
    //
    // Note that we shift the x coordinates of each line of text by `0.5 *
    // char_width` in `highlighted_spans_compute`.
    line_char_column_count += 1.0;

    line_char_column_count * char_width
}

/// Wrap text for display, returning owned strings for each line.
fn wrap_text_monospace(text: &str, char_width: f32, max_width: f32) -> Vec<String> {
    let max_chars = (max_width / char_width).floor() as usize;

    if max_chars == 0 {
        return text.lines().map(String::from).collect();
    }

    let mut result = Vec::new();

    text.lines().for_each(|line| {
        let wrapped = wrap_line_monospace(line, max_chars);
        result.extend(wrapped.into_iter().map(String::from));
    });

    if result.is_empty() {
        result.push(String::new());
    }

    result
}

/// Wraps a single line to fit within max_chars characters.
///
/// Tries to break at word boundaries when possible.
fn wrap_line_monospace(line: &str, max_chars: usize) -> Vec<&str> {
    if max_chars == 0 {
        return vec![line];
    }

    let mut result = Vec::new();
    let mut remaining = line;

    while !remaining.is_empty() {
        let char_count = remaining.chars().count();
        if char_count <= max_chars {
            result.push(remaining);
            break;
        }

        // Find a good break point (try to break at whitespace)
        let mut break_at_byte = 0;
        let mut break_at_char = 0;
        let mut last_space_byte = None;
        let mut last_space_char = 0;

        remaining
            .char_indices()
            .enumerate()
            .for_each(|(char_idx, (byte_idx, c))| {
                if char_idx >= max_chars {
                    return;
                }
                if c.is_whitespace() {
                    last_space_byte = Some(byte_idx);
                    last_space_char = char_idx;
                }
                break_at_byte = byte_idx + c.len_utf8();
                break_at_char = char_idx + 1;
            });

        // Prefer breaking at whitespace if we found one in the second half
        let (split_byte, split_char) =
            if let Some(space_byte) = last_space_byte.filter(|_| last_space_char > max_chars / 2) {
                (space_byte, last_space_char)
            } else {
                (break_at_byte, break_at_char)
            };

        if split_char == 0 {
            // Safety: if we can't make progress, just take the whole thing
            result.push(remaining);
            break;
        }

        result.push(&remaining[..split_byte]);
        remaining = remaining[split_byte..].trim_start();
    }

    if result.is_empty() {
        result.push("");
    }

    result
}

struct TaffyNodeBuildContext<'ctx> {
    taffy_tree: &'ctx mut TaffyTree<NodeContext>,
    nodes: &'ctx NodeNames<'static>,
    node_layouts: &'ctx NodeLayouts<'static>,
    node_hierarchy: &'ctx NodeHierarchy<'static>,
    entity_types: &'ctx EntityTypes<'static>,
    node_id_to_taffy: &'ctx mut Map<NodeId<'static>, NodeToTaffyNodeIds>,
    taffy_id_to_node: &'ctx mut Map<taffy::NodeId, NodeId<'static>>,
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
                align_items: Some(AlignItems::Start),
                justify_items: Some(AlignItems::Start),
                align_content: Some(AlignContent::Start),
                justify_content: Some(AlignContent::Start),
                ..Default::default()
            },
        }
    }
}

struct NodeMeasureContext<'ctx> {
    nodes: &'ctx NodeNames<'static>,
    entity_descs: &'ctx EntityDescs<'static>,
    /// Monospace character width in pixels.
    char_width: f32,
    /// Level of detail for the diagram.
    lod: &'ctx DiagramLod,
}
