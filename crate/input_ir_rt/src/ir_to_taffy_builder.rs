use disposition_ir_model::{
    edge::EdgeId,
    entity::EntityType,
    node::{NodeId, NodeInbuilt},
    IrDiagram,
};
use disposition_model_common::Map;
use disposition_taffy_model::{
    taffy::{self, AvailableSpace, Size, TaffyTree},
    DimensionAndLod, EdgeDescriptionTaffyNodes, EdgeSpacerTaffyNodes, IrToTaffyError,
    MdNodeTaffyIds, ProcessesIncluded, TaffyNodeKind, TaffyNodeMappings, TEXT_FONT_SIZE,
};
use typed_builder::TypedBuilder;

use crate::EdgeIdGenerator;

pub(crate) use self::edge_spacer_builder::LcaDepthCalculator;

use self::{
    edge_description_builder::{EdgeDescriptionBuildResult, EdgeDescriptionBuilder},
    edge_label_builder::EdgeLabelBuilder,
    edge_spacer_builder::EdgeSpacerBuilder,
    highlighted_spans_computer::HighlightedSpansComputer,
    md_spans_computer::MdSpansComputer,
    taffy_build_ctx::TaffyBuildCtx,
    taffy_build_state::TaffyBuildState,
    taffy_container_builder::TaffyContainerBuilder,
    taffy_diagram_node_builder::{FirstLevelNodesBuilt, TaffyDiagramNodeBuilder},
    taffy_node_build_context::NodeMeasureContext,
    text_measure::MONOSPACE_CHAR_WIDTH_RATIO,
};

