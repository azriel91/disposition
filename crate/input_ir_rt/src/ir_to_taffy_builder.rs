use std::{borrow::Cow, collections::BTreeMap};

use disposition_ir_model::{
    edge::{EdgeFaceAssignments, EdgeGroups, EdgeId, EdgeLabels},
    entity::{EntityDescs, EntityType, EntityTypes},
    layout::{FlexDirection as ModelFlexDirection, NodeLayout, NodeLayouts},
    node::{
        NodeFace, NodeFaceEdges, NodeHierarchy, NodeId, NodeInbuilt, NodeNames, NodeNestingInfos,
        NodeRank, NodeRanksNested, NodeShape, NodeShapes,
    },
    IrDiagram,
};
use disposition_model_common::{Id, Map, RankDir};
use disposition_taffy_model::{
    taffy::{
        self,
        style::{FlexDirection, LengthPercentageAuto},
        AlignContent, AlignItems, AvailableSpace, Display, FlexWrap, LengthPercentage, Rect, Size,
        Style, TaffyTree,
    },
    DiagramLod, DiagramNodeCtx, Dimension, DimensionAndLod, EdgeLabelCtx, EdgeLabelTaffyNodeIds,
    EdgeSpacerTaffyNodes, EntityHighlightedSpan, EntityHighlightedSpans, IrToTaffyError,
    NodeToTaffyNodeIds, ProcessesIncluded, TaffyNodeCtx, TaffyNodeMappings, TEXT_FONT_SIZE,
    TEXT_LINE_HEIGHT,
};
use taffy::{
    prelude::TaffyZero,
    style_helpers::{auto, line},
    JustifyContent, JustifyItems,
};
use typed_builder::TypedBuilder;

use self::{
    edge_lca_sibling_distance::EdgeLcaSiblingDistance,
    edge_spacer_builder::EdgeSpacerBuilder,
    taffy_node_build_context::{
        EdgeLabelLeafBuilt, NodeMeasureContext, TaffyNodeBuildContext, TaffyWrapperNodeStyles,
    },
    text_measure::{
        compute_text_dimensions, line_width_measure, wrap_text_monospace,
        MONOSPACE_CHAR_WIDTH_RATIO,
    },
};

mod edge_lca_sibling_distance;
mod edge_spacer_builder;
mod taffy_node_build_context;
mod text_measure;

type NodeRankToTaffyNodeId = BTreeMap<NodeRank, Vec<taffy::NodeId>>;

/// Padding (in pixels) added to the far side of an edge label leaf node.
///
/// The edge path contacts the label at one edge; this padding creates visual
/// separation on the opposite side so the path does not overlap adjacent
/// labels or the face boundary on the exit side.
const EDGE_LABEL_PADDING_PX: f32 = 4.0;