mod edge_description_builder;
mod edge_label_builder;
mod edge_spacer_builder;
mod highlighted_spans_computer;
mod md_node_builder;
mod md_spans_computer;
mod rank_and_sibling_index_middle;
mod rank_sibling_inserter;
mod taffy_build_ctx;
mod taffy_build_state;
mod taffy_container_builder;
mod taffy_diagram_node_builder;
mod taffy_envelope_builder;
mod taffy_node_build_context;
mod text_measure;

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
            edge_route_reversals: _,
            thing_descs,
            thing_layout_edges: _,
            edge_descs,
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
            process_step_edges: _,
            process_step_ranks,
            process_step_graphs,
            render_options,
            css: _,
            interaction_edge_halo,
        } = ir_diagram;

        let DimensionAndLod { dimension, lod } = dimension_and_lod;

        let mut taffy_tree = TaffyTree::new();
        let mut node_id_to_taffy = Map::new();
        let mut taffy_id_to_node = Map::new();
        let mut node_id_to_envelope_taffy_node: Map<NodeId<'static>, taffy::NodeId> = Map::new();
        let mut taffy_id_to_kind: Map<taffy::NodeId, TaffyNodeKind<'static>> = Map::new();
        let mut edge_label_leaf_builts = Vec::new();

        // Precompute monospace character width
        let char_width = TEXT_FONT_SIZE * MONOSPACE_CHAR_WIDTH_RATIO;

        let mut md_node_taffy_ids: Map<NodeId<'static>, MdNodeTaffyIds> = Map::new();

        // Precompute markdown / text content per node once, so the
        // node-building, size-measuring, and highlighted-span passes all share
        // the same text.
        let node_md_texts = TaffyBuildCtx::node_md_texts_compute(nodes, thing_descs, *lod);

        // Precompute the edge ID -> endpoint node IDs lookup once. It is used
        // both when building edge label markdown sub-trees during envelope
        // construction and when sizing edge label slots during layout.
        let edge_id_to_endpoint_node_ids = EdgeLabelBuilder::edge_id_to_node_ids_build(edge_groups);

        // Precompute the edge ID -> edge group ID lookup once. It is used to
        // resolve the group-ID fallback when looking up `edge_descs` /
        // `edge_labels` for a specific edge instance.
        let edge_id_to_group_id = EdgeIdGenerator::edge_id_to_group_id_build(edge_groups);

        let ctx = TaffyBuildCtx {
            node_layouts,
            node_hierarchy,
            entity_types,
            edge_descs,
            node_shapes,
            node_ranks_nested,
            process_step_ranks,
            process_step_graphs,
            node_nesting_infos,
            node_face_edges,
            edge_groups,
            edge_labels,
            edge_id_to_endpoint_node_ids: &edge_id_to_endpoint_node_ids,
            edge_id_to_group_id: &edge_id_to_group_id,
            render_options,
            lod: *lod,
            char_width,
            node_md_texts: &node_md_texts,
            interaction_edge_halo_stroke_width: interaction_edge_halo.stroke_width,
        };

        let FirstLevelNodesBuilt {
            entity_type_to_rank_nodes: node_rank_to_nodes_by_entity_type,
            nested_edge_taffy_nodes: first_level_nested_edge_taffy_nodes,
        } = {
            let mut state = TaffyBuildState {
                taffy_tree: &mut taffy_tree,
                node_id_to_taffy: &mut node_id_to_taffy,
                taffy_id_to_node: &mut taffy_id_to_node,
                taffy_id_to_kind: &mut taffy_id_to_kind,
                node_id_to_envelope_taffy_node: &mut node_id_to_envelope_taffy_node,
                edge_label_leaf_builts: &mut edge_label_leaf_builts,
                md_node_taffy_ids: &mut md_node_taffy_ids,
            };
            TaffyDiagramNodeBuilder::build_first_level_nodes(ctx, &mut state, processes_included)
        };
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
        // Merge field-by-field per edge: an edge may have cross-container
        // spacers bubbled up from `first_level_nested_edge_taffy_nodes` *and* a
        // top-level rank-based (LCA-gap) spacer built here. A plain
        // `Map::extend` would drop one set; `EdgeSpacerTaffyNodes::map_merge`
        // keeps both.
        let mut edge_spacer_taffy_nodes: Map<EdgeId<'static>, EdgeSpacerTaffyNodes> = Map::new();
        EdgeSpacerTaffyNodes::map_merge(
            &mut edge_spacer_taffy_nodes,
            first_level_nested_edge_taffy_nodes.edge_spacer_taffy_nodes,
        );
        EdgeSpacerTaffyNodes::map_merge(
            &mut edge_spacer_taffy_nodes,
            EdgeSpacerBuilder::build(
                ctx,
                &mut taffy_tree,
                &EntityType::ThingDefault,
                &mut thing_rank_to_taffy_ids,
                None,
            ),
        );
        EdgeSpacerTaffyNodes::map_merge(
            &mut edge_spacer_taffy_nodes,
            EdgeSpacerBuilder::build(
                ctx,
                &mut taffy_tree,
                &EntityType::TagDefault,
                &mut tag_rank_to_taffy_ids,
                None,
            ),
        );
        EdgeSpacerTaffyNodes::map_merge(
            &mut edge_spacer_taffy_nodes,
            EdgeSpacerBuilder::build(
                ctx,
                &mut taffy_tree,
                &EntityType::ProcessDefault,
                &mut process_rank_to_taffy_ids,
                None,
            ),
        );

        // === Build edge_description_container nodes for top-level described edges ===
        // //
        //
        // For each described edge at the top level (LCA = root), we create a
        // container node interleaved between rank containers, plus a leaf node
        // for text measurement.
        let mut edge_description_taffy_nodes: Map<EdgeId<'static>, EdgeDescriptionTaffyNodes> =
            Map::new();
        edge_description_taffy_nodes
            .extend(first_level_nested_edge_taffy_nodes.edge_description_taffy_nodes);

        let thing_rank_container_style = TaffyContainerBuilder::taffy_container_style(
            node_layouts,
            &NodeInbuilt::ThingsContainer.id(),
            Size::auto(),
        );
        let tag_rank_container_style = TaffyContainerBuilder::taffy_container_style(
            node_layouts,
            &NodeInbuilt::TagsContainer.id(),
            Size::auto(),
        );
        let process_rank_container_style = TaffyContainerBuilder::taffy_container_style(
            node_layouts,
            &NodeInbuilt::ProcessesContainer.id(),
            Size::auto(),
        );

        let EdgeDescriptionBuildResult {
            edge_description_taffy_nodes: thing_edge_desc_taffy_nodes,
            position_to_container_ids: thing_position_to_container_ids,
            same_rank_position_to_container_ids: thing_same_rank_position_to_container_ids,
        } = EdgeDescriptionBuilder::build(
            ctx,
            &mut taffy_tree,
            &EntityType::ThingDefault,
            None,
            &thing_rank_container_style,
            &mut thing_rank_to_taffy_ids,
        );
        let EdgeDescriptionBuildResult {
            edge_description_taffy_nodes: tag_edge_desc_taffy_nodes,
            position_to_container_ids: tag_position_to_container_ids,
            same_rank_position_to_container_ids: tag_same_rank_position_to_container_ids,
        } = EdgeDescriptionBuilder::build(
            ctx,
            &mut taffy_tree,
            &EntityType::TagDefault,
            None,
            &tag_rank_container_style,
            &mut tag_rank_to_taffy_ids,
        );
        let EdgeDescriptionBuildResult {
            edge_description_taffy_nodes: process_edge_desc_taffy_nodes,
            position_to_container_ids: process_position_to_container_ids,
            same_rank_position_to_container_ids: process_same_rank_position_to_container_ids,
        } = EdgeDescriptionBuilder::build(
            ctx,
            &mut taffy_tree,
            &EntityType::ProcessDefault,
            None,
            &process_rank_container_style,
            &mut process_rank_to_taffy_ids,
        );
        edge_description_taffy_nodes.extend(thing_edge_desc_taffy_nodes);
        edge_description_taffy_nodes.extend(tag_edge_desc_taffy_nodes);
        edge_description_taffy_nodes.extend(process_edge_desc_taffy_nodes);

        // === Build edge_description_container spacers === //
        //
        // For each edge that crosses through an edge_description_container at
        // the top level, insert a spacer inside that container so the edge
        // path can route around it. Must run before position_to_container_ids
        // is consumed by `rank_containers_for_first_level_nodes_build`.
        for (target_entity_type, position_to_container_ids, same_rank_position_to_container_ids) in [
            (
                &EntityType::ThingDefault,
                &thing_position_to_container_ids,
                &thing_same_rank_position_to_container_ids,
            ),
            (
                &EntityType::TagDefault,
                &tag_position_to_container_ids,
                &tag_same_rank_position_to_container_ids,
            ),
            (
                &EntityType::ProcessDefault,
                &process_position_to_container_ids,
                &process_same_rank_position_to_container_ids,
            ),
        ] {
            for (edge_id, new_spacers) in EdgeSpacerBuilder::build_edge_desc_container_spacers(
                ctx,
                &mut taffy_tree,
                target_entity_type,
                None,
                position_to_container_ids,
                same_rank_position_to_container_ids,
                &edge_description_taffy_nodes,
            ) {
                edge_spacer_taffy_nodes
                    .entry(edge_id)
                    .or_default()
                    .merge(new_spacers);
            }
        }

        // Create rank sub-containers for top-level nodes, mirroring the
        // rank-based child container logic used inside
        // `TaffyDiagramNodeBuilder::build_node_with_child_hierarchy`.
        //
        // Each entity type gets its own set of rank containers using the
        // style of its parent container, interleaved with any
        // edge_description_containers at the same level.
        let thing_rank_container_ids =
            TaffyContainerBuilder::rank_containers_for_first_level_nodes_build(
                &mut taffy_tree,
                thing_rank_to_taffy_ids,
                thing_rank_container_style,
                thing_position_to_container_ids,
                NodeInbuilt::ThingsContainer,
                &mut taffy_id_to_kind,
            );
        let tag_rank_container_ids =
            TaffyContainerBuilder::rank_containers_for_first_level_nodes_build(
                &mut taffy_tree,
                tag_rank_to_taffy_ids,
                tag_rank_container_style,
                tag_position_to_container_ids,
                NodeInbuilt::TagsContainer,
                &mut taffy_id_to_kind,
            );
        let process_rank_container_ids =
            TaffyContainerBuilder::rank_containers_for_first_level_nodes_build(
                &mut taffy_tree,
                process_rank_to_taffy_ids,
                process_rank_container_style,
                process_position_to_container_ids,
                NodeInbuilt::ProcessesContainer,
                &mut taffy_id_to_kind,
            );

        let node_inbuilt_to_taffy = TaffyContainerBuilder::build(
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

        // Compute layout (size measurement only, no syntax highlighting)
        let mut node_measure_context = NodeMeasureContext {
            ctx,
            edge_labels,
            edge_id_to_endpoint_node_ids: &edge_id_to_endpoint_node_ids,
        };

        taffy_tree
            .compute_layout_with_measure(
                root,
                Size::<AvailableSpace> {
                    width: AvailableSpace::Definite(dimension.width()),
                    height: AvailableSpace::Definite(dimension.height()),
                },
                |known_dimensions, available_space, _taffy_node_id, taffy_node_ctx, style| {
                    node_measure_context.size_measure(
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
        let edge_label_taffy_nodes =
            EdgeLabelBuilder::build(edge_label_leaf_builts, edge_face_assignments, edge_groups);

        // Compute highlighted spans *after* layout is complete, once per node
        // instead of multiple times during layout measurement.
        let edge_description_highlighted_spans =
            HighlightedSpansComputer::compute_edge_desc_containers(
                &taffy_tree,
                &edge_description_taffy_nodes,
                edge_descs,
                &edge_id_to_group_id,
                char_width,
                lod,
            );

        // Compute highlighted text spans and image spans for diagram nodes
        // after layout is complete. All node text is built via the markdown
        // content path, so every node's spans are produced by `MdSpansComputer`.
        let (mut entity_highlighted_spans, mut entity_image_spans) = MdSpansComputer::compute(
            &taffy_tree,
            &node_id_to_taffy,
            &md_node_taffy_ids,
            char_width,
        );

        // Compute highlighted text spans and image spans for edge labels that
        // used the markdown path (`DiagramLod::Normal` with non-empty label
        // text). The text spans are keyed by `{edge_id}__from_label` /
        // `{edge_id}__to_label` and merged into `entity_highlighted_spans`;
        // the image spans are keyed by the same label key and merged into
        // `entity_image_spans`.
        let (md_edge_label_spans, md_edge_label_image_spans) =
            MdSpansComputer::compute_edge_labels(&taffy_tree, &edge_label_taffy_nodes, char_width);
        for (label_key, spans) in md_edge_label_spans.into_inner() {
            entity_highlighted_spans.insert(label_key, spans);
        }
        for (label_key, image_spans) in md_edge_label_image_spans {
            entity_image_spans.insert(label_key, image_spans);
        }

        // Compute highlighted text spans and image spans for edge descriptions
        // that used the markdown path.
        //
        // Edge descriptions with markdown are skipped by
        // `HighlightedSpansComputer::compute_edge_desc_containers` (their
        // `md_node_taffy_ids` is `Some`), so their spans must be computed
        // separately here and merged in.
        let (md_edge_desc_spans, edge_description_image_spans) =
            MdSpansComputer::compute_edge_descs(
                &taffy_tree,
                &edge_description_taffy_nodes,
                char_width,
            );

        let mut edge_description_highlighted_spans = edge_description_highlighted_spans;
        for (edge_id, spans) in md_edge_desc_spans {
            edge_description_highlighted_spans.insert(edge_id, spans);
        }

        std::iter::once(TaffyNodeMappings {
            taffy_tree,
            node_inbuilt_to_taffy,
            node_id_to_taffy,
            taffy_id_to_node,
            taffy_id_to_kind,
            edge_spacer_taffy_nodes,
            entity_highlighted_spans,
            edge_label_taffy_nodes,
            edge_description_taffy_nodes,
            edge_description_highlighted_spans,
            node_id_to_envelope_taffy_node,
            md_node_taffy_ids,
            entity_image_spans,
            edge_description_image_spans,
        })
    }
}