/// Padding (in pixels) added to both sides of a node so edges don't begin at
/// the very edge.
const NODE_SIDE_PADDING_PX: LengthPercentage = LengthPercentage::length(8.0);

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
            edge_groups,
            entity_descs,
            edge_labels,
            entity_tooltips: _,
            entity_types,
            tailwind_classes: _,
            node_layouts,
            node_ranks_nested,
            node_nesting_infos,
            edge_face_assignments,
            node_face_edges,
            node_shapes,
            process_step_entities: _,
            render_options,
            css: _,
        } = ir_diagram;

        let rank_dir = render_options.rank_dir;

        let DimensionAndLod { dimension, lod } = dimension_and_lod;

        let mut taffy_tree = TaffyTree::new();
        let mut node_id_to_taffy = Map::new();
        let mut taffy_id_to_node = Map::new();
        let mut node_id_to_envelope_taffy_node: Map<NodeId<'static>, taffy::NodeId> = Map::new();
        let mut edge_label_leaves: Vec<EdgeLabelLeafBuilt> = Vec::new();

        let taffy_node_build_context = TaffyNodeBuildContext {
            taffy_tree: &mut taffy_tree,
            nodes,
            node_layouts,
            node_hierarchy,
            entity_types,
            node_shapes,
            node_ranks_nested,
            node_nesting_infos,
            node_id_to_taffy: &mut node_id_to_taffy,
            taffy_id_to_node: &mut taffy_id_to_node,
            node_face_edges,
            node_id_to_envelope_taffy_node: &mut node_id_to_envelope_taffy_node,
            edge_label_leaves: &mut edge_label_leaves,
            rank_dir,
        };
        let (node_rank_to_nodes_by_entity_type, nested_edge_spacer_taffy_nodes) =
            Self::build_taffy_nodes_for_first_level_nodes(
                taffy_node_build_context,
                processes_included,
                edge_groups,
            );
        let mut thing_rank_to_taffy_ids = node_rank_to_nodes_by_entity_type
            .get(&EntityType::ThingDefault)
            .cloned()
            .unwrap_or_default();
        let mut tag_rank_to_taffy_ids = node_rank_to_nodes_by_entity_type
            .get(&EntityType::TagDefault)
            .cloned()
            .unwrap_or_default();
        let mut process_rank_to_taffy_ids = node_rank_to_nodes_by_entity_type
            .get(&EntityType::ProcessDefault)
            .cloned()
            .unwrap_or_default();

        // === Insert spacer taffy nodes for cross-rank edges === //
        //
        // For each edge that crosses multiple ranks, we insert small spacer
        // leaf nodes at every intermediate rank. The edge path will later
        // be routed through these spacer positions to avoid overlapping
        // other nodes.
        let mut edge_spacer_taffy_nodes: Map<EdgeId<'static>, EdgeSpacerTaffyNodes> = Map::new();
        edge_spacer_taffy_nodes.extend(nested_edge_spacer_taffy_nodes);
        edge_spacer_taffy_nodes.extend(EdgeSpacerBuilder::build(
            &mut taffy_tree,
            edge_groups,
            node_nesting_infos,
            node_ranks_nested,
            entity_types,
            &EntityType::ThingDefault,
            &mut thing_rank_to_taffy_ids,
            None,
        ));
        edge_spacer_taffy_nodes.extend(EdgeSpacerBuilder::build(
            &mut taffy_tree,
            edge_groups,
            node_nesting_infos,
            node_ranks_nested,
            entity_types,
            &EntityType::TagDefault,
            &mut tag_rank_to_taffy_ids,
            None,
        ));
        edge_spacer_taffy_nodes.extend(EdgeSpacerBuilder::build(
            &mut taffy_tree,
            edge_groups,
            node_nesting_infos,
            node_ranks_nested,
            entity_types,
            &EntityType::ProcessDefault,
            &mut process_rank_to_taffy_ids,
            None,
        ));

        // Create rank sub-containers for top-level nodes, mirroring the
        // rank-based child container logic used inside
        // `build_taffy_nodes_for_node_with_child_hierarchy`.
        //
        // Each entity type gets its own set of rank containers using the
        // style of its parent container.
        let thing_rank_container_ids = Self::build_taffy_rank_containers_for_first_level_nodes(
            &mut taffy_tree,
            node_layouts,
            NodeInbuilt::ThingsContainer,
            thing_rank_to_taffy_ids,
        );
        let tag_rank_container_ids = Self::build_taffy_rank_containers_for_first_level_nodes(
            &mut taffy_tree,
            node_layouts,
            NodeInbuilt::TagsContainer,
            tag_rank_to_taffy_ids,
        );
        let process_rank_container_ids = Self::build_taffy_rank_containers_for_first_level_nodes(
            &mut taffy_tree,
            node_layouts,
            NodeInbuilt::ProcessesContainer,
            process_rank_to_taffy_ids,
        );

        let node_inbuilt_to_taffy = Self::build_taffy_container_nodes(
            &mut taffy_tree,
            &mut taffy_id_to_node,
            node_layouts,
            dimension,
            &thing_rank_container_ids,
            &process_rank_container_ids,
            &tag_rank_container_ids,
        );

        let Some(root) = node_inbuilt_to_taffy.get(&NodeInbuilt::Root).copied() else {
            panic!("`root` node not present in `node_inbuilt_to_taffy`.");
        };

        // Precompute monospace character width
        let char_width = TEXT_FONT_SIZE * MONOSPACE_CHAR_WIDTH_RATIO;

        // Pre-compute edge endpoint node IDs for edge label slot sizing.
        let edge_id_to_endpoint_node_ids = Self::edge_id_to_node_ids_build(edge_groups);

        // Compute layout (size measurement only, no syntax highlighting)
        let mut node_measure_context = NodeMeasureContext {
            nodes,
            entity_descs,
            edge_labels,
            edge_id_to_endpoint_node_ids: &edge_id_to_endpoint_node_ids,
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
                |known_dimensions, available_space, _taffy_node_id, taffy_node_ctx, style| {
                    Self::node_size_measure(
                        &mut node_measure_context,
                        known_dimensions,
                        available_space,
                        taffy_node_ctx,
                        style,
                    )
                },
            )
            .expect("Expected layout computation to succeed.");

        // Merge collected edge label leaf nodes into the edge label taffy node
        // map now that all envelope nodes have been built.
        let edge_label_taffy_nodes = Self::edge_label_taffy_nodes_build(
            edge_label_leaves,
            edge_face_assignments,
            edge_groups,
        );

        // Compute highlighted spans *after* layout is complete.
        //
        // This is done once per node instead of multiple times during layout
        // measurement
        let entity_highlighted_spans = Self::highlighted_spans_compute(
            &taffy_tree,
            &node_id_to_taffy,
            &edge_label_taffy_nodes,
            nodes,
            entity_descs,
            edge_labels,
            char_width,
            lod,
        );

        std::iter::once(TaffyNodeMappings {
            taffy_tree,
            node_inbuilt_to_taffy,
            node_id_to_taffy,
            taffy_id_to_node,
            edge_spacer_taffy_nodes,
            entity_highlighted_spans,
            edge_label_taffy_nodes,
            node_id_to_envelope_taffy_node,
        })
    }

    /// Compute highlighted spans for all nodes after layout is complete.
    /// This is much more efficient than doing it during measure() which gets
    /// called multiple times.
    #[allow(clippy::too_many_arguments)]
    fn highlighted_spans_compute(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        node_id_to_taffy: &Map<NodeId<'static>, NodeToTaffyNodeIds>,
        edge_label_taffy_nodes: &Map<EdgeId<'static>, EdgeLabelTaffyNodeIds>,
        nodes: &NodeNames<'static>,
        entity_descs: &EntityDescs<'static>,
        edge_labels: &EdgeLabels<'static>,
        char_width: f32,
        lod: &DiagramLod,
    ) -> EntityHighlightedSpans<'static> {
        let mut entity_highlighted_spans = EntityHighlightedSpans::with_capacity(
            node_id_to_taffy.len() + edge_label_taffy_nodes.len(),
        );

        let line_height = TEXT_LINE_HEIGHT;

        node_id_to_taffy
            .iter()
            .for_each(|(node_id, &taffy_node_ids)| {
                let (wrapper_node_layout, text_node_layout, diagram_node_ctx) = match taffy_node_ids
                {
                    NodeToTaffyNodeIds::Leaf { text_node_id } => {
                        let Ok(text_node_layout) = taffy_tree.layout(text_node_id) else {
                            return;
                        };
                        let Some(TaffyNodeCtx::DiagramNode(diagram_node_ctx)) =
                            taffy_tree.get_node_context(text_node_id)
                        else {
                            return;
                        };
                        (text_node_layout, text_node_layout, diagram_node_ctx)
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
                        let Some(TaffyNodeCtx::DiagramNode(diagram_node_ctx)) =
                            taffy_tree.get_node_context(text_node_id)
                        else {
                            return;
                        };

                        (wrapper_node_layout, text_node_layout, diagram_node_ctx)
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

                let entity_id = &diagram_node_ctx.entity_id;

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

        // === Edge label spans === //
        //
        // For DiagramLod::Normal, compute highlighted spans for edge label
        // slots. The from_label slot uses `edge_label.from` as its text and
        // the to_label slot uses `edge_label.to`, allowing each endpoint to
        // show different text. Spans are stored under
        // `{edge_id}__from_label` and `{edge_id}__to_label` keys.
        if matches!(lod, DiagramLod::Normal) {
            edge_label_taffy_nodes
                .iter()
                .for_each(|(edge_id, edge_label_taffy_node_ids)| {
                    let Some(edge_label) = edge_labels.get(edge_id) else {
                        return;
                    };

                    // Compute and store highlighted spans for the from_label slot.
                    if let Some(from_taffy_node_id) =
                        edge_label_taffy_node_ids.from_label_taffy_node_id
                    {
                        let from_text = edge_label.from.as_str();
                        if let Some(from_spans) =
                            Self::highlighted_spans_compute_edge_label_slot(
                                taffy_tree,
                                from_taffy_node_id,
                                from_text,
                                char_width,
                                line_height,
                            )
                        {
                            let from_label_key =
                                Id::try_from(format!("{edge_id}__from_label"))
                                    .expect("`edge_id` is a valid `Id`, so appending `__from_label` is also valid");
                            entity_highlighted_spans.insert(from_label_key, from_spans);
                        }
                    }

                    // Compute and store highlighted spans for the to_label slot.
                    if let Some(to_taffy_node_id) =
                        edge_label_taffy_node_ids.to_label_taffy_node_id
                    {
                        let to_text = edge_label.to.as_str();
                        if let Some(to_spans) =
                            Self::highlighted_spans_compute_edge_label_slot(
                                taffy_tree,
                                to_taffy_node_id,
                                to_text,
                                char_width,
                                line_height,
                            )
                        {
                            let to_label_key =
                                Id::try_from(format!("{edge_id}__to_label"))
                                    .expect("`edge_id` is a valid `Id`, so appending `__to_label` is also valid");
                            entity_highlighted_spans.insert(to_label_key, to_spans);
                        }
                    }
                });
        }

        entity_highlighted_spans
    }

    /// Computes highlighted spans for a single edge label slot.
    ///
    /// Returns `None` if `text` is empty or the taffy layout cannot be read.
    fn highlighted_spans_compute_edge_label_slot(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        taffy_node_id: taffy::NodeId,
        text: &str,
        char_width: f32,
        line_height: f32,
    ) -> Option<Vec<EntityHighlightedSpan>> {
        if text.is_empty() {
            return None;
        }

        let Ok(node_layout) = taffy_tree.layout(taffy_node_id) else {
            return None;
        };

        let max_width = node_layout.size.width;
        let wrapped_lines = wrap_text_monospace(text, char_width, max_width);

        let padding_left = node_layout.padding.left;
        let padding_top = node_layout.padding.top;
        let text_leftmost_x = padding_left + 0.5 * char_width;

        let spans = wrapped_lines
            .iter()
            .enumerate()
            .map(|(line_index, line)| EntityHighlightedSpan {
                x: text_leftmost_x,
                y: (line_index + 1) as f32 * line_height + padding_top,
                width: line_width_measure(line, char_width),
                height: line_height,
                text: line.to_string(),
            })
            .collect();

        Some(spans)
    }

    /// Creates rank sub-containers for first-level nodes of a given entity
    /// type.
    ///
    /// Each rank level gets its own flex container using the style of the
    /// parent `NodeInbuilt` container. The returned `Vec` contains one taffy
    /// node per rank, ordered by rank.
    fn build_taffy_rank_containers_for_first_level_nodes(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        node_layouts: &NodeLayouts,
        node_inbuilt: NodeInbuilt,
        rank_to_taffy_ids: NodeRankToTaffyNodeId,
    ) -> Vec<taffy::NodeId> {
        // Not sure if this is the best way to handle the container styles, but we use
        // the `NodeInbuilt` container style for the rank children containers, and
        // invert the `FlexDirection` on the actual `NodeInbuilt` container style.
        let rank_container_style =
            Self::taffy_container_style(node_layouts, &node_inbuilt.id(), Size::auto());
        // Creates a new taffy node for each rank to be placed in the container.
        //
        // i.e.
        //
        // ```yaml
        // container_node:
        //   child_container_0: {} # nodes with rank n
        //   child_container_1: {} # nodes with rank n + 1
        //   child_container_2: {} # nodes with rank n + 2
        // ```
        rank_to_taffy_ids
            .into_values()
            .map(|taffy_ids| {
                taffy_tree
                    .new_with_children(rank_container_style.clone(), &taffy_ids)
                    .unwrap_or_else(|e| {
                        panic!(
                            "Expected to create rank container node for \
                             top-level {node_inbuilt}. Error: {e}"
                        )
                    })
            })
            .collect()
    }

    /// Adds the inbuilt container nodes to the `TaffyTree`.
    fn build_taffy_container_nodes(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        taffy_id_to_node: &mut Map<taffy::NodeId, NodeId>,
        node_layouts: &NodeLayouts,
        dimension: &disposition_taffy_model::Dimension,
        thing_rank_container_ids: &[taffy::NodeId],
        process_rank_container_ids: &[taffy::NodeId],
        tag_rank_container_ids: &[taffy::NodeId],
    ) -> Map<NodeInbuilt, taffy::NodeId> {
        let things_container_style = {
            let container_style = Self::taffy_container_style(
                node_layouts,
                &NodeInbuilt::ThingsContainer.id(),
                Size::auto(),
            );
            Self::container_style_invert_and_stretch(container_style)
        };
        let things_container = taffy_tree
            .new_with_children(things_container_style, thing_rank_container_ids)
            .expect("`TaffyTree::new_with_children` should be infallible.");
        let processes_container_style = {
            let container_style = Self::taffy_container_style(
                node_layouts,
                &NodeInbuilt::ProcessesContainer.id(),
                Size::auto(),
            );
            Self::container_style_invert_and_stretch(container_style)
        };
        let processes_container = taffy_tree
            .new_with_children(processes_container_style, process_rank_container_ids)
            .expect("`TaffyTree::new_with_children` should be infallible.");
        let things_and_processes_container = Self::taffy_container_node(
            taffy_tree,
            node_layouts,
            NodeInbuilt::ThingsAndProcessesContainer,
            Size::auto(),
            &[processes_container, things_container],
        );
        let tags_container_style = {
            let container_style = Self::taffy_container_style(
                node_layouts,
                &NodeInbuilt::TagsContainer.id(),
                Size::auto(),
            );
            Self::container_style_invert_and_stretch(container_style)
        };
        let tags_container = taffy_tree
            .new_with_children(tags_container_style, tag_rank_container_ids)
            .expect("`TaffyTree::new_with_children` should be infallible.");

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

    /// Sets the flex direction to the opposite of the container style.
    ///
    /// The flex direction inversion is because the desired flex direction is
    /// set on the rank container nodes, so when the user has requested `Row`,
    /// each rank container uses the `Row` layout, and the parent of the ranked
    /// containers should be `Column`.
    fn container_style_invert_and_stretch(container_style: Style) -> Style {
        let flex_direction = match container_style.flex_direction {
            FlexDirection::Row => FlexDirection::Column,
            FlexDirection::Column => FlexDirection::Row,
            FlexDirection::RowReverse => FlexDirection::ColumnReverse,
            FlexDirection::ColumnReverse => FlexDirection::RowReverse,
        };
        Style {
            flex_direction,
            ..container_style
        }
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
        edge_groups: &EdgeGroups<'static>,
    ) -> (
        Map<EntityType, NodeRankToTaffyNodeId>,
        Map<EdgeId<'static>, EdgeSpacerTaffyNodes>,
    ) {
        let TaffyNodeBuildContext {
            nodes,
            taffy_tree,
            node_layouts,
            node_hierarchy,
            entity_types,
            node_shapes,
            node_ranks_nested,
            node_nesting_infos,
            node_id_to_taffy,
            taffy_id_to_node,
            node_face_edges,
            node_id_to_envelope_taffy_node,
            edge_label_leaves,
            rank_dir,
        } = taffy_node_build_context;

        let mut edge_spacer_taffy_nodes: Map<EdgeId<'static>, EdgeSpacerTaffyNodes> = Map::new();

        let entity_type_to_node_rank_to_taffy_node_ids = node_hierarchy.iter().fold(
            Map::<EntityType, NodeRankToTaffyNodeId>::new(),
            |mut entity_type_to_node_rank_to_taffy_node_ids, (node_id, child_hierarchy)| {
                let node_id: &Id = node_id.as_ref();
                let Some(entity_type) = entity_types
                    .get(node_id)
                    .and_then(|entity_types| entity_types.first())
                else {
                    // Skip nodes without an entity type -- probably something extra in the
                    // hierarchy without a node name.
                    return entity_type_to_node_rank_to_taffy_node_ids;
                };

                if matches!(entity_type, EntityType::ProcessDefault) {
                    match processes_included {
                        ProcessesIncluded::All => {}
                        ProcessesIncluded::Filter { process_ids } => {
                            if process_ids.contains(node_id) {
                                // Don't add this process.
                                return entity_type_to_node_rank_to_taffy_node_ids;
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
                        node_face_edges,
                        node_id_to_envelope_taffy_node,
                        edge_label_leaves,
                        rank_dir,
                    )
                } else {
                    let (wrapper_node_id, nested_edge_spacer_taffy_nodes) =
                        Self::build_taffy_nodes_for_node_with_child_hierarchy(
                            nodes,
                            taffy_tree,
                            node_layouts,
                            node_shapes,
                            entity_types,
                            node_ranks_nested,
                            node_nesting_infos,
                            node_id_to_taffy,
                            taffy_id_to_node,
                            child_hierarchy,
                            node_id,
                            entity_type,
                            edge_groups,
                            node_face_edges,
                            node_id_to_envelope_taffy_node,
                            edge_label_leaves,
                            rank_dir,
                        );
                    edge_spacer_taffy_nodes.extend(nested_edge_spacer_taffy_nodes);
                    wrapper_node_id
                };

                let ir_node_id = NodeId::from(node_id.clone());
                let rank = node_ranks_nested
                    .node_rank_for(&ir_node_id, node_nesting_infos)
                    .unwrap_or(NodeRank::new(0));

                entity_type_to_node_rank_to_taffy_node_ids
                    .entry(entity_type.clone())
                    .or_default()
                    .entry(rank)
                    .or_default()
                    .push(wrapper_node_id);

                entity_type_to_node_rank_to_taffy_node_ids
            },
        );

        (
            entity_type_to_node_rank_to_taffy_node_ids,
            edge_spacer_taffy_nodes,
        )
    }

    /// Adds the child taffy nodes for a given IR diagram node, grouped by rank.
    ///
    /// Returns a `BTreeMap` from `NodeRank` to the list of taffy node IDs at
    /// that rank. This allows the caller to create separate child containers
    /// for each rank level.
    fn build_taffy_child_nodes_for_node_by_rank(
        taffy_node_build_context: TaffyNodeBuildContext<'_>,
        edge_groups: &EdgeGroups<'static>,
    ) -> (
        NodeRankToTaffyNodeId,
        Map<EdgeId<'static>, EdgeSpacerTaffyNodes>,
    ) {
        let TaffyNodeBuildContext {
            nodes,
            taffy_tree,
            node_layouts,
            node_hierarchy,
            entity_types,
            node_shapes,
            node_ranks_nested,
            node_nesting_infos,
            node_id_to_taffy,
            taffy_id_to_node,
            node_face_edges,
            node_id_to_envelope_taffy_node,
            edge_label_leaves,
            rank_dir,
        } = taffy_node_build_context;

        let mut rank_to_taffy_ids: NodeRankToTaffyNodeId = BTreeMap::new();
        let mut edge_spacer_taffy_nodes: Map<EdgeId<'static>, EdgeSpacerTaffyNodes> = Map::new();

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
                    node_face_edges,
                    node_id_to_envelope_taffy_node,
                    edge_label_leaves,
                    rank_dir,
                )
            } else {
                let (wrapper_node_id, nested_edge_spacer_taffy_nodes) =
                    Self::build_taffy_nodes_for_node_with_child_hierarchy(
                        nodes,
                        taffy_tree,
                        node_layouts,
                        node_shapes,
                        entity_types,
                        node_ranks_nested,
                        node_nesting_infos,
                        node_id_to_taffy,
                        taffy_id_to_node,
                        child_hierarchy,
                        node_id,
                        entity_type,
                        edge_groups,
                        node_face_edges,
                        node_id_to_envelope_taffy_node,
                        edge_label_leaves,
                        rank_dir,
                    );
                edge_spacer_taffy_nodes.extend(nested_edge_spacer_taffy_nodes);
                wrapper_node_id
            };

            let ir_node_id = NodeId::from(node_id.clone());
            let rank = node_ranks_nested
                .node_rank_for(&ir_node_id, node_nesting_infos)
                .unwrap_or(NodeRank::new(0));

            rank_to_taffy_ids
                .entry(rank)
                .or_default()
                .push(taffy_node_id);
        }

        (rank_to_taffy_ids, edge_spacer_taffy_nodes)
    }

    #[allow(clippy::too_many_arguments)]
    fn build_taffy_nodes_for_node_without_child_hierarchy(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        node_layouts: &NodeLayouts<'static>,
        node_shapes: &NodeShapes<'static>,
        node_id_to_taffy: &mut Map<NodeId<'static>, NodeToTaffyNodeIds>,
        taffy_id_to_node: &mut Map<taffy::NodeId, NodeId<'static>>,
        node_id: &Id<'static>,
        entity_type: &EntityType,
        node_face_edges: &NodeFaceEdges<'static>,
        node_id_to_envelope_taffy_node: &mut Map<NodeId<'static>, taffy::NodeId>,
        edge_label_leaves: &mut Vec<EdgeLabelLeafBuilt>,
        rank_dir: RankDir,
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
                        TaffyNodeCtx::DiagramNode(DiagramNodeCtx {
                            entity_id: node_id.clone(),
                            entity_type: entity_type.clone(),
                        }),
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
                let (envelope_node_id, new_label_leaves) = Self::taffy_envelope_node_build(
                    taffy_tree,
                    &ir_node_id,
                    taffy_text_node_id,
                    node_face_edges,
                    rank_dir,
                );
                edge_label_leaves.extend(new_label_leaves);
                node_id_to_envelope_taffy_node.insert(ir_node_id.clone(), envelope_node_id);
                taffy_id_to_node.insert(taffy_text_node_id, ir_node_id);

                envelope_node_id
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
                        TaffyNodeCtx::DiagramNode(DiagramNodeCtx {
                            entity_id: node_id.clone(),
                            entity_type: entity_type.clone(),
                        }),
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
                    gap: Size::length(4.0f32),
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
                let (envelope_node_id, new_label_leaves) = Self::taffy_envelope_node_build(
                    taffy_tree,
                    &ir_node_id,
                    wrapper_node_id,
                    node_face_edges,
                    rank_dir,
                );
                edge_label_leaves.extend(new_label_leaves);
                node_id_to_envelope_taffy_node.insert(ir_node_id.clone(), envelope_node_id);
                taffy_id_to_node.insert(wrapper_node_id, ir_node_id);

                envelope_node_id
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn build_taffy_nodes_for_node_with_child_hierarchy(
        nodes: &NodeNames<'static>,
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        node_layouts: &NodeLayouts<'static>,
        node_shapes: &NodeShapes<'static>,
        entity_types: &EntityTypes<'static>,
        node_ranks_nested: &NodeRanksNested<'static>,
        node_nesting_infos: &NodeNestingInfos<'static>,
        node_id_to_taffy: &mut Map<NodeId<'static>, NodeToTaffyNodeIds>,
        taffy_id_to_node: &mut Map<taffy::NodeId, NodeId<'static>>,
        child_hierarchy: &NodeHierarchy<'static>,
        node_id: &Id<'static>,
        entity_type: &EntityType,
        edge_groups: &EdgeGroups<'static>,
        node_face_edges: &NodeFaceEdges<'static>,
        node_id_to_envelope_taffy_node: &mut Map<NodeId<'static>, taffy::NodeId>,
        edge_label_leaves: &mut Vec<EdgeLabelLeafBuilt>,
        rank_dir: RankDir,
    ) -> (taffy::NodeId, Map<EdgeId<'static>, EdgeSpacerTaffyNodes>) {
        let ir_node_id = NodeId::from(node_id.clone());
        let mut edge_spacer_taffy_nodes: Map<EdgeId<'static>, EdgeSpacerTaffyNodes> = Map::new();

        let TaffyWrapperNodeStyles {
            wrapper_style,
            text_style,
            child_container_style,
        } = Self::taffy_wrapper_node_styles(node_layouts, node_id);
        let taffy_text_node_id = taffy_tree
            .new_leaf_with_context(
                text_style,
                TaffyNodeCtx::DiagramNode(DiagramNodeCtx {
                    entity_id: node_id.clone(),
                    entity_type: entity_type.clone(),
                }),
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
            node_ranks_nested,
            node_nesting_infos,
            node_id_to_taffy,
            taffy_id_to_node,
            node_face_edges,
            node_id_to_envelope_taffy_node,
            edge_label_leaves,
            rank_dir,
        };
        let (mut rank_to_taffy_ids, nested_edge_spacer_taffy_nodes) =
            Self::build_taffy_child_nodes_for_node_by_rank(taffy_node_build_context, edge_groups);
        edge_spacer_taffy_nodes.extend(nested_edge_spacer_taffy_nodes);

        // === Insert spacer nodes for edges nested within this node === //
        let lca_node_id = NodeId::from(node_id.clone());
        for target_entity_type in &[
            EntityType::ThingDefault,
            EntityType::TagDefault,
            EntityType::ProcessDefault,
        ] {
            edge_spacer_taffy_nodes.extend(EdgeSpacerBuilder::build(
                taffy_tree,
                edge_groups,
                node_nesting_infos,
                node_ranks_nested,
                entity_types,
                target_entity_type,
                &mut rank_to_taffy_ids,
                Some(&lca_node_id),
            ));
        }

        // === Insert spacer nodes for edges crossing this container === //
        //
        // When an edge has one endpoint outside this container and the
        // other deeply nested inside, the edge path needs waypoints
        // alongside the intermediate sibling children so it routes
        // around them instead of drawing over them.
        edge_spacer_taffy_nodes.extend(EdgeSpacerBuilder::build_cross_container_spacers(
            taffy_tree,
            edge_groups,
            node_nesting_infos,
            node_ranks_nested,
            &mut rank_to_taffy_ids,
            &ir_node_id,
            child_hierarchy,
        ));

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
            .into_values()
            .map(|taffy_ids| {
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
                let (envelope_node_id, new_label_leaves) = Self::taffy_envelope_node_build(
                    taffy_tree,
                    &ir_node_id,
                    wrapper_node_id,
                    node_face_edges,
                    rank_dir,
                );
                edge_label_leaves.extend(new_label_leaves);
                node_id_to_envelope_taffy_node.insert(ir_node_id.clone(), envelope_node_id);
                taffy_id_to_node.insert(wrapper_node_id, ir_node_id);

                (envelope_node_id, edge_spacer_taffy_nodes)
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
                    gap: Size::length(4.0f32),
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
                let (envelope_node_id, new_label_leaves) = Self::taffy_envelope_node_build(
                    taffy_tree,
                    &ir_node_id,
                    wrapper_node_id,
                    node_face_edges,
                    rank_dir,
                );
                edge_label_leaves.extend(new_label_leaves);
                node_id_to_envelope_taffy_node.insert(ir_node_id.clone(), envelope_node_id);
                taffy_id_to_node.insert(wrapper_node_id, ir_node_id);

                (envelope_node_id, edge_spacer_taffy_nodes)
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
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
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

    /// Returns the `taffy::Style` for container nodes and leaf nodes.
    ///
    /// The values for each style are taken from the `NodeLayouts` map.
    ///
    /// This includes:
    ///
    /// * `inbuilt` nodes: e.g. `NodeInbuilt::ThingsAndProcessesContainer`.
    /// * `rank_container` nodes: `taffy` nodes that contain the taffy container
    ///   nodes for a given rank.
    /// * `rank` nodes: The `taffy` nodes that contain the child nodes for a
    ///   given rank.
    /// * leaf nodes: the `taffy` nodes that contain the text of a diagram node.
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
                    // We previously used `AlignItems::Stretch` because we want coordinates to be as
                    // close to the top-left corner as possible, as well as resizing each node to be
                    // as wide as the widest node which looks more visually aesthetic.
                    //
                    // We now use `AlignItems::FlexStart` to align the content to the start of the
                    // container, which ensures that the coordinates are as close to the top-left
                    // corner as possible, as well as not inadvertently stretching nodes' height.
                    //
                    // If we use `AlignItems::Center`, the coordinates
                    // may be negative when the content width exceeds the diagram dimension, and
                    // starts outside the diagram bounds.
                    align_items: Some(AlignItems::FlexStart),
                    align_content: Some(AlignContent::Start),
                    justify_items: Some(JustifyItems::Start),
                    justify_content: Some(JustifyContent::Start),
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
                        align_content: Some(AlignContent::FlexStart),
                        justify_items: Some(JustifyItems::FlexStart),
                        justify_content: Some(JustifyContent::FlexStart),
                        // Gap between the text node and the child container node.
                        gap: Size::length(flex_layout.gap()),
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
                        // Rank sub-containers must not shrink below their
                        // content size; otherwise the column wrapper parent
                        // compresses them when space is tight, causing wrapped
                        // rows to overlap with the next rank container.
                        flex_shrink: 0.0,
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
        taffy_node_ctx: Option<&mut TaffyNodeCtx>,
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
            edge_labels,
            edge_id_to_endpoint_node_ids,
            char_width,
            lod,
        } = node_measure_context;

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
                            let node_desc = entity_descs.get(entity_id).map(String::as_str);

                            match node_desc {
                                Some(desc) => Some(Cow::Owned(format!("# {node_name}\n\n{desc}"))),
                                None => Some(Cow::Borrowed(node_name)),
                            }
                        }
                    }
                }
                TaffyNodeCtx::EdgeSpacer(_) => None,
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

    /// Builds the `edge_label_taffy_nodes` map by merging per-node label
    /// leaves collected during envelope construction.
    ///
    /// For each [`EdgeLabelLeafBuilt`], the raw edge endpoints are looked up
    /// via `edge_groups` and compared against the leaf's `node_id` to
    /// determine whether the leaf is the `from` or `to` slot for that edge.
    /// Self-loop edges (where `from == to`) use only a `from_label` slot;
    /// their `to_face` is `None`, so the `to_label` slot is never populated.
    fn edge_label_taffy_nodes_build(
        edge_label_leaves: Vec<EdgeLabelLeafBuilt>,
        edge_face_assignments: &EdgeFaceAssignments<'static>,
        edge_groups: &EdgeGroups<'static>,
    ) -> Map<EdgeId<'static>, EdgeLabelTaffyNodeIds> {
        let edge_id_to_node_ids = Self::edge_id_to_node_ids_build(edge_groups);

        let mut edge_label_taffy_nodes: Map<EdgeId<'static>, EdgeLabelTaffyNodeIds> = Map::new();
        for built in edge_label_leaves {
            let Some((from_node_id, to_node_id)) = edge_id_to_node_ids.get(&built.edge_id) else {
                continue;
            };

            // Only create an entry when there is a face assignment to populate.
            let Some(assignment) = edge_face_assignments.get(&built.edge_id) else {
                continue;
            };

            let entry =
                edge_label_taffy_nodes
                    .entry(built.edge_id)
                    .or_insert(EdgeLabelTaffyNodeIds {
                        from_label_taffy_node_id: None,
                        to_label_taffy_node_id: None,
                    });

            if &built.node_id == from_node_id && assignment.from_face.is_some() {
                entry.from_label_taffy_node_id = Some(built.taffy_node_id);
            }
            if &built.node_id == to_node_id && assignment.to_face.is_some() {
                entry.to_label_taffy_node_id = Some(built.taffy_node_id);
            }
        }

        edge_label_taffy_nodes
    }

    /// Builds a lookup from each edge ID to the node IDs of its endpoints.
    ///
    /// The edge ID format mirrors `NodeFaceEdges::edge_id_generate`:
    /// `"{edge_group_id}__{edge_index}"`.
    fn edge_id_to_node_ids_build(
        edge_groups: &EdgeGroups<'static>,
    ) -> Map<EdgeId<'static>, (NodeId<'static>, NodeId<'static>)> {
        edge_groups
            .iter()
            .flat_map(|(edge_group_id, edge_group)| {
                edge_group
                    .iter()
                    .enumerate()
                    .map(|(edge_index, edge)| {
                        let edge_id_str = format!("{edge_group_id}__{edge_index}");
                        let edge_id: EdgeId<'static> = Id::try_from(edge_id_str)
                            .expect("edge group ID and index should produce a valid edge ID")
                            .into();
                        (edge_id, (edge.from.clone(), edge.to.clone()))
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Builds an envelope taffy node around `diagram_node_wrapper_node`.
    ///
    /// The envelope adds flex-row/column slots for edge label leaf nodes on
    /// each face of the diagram node. The structure is:
    ///
    /// ```text
    /// envelope_node:               (flex column, align_items: Stretch)
    ///   edge_wrapper_top:          (flex row)
    ///   edge_and_diagram_wrapper:  (flex row, align_items: Stretch)
    ///     edge_wrapper_left:       (flex column)
    ///     diagram_node_wrapper_node
    ///     edge_wrapper_right:      (flex column)
    ///   edge_wrapper_bottom:       (flex row)
    /// ```
    ///
    /// # Parameters
    ///
    /// * `taffy_tree`: The `TaffyTree` to insert nodes into.
    /// * `node_id`: The diagram node ID for this envelope.
    /// * `diagram_node_wrapper_node`: The existing wrapper node taffy ID to
    ///   wrap.
    /// * `node_face_edges`: The per-node face-to-edge-IDs mapping from
    ///   `IrDiagram`.
    fn taffy_envelope_node_build(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        node_id: &NodeId<'static>,
        diagram_node_wrapper_node: taffy::NodeId,
        node_face_edges: &NodeFaceEdges<'static>,
        rank_dir: RankDir,
    ) -> (taffy::NodeId, Vec<EdgeLabelLeafBuilt>) {
        let mut edge_label_leaves = Vec::new();

        let pad = LengthPercentage::length(EDGE_LABEL_PADDING_PX);
        let zero = LengthPercentage::length(0.0);

        // For Top/Bottom faces the edge contacts the left x edge (all rank
        // directions except BottomToTop) or the right x edge (BottomToTop).
        // Padding goes on the opposite side to avoid overlapping adjacent labels.
        let label_leaf_style_top_bottom = {
            let padding = match rank_dir {
                RankDir::BottomToTop => Rect {
                    left: pad,
                    right: zero,
                    top: zero,
                    bottom: zero,
                },
                RankDir::TopToBottom | RankDir::LeftToRight | RankDir::RightToLeft => Rect {
                    left: zero,
                    right: pad,
                    top: zero,
                    bottom: zero,
                },
            };
            Style {
                flex_shrink: 0.0,
                padding,
                ..Default::default()
            }
        };

        // For Left/Right faces the edge contacts the top y edge (all rank
        // directions except RightToLeft) or the bottom y edge (RightToLeft).
        // Padding goes on the opposite side.
        let label_leaf_style_left_right = {
            let padding = match rank_dir {
                RankDir::RightToLeft => Rect {
                    left: zero,
                    right: zero,
                    top: pad,
                    bottom: zero,
                },
                RankDir::LeftToRight | RankDir::TopToBottom | RankDir::BottomToTop => Rect {
                    left: zero,
                    right: zero,
                    top: zero,
                    bottom: pad,
                },
            };
            Style {
                flex_shrink: 0.0,
                padding,
                ..Default::default()
            }
        };

        let top_leaf_ids = Self::taffy_envelope_node_build_face_leaves(
            taffy_tree,
            node_id,
            NodeFace::Top,
            node_face_edges.edges_for(node_id, NodeFace::Top),
            &label_leaf_style_top_bottom,
            &mut edge_label_leaves,
        );
        let bottom_leaf_ids = Self::taffy_envelope_node_build_face_leaves(
            taffy_tree,
            node_id,
            NodeFace::Bottom,
            node_face_edges.edges_for(node_id, NodeFace::Bottom),
            &label_leaf_style_top_bottom,
            &mut edge_label_leaves,
        );
        let left_leaf_ids = Self::taffy_envelope_node_build_face_leaves(
            taffy_tree,
            node_id,
            NodeFace::Left,
            node_face_edges.edges_for(node_id, NodeFace::Left),
            &label_leaf_style_left_right,
            &mut edge_label_leaves,
        );
        let right_leaf_ids = Self::taffy_envelope_node_build_face_leaves(
            taffy_tree,
            node_id,
            NodeFace::Right,
            node_face_edges.edges_for(node_id, NodeFace::Right),
            &label_leaf_style_left_right,
            &mut edge_label_leaves,
        );

        // edge_wrapper_top: row 1, col 2 (top-middle cell of the 3x3 grid)
        let edge_wrapper_top = taffy_tree
            .new_with_children(
                Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    grid_row: line(1),
                    grid_column: line(2),
                    justify_content: Some(JustifyContent::SpaceEvenly),
                    padding: Rect {
                        left: NODE_SIDE_PADDING_PX,
                        right: NODE_SIDE_PADDING_PX,
                        top: zero,
                        bottom: zero,
                    },
                    ..Default::default()
                },
                &top_leaf_ids,
            )
            .unwrap_or_else(|e| {
                panic!("Expected to create edge_wrapper_top for {node_id}. Error: {e}")
            });

        // edge_wrapper_left: row 2, col 1 (left-middle cell of the 3x3 grid)
        let edge_wrapper_left = taffy_tree
            .new_with_children(
                Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    grid_row: line(2),
                    grid_column: line(1),
                    justify_content: Some(JustifyContent::SpaceEvenly),
                    padding: Rect {
                        left: zero,
                        right: zero,
                        top: NODE_SIDE_PADDING_PX,
                        bottom: NODE_SIDE_PADDING_PX,
                    },
                    ..Default::default()
                },
                &left_leaf_ids,
            )
            .unwrap_or_else(|e| {
                panic!("Expected to create edge_wrapper_left for {node_id}. Error: {e}")
            });

        // Place diagram_node_wrapper_node at row 2, col 2 (center cell of the 3x3
        // grid).
        let diagram_wrapper_style = taffy_tree
            .style(diagram_node_wrapper_node)
            .unwrap_or_else(|e| {
                panic!(
                    "Expected to get style of diagram_node_wrapper_node for {node_id}. \
                     Error: {e}"
                )
            })
            .clone();
        taffy_tree
            .set_style(
                diagram_node_wrapper_node,
                Style {
                    grid_row: line(2),
                    grid_column: line(2),
                    ..diagram_wrapper_style
                },
            )
            .unwrap_or_else(|e| {
                panic!(
                    "Expected to set grid placement on diagram_node_wrapper_node for \
                     {node_id}. Error: {e}"
                )
            });

        // edge_wrapper_right: row 2, col 3 (right-middle cell of the 3x3 grid)
        let edge_wrapper_right = taffy_tree
            .new_with_children(
                Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    grid_row: line(2),
                    grid_column: line(3),
                    justify_content: Some(JustifyContent::SpaceEvenly),
                    padding: Rect {
                        left: zero,
                        right: zero,
                        top: NODE_SIDE_PADDING_PX,
                        bottom: NODE_SIDE_PADDING_PX,
                    },
                    ..Default::default()
                },
                &right_leaf_ids,
            )
            .unwrap_or_else(|e| {
                panic!("Expected to create edge_wrapper_right for {node_id}. Error: {e}")
            });

        // edge_wrapper_bottom: row 3, col 2 (bottom-middle cell of the 3x3 grid)
        let edge_wrapper_bottom = taffy_tree
            .new_with_children(
                Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    grid_row: line(3),
                    grid_column: line(2),
                    justify_content: Some(JustifyContent::SpaceEvenly),
                    padding: Rect {
                        left: NODE_SIDE_PADDING_PX,
                        right: NODE_SIDE_PADDING_PX,
                        top: zero,
                        bottom: zero,
                    },
                    ..Default::default()
                },
                &bottom_leaf_ids,
            )
            .unwrap_or_else(|e| {
                panic!("Expected to create edge_wrapper_bottom for {node_id}. Error: {e}")
            });

        // envelope_node: 3x3 CSS Grid -- top-middle, left, center, right,
        // bottom-middle.  The four corner cells are left empty.  Column and
        // row track sizes are `auto` so each track sizes to its content; the
        // center cell stretches to fill the row/column size allocated by any
        // larger adjacent cell.
        let envelope_node = taffy_tree
            .new_with_children(
                Style {
                    display: Display::Grid,
                    grid_template_columns: vec![auto(), auto(), auto()],
                    grid_template_rows: vec![auto(), auto(), auto()],
                    ..Default::default()
                },
                &[
                    edge_wrapper_top,
                    edge_wrapper_left,
                    diagram_node_wrapper_node,
                    edge_wrapper_right,
                    edge_wrapper_bottom,
                ],
            )
            .unwrap_or_else(|e| {
                panic!("Expected to create envelope_node for {node_id}. Error: {e}")
            });

        (envelope_node, edge_label_leaves)
    }

    /// Builds the edge label leaf nodes for one face of an envelope node.
    ///
    /// For each edge ID in `edge_ids`, a leaf node is created with
    /// [`TaffyNodeCtx::EdgeLabel`] context and appended to `label_leaves`.
    /// Returns the `taffy::NodeId`s of all created leaves in order.
    ///
    /// # Parameters
    ///
    /// * `taffy_tree`: The `TaffyTree` to insert nodes into.
    /// * `node_id`: The diagram node ID that owns this face.
    /// * `face`: Which face of the node these labels are on.
    /// * `edge_ids`: The edge IDs that attach to `face` on `node_id`.
    /// * `label_leaf_style`: The taffy `Style` applied to every label leaf.
    /// * `label_leaves`: Output accumulator for the built leaves.
    fn taffy_envelope_node_build_face_leaves(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        node_id: &NodeId<'static>,
        face: NodeFace,
        edge_ids: &[EdgeId<'static>],
        label_leaf_style: &Style,
        label_leaves: &mut Vec<EdgeLabelLeafBuilt>,
    ) -> Vec<taffy::NodeId> {
        edge_ids
            .iter()
            .map(|edge_id| {
                let taffy_node_id = taffy_tree
                    .new_leaf_with_context(
                        label_leaf_style.clone(),
                        TaffyNodeCtx::EdgeLabel(EdgeLabelCtx {
                            edge_id: edge_id.clone(),
                            node_id: node_id.clone(),
                            face,
                        }),
                    )
                    .unwrap_or_else(|e| {
                        panic!(
                            "Expected to create edge label leaf for edge {edge_id} on \
                             face {face:?} of node {node_id}. Error: {e}"
                        )
                    });
                label_leaves.push(EdgeLabelLeafBuilt {
                    edge_id: edge_id.clone(),
                    node_id: node_id.clone(),
                    face,
                    taffy_node_id,
                });
                taffy_node_id
            })
            .collect()
    }
}
