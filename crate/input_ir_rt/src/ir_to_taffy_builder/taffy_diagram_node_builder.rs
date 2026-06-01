use std::collections::BTreeMap;

use disposition_ir_model::{
    edge::EdgeId,
    entity::EntityType,
    node::{NodeHierarchy, NodeId, NodeRank, NodeShape},
    process::{ProcessStepGraph, ProcessStepLane},
};
use disposition_model_common::{Id, Map};
use disposition_taffy_model::{
    taffy::{self, AlignItems, Display, FlexDirection, LengthPercentageAuto, Rect, Size, Style},
    DiagramLod, DiagramNodeCtx, EdgeDescriptionTaffyNodes, EdgeSpacerTaffyNodes,
    NodeToTaffyNodeIds, ProcessesIncluded, TaffyNodeCtx, LANE_WIDTH,
};

use super::{
    edge_description_builder::EdgeDescriptionBuilder,
    edge_spacer_builder::EdgeSpacerBuilder,
    md_node_builder::MdNodeBuilder,
    taffy_build_ctx::TaffyBuildCtx,
    taffy_build_state::TaffyBuildState,
    taffy_container_builder::{NodeRankToTaffyNodeId, TaffyContainerBuilder},
    taffy_envelope_builder::TaffyEnvelopeBuilder,
    taffy_node_build_context::TaffyWrapperNodeStyles,
};
use crate::md_text::md_blocks_parser::MdBlocksParser;

/// Builds taffy nodes for diagram nodes, handling both leaf nodes (no children)
/// and container nodes (with child hierarchies), grouping children by rank.
pub(crate) struct TaffyDiagramNodeBuilder;

impl TaffyDiagramNodeBuilder {
    /// Builds taffy nodes for all first-level nodes in the diagram, grouped by
    /// entity type and rank.
    pub(crate) fn build_first_level_nodes(
        ctx: TaffyBuildCtx<'_>,
        state: &mut TaffyBuildState<'_>,
        processes_included: &ProcessesIncluded,
    ) -> (
        Map<EntityType, NodeRankToTaffyNodeId>,
        Map<EdgeId<'static>, EdgeSpacerTaffyNodes>,
        Map<EdgeId<'static>, EdgeDescriptionTaffyNodes>,
    ) {
        let mut edge_spacer_taffy_nodes: Map<EdgeId<'static>, EdgeSpacerTaffyNodes> = Map::new();
        let mut edge_description_taffy_nodes: Map<EdgeId<'static>, EdgeDescriptionTaffyNodes> =
            Map::new();

        let entity_type_to_node_rank_to_taffy_node_ids = ctx.node_hierarchy.iter().fold(
            Map::<EntityType, NodeRankToTaffyNodeId>::new(),
            |mut entity_type_to_node_rank_to_taffy_node_ids, (node_id, child_hierarchy)| {
                let node_id: &Id = node_id.as_ref();
                let Some(entity_type) = ctx
                    .entity_types
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
                    Self::build_node_without_child_hierarchy(ctx, state, node_id, entity_type)
                } else {
                    let (
                        wrapper_node_id,
                        nested_edge_spacer_taffy_nodes,
                        nested_edge_description_taffy_nodes,
                    ) = Self::build_node_with_child_hierarchy(
                        ctx,
                        state,
                        child_hierarchy,
                        node_id,
                        entity_type,
                    );
                    edge_spacer_taffy_nodes.extend(nested_edge_spacer_taffy_nodes);
                    edge_description_taffy_nodes.extend(nested_edge_description_taffy_nodes);
                    wrapper_node_id
                };

                let ir_node_id = NodeId::from(node_id.clone());
                let rank = ctx
                    .node_ranks_nested
                    .node_rank_for(&ir_node_id, ctx.node_nesting_infos)
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
            edge_description_taffy_nodes,
        )
    }

    /// Adds the child taffy nodes for a given IR diagram node, grouped by rank.
    ///
    /// Returns a `BTreeMap` from `NodeRank` to the list of taffy node IDs at
    /// that rank. This allows the caller to create separate child containers
    /// for each rank level.
    pub(crate) fn build_child_nodes_by_rank(
        ctx: TaffyBuildCtx<'_>,
        state: &mut TaffyBuildState<'_>,
    ) -> (
        NodeRankToTaffyNodeId,
        Map<EdgeId<'static>, EdgeSpacerTaffyNodes>,
        Map<EdgeId<'static>, EdgeDescriptionTaffyNodes>,
    ) {
        let mut rank_to_taffy_ids: NodeRankToTaffyNodeId = BTreeMap::new();
        let mut edge_spacer_taffy_nodes: Map<EdgeId<'static>, EdgeSpacerTaffyNodes> = Map::new();
        let mut edge_description_taffy_nodes: Map<EdgeId<'static>, EdgeDescriptionTaffyNodes> =
            Map::new();

        for (node_id, child_hierarchy) in ctx.node_hierarchy.iter() {
            let node_id: &Id = node_id.as_ref();
            let Some(entity_type) = ctx
                .entity_types
                .get(node_id)
                .and_then(|entity_types| entity_types.first())
            else {
                // Skip nodes without an entity type -- probably something extra in the
                // hierarchy without a node name.
                continue;
            };

            let taffy_node_id = if child_hierarchy.is_empty() {
                Self::build_node_without_child_hierarchy(ctx, state, node_id, entity_type)
            } else {
                let (
                    wrapper_node_id,
                    nested_edge_spacer_taffy_nodes,
                    nested_edge_description_taffy_nodes,
                ) = Self::build_node_with_child_hierarchy(
                    ctx,
                    state,
                    child_hierarchy,
                    node_id,
                    entity_type,
                );
                edge_spacer_taffy_nodes.extend(nested_edge_spacer_taffy_nodes);
                edge_description_taffy_nodes.extend(nested_edge_description_taffy_nodes);
                wrapper_node_id
            };

            let ir_node_id = NodeId::from(node_id.clone());

            // Process steps are ordered by their process step rank (derived from
            // process step dependencies), with declaration order as the
            // tiebreaker within a rank. All other nodes use the hierarchy-aware
            // node ranks.
            let rank = if matches!(entity_type, EntityType::ProcessStepDefault) {
                ctx.process_step_ranks
                    .get(&ir_node_id)
                    .map(|process_step_rank| NodeRank::new(process_step_rank.value()))
                    .unwrap_or(NodeRank::new(0))
            } else {
                ctx.node_ranks_nested
                    .node_rank_for(&ir_node_id, ctx.node_nesting_infos)
                    .unwrap_or(NodeRank::new(0))
            };

            rank_to_taffy_ids
                .entry(rank)
                .or_default()
                .push(taffy_node_id);
        }

        (
            rank_to_taffy_ids,
            edge_spacer_taffy_nodes,
            edge_description_taffy_nodes,
        )
    }

    fn build_node_without_child_hierarchy(
        ctx: TaffyBuildCtx<'_>,
        state: &mut TaffyBuildState<'_>,
        node_id: &Id<'static>,
        entity_type: &EntityType,
    ) -> taffy::NodeId {
        let ir_node_id = NodeId::from(node_id.clone());
        let node_shape = ctx
            .node_shapes
            .get(&ir_node_id)
            .unwrap_or_else(|| panic!("There was no node shape for {ir_node_id}."));
        match node_shape {
            NodeShape::Rect(_node_shape_rect) => {
                let taffy_style = TaffyContainerBuilder::taffy_container_style(
                    ctx.node_layouts,
                    node_id,
                    Size::auto(),
                );
                let taffy_text_node_id = Self::text_leaf_build(
                    ctx,
                    state,
                    node_id,
                    entity_type,
                    &ir_node_id,
                    taffy_style,
                );

                state.node_id_to_taffy.insert(
                    ir_node_id.clone(),
                    NodeToTaffyNodeIds::Leaf {
                        text_node_id: taffy_text_node_id,
                    },
                );
                let (envelope_node_id, new_label_leaves) = TaffyEnvelopeBuilder::build(
                    state.taffy_tree,
                    &ir_node_id,
                    taffy_text_node_id,
                    ctx.node_face_edges,
                    ctx.rank_dir,
                );
                state.edge_label_leaves.extend(new_label_leaves);
                state
                    .node_id_to_envelope_taffy_node
                    .insert(ir_node_id.clone(), envelope_node_id);
                state
                    .taffy_id_to_node
                    .insert(taffy_text_node_id, ir_node_id);

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

                let circle_node_id = state
                    .taffy_tree
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
                let taffy_text_node_id = Self::text_leaf_build(
                    ctx,
                    state,
                    node_id,
                    entity_type,
                    &ir_node_id,
                    text_style,
                );

                let label_wrapper_style = TaffyContainerBuilder::taffy_container_style(
                    ctx.node_layouts,
                    node_id,
                    Size::auto(),
                );

                // Override to flex row for circle + text side by side
                let label_wrapper_style = Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: Some(AlignItems::Center),
                    gap: Size::length(4.0f32),
                    ..label_wrapper_style
                };

                let wrapper_node_id = state
                    .taffy_tree
                    .new_with_children(label_wrapper_style, &[circle_node_id, taffy_text_node_id])
                    .unwrap_or_else(|e| {
                        panic!("Expected to create label wrapper node for {node_id}. Error: {e}")
                    });

                state.node_id_to_taffy.insert(
                    ir_node_id.clone(),
                    NodeToTaffyNodeIds::LeafWithCircle {
                        wrapper_node_id,
                        circle_node_id,
                        text_node_id: taffy_text_node_id,
                    },
                );
                let (envelope_node_id, new_label_leaves) = TaffyEnvelopeBuilder::build(
                    state.taffy_tree,
                    &ir_node_id,
                    wrapper_node_id,
                    ctx.node_face_edges,
                    ctx.rank_dir,
                );
                state.edge_label_leaves.extend(new_label_leaves);
                state
                    .node_id_to_envelope_taffy_node
                    .insert(ir_node_id.clone(), envelope_node_id);
                state.taffy_id_to_node.insert(wrapper_node_id, ir_node_id);

                envelope_node_id
            }
        }
    }

    fn build_node_with_child_hierarchy(
        ctx: TaffyBuildCtx<'_>,
        state: &mut TaffyBuildState<'_>,
        child_hierarchy: &NodeHierarchy<'static>,
        node_id: &Id<'static>,
        entity_type: &EntityType,
    ) -> (
        taffy::NodeId,
        Map<EdgeId<'static>, EdgeSpacerTaffyNodes>,
        Map<EdgeId<'static>, EdgeDescriptionTaffyNodes>,
    ) {
        let ir_node_id = NodeId::from(node_id.clone());
        let mut edge_spacer_taffy_nodes: Map<EdgeId<'static>, EdgeSpacerTaffyNodes> = Map::new();
        let mut edge_description_taffy_nodes: Map<EdgeId<'static>, EdgeDescriptionTaffyNodes> =
            Map::new();

        // Processes with a git-graph layout lay their step circles out in lanes
        // instead of rank containers. Connectors between steps are drawn later by
        // `ProcessStepGraphEdgesBuilder`, so no edge spacers / descriptions are
        // produced here.
        if matches!(entity_type, EntityType::ProcessDefault)
            && let Some(process_step_graph) = ctx.process_step_graphs.get(&ir_node_id)
        {
            let envelope_node_id = Self::process_node_step_graph_build(
                ctx,
                state,
                node_id,
                &ir_node_id,
                entity_type,
                process_step_graph,
            );
            return (
                envelope_node_id,
                edge_spacer_taffy_nodes,
                edge_description_taffy_nodes,
            );
        }

        let TaffyWrapperNodeStyles {
            wrapper_style,
            text_style,
            child_container_style,
        } = TaffyContainerBuilder::taffy_wrapper_node_styles(ctx.node_layouts, node_id);
        let taffy_text_node_id =
            Self::text_leaf_build(ctx, state, node_id, entity_type, &ir_node_id, text_style);

        // Build the child nodes within this container's own hierarchy.
        let child_ctx = TaffyBuildCtx {
            node_hierarchy: child_hierarchy,
            ..ctx
        };
        let (
            mut rank_to_taffy_ids,
            nested_edge_spacer_taffy_nodes,
            nested_edge_description_taffy_nodes,
        ) = Self::build_child_nodes_by_rank(child_ctx, state);
        edge_spacer_taffy_nodes.extend(nested_edge_spacer_taffy_nodes);
        edge_description_taffy_nodes.extend(nested_edge_description_taffy_nodes);

        // === Insert spacer nodes for edges nested within this node === //
        let lca_node_id = NodeId::from(node_id.clone());
        for target_entity_type in &[
            EntityType::ThingDefault,
            EntityType::TagDefault,
            EntityType::ProcessDefault,
        ] {
            edge_spacer_taffy_nodes.extend(EdgeSpacerBuilder::build(
                ctx,
                state.taffy_tree,
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
            ctx,
            state.taffy_tree,
            &mut rank_to_taffy_ids,
            &ir_node_id,
            child_hierarchy,
        ));

        // === Build edge_description_container nodes for described edges at this
        // === level === //
        let mut position_to_container_ids: BTreeMap<Option<NodeRank>, Vec<taffy::NodeId>> =
            BTreeMap::new();
        for target_entity_type in &[
            EntityType::ThingDefault,
            EntityType::TagDefault,
            EntityType::ProcessDefault,
        ] {
            let edge_description_containers_build_result = EdgeDescriptionBuilder::build(
                ctx,
                state.taffy_tree,
                target_entity_type,
                Some(&lca_node_id),
                &child_container_style,
            );
            edge_description_taffy_nodes
                .extend(edge_description_containers_build_result.edge_description_taffy_nodes);
            for (pos, containers) in
                edge_description_containers_build_result.position_to_container_ids
            {
                position_to_container_ids
                    .entry(pos)
                    .or_default()
                    .extend(containers);
            }
        }

        // === Build edge_description_container spacers === //
        //
        // For each edge that crosses through an edge_description_container at
        // this nesting level, insert a spacer inside that container. Must run
        // before position_to_container_ids is consumed by
        // `rank_containers_interleave`.
        for target_entity_type in &[
            EntityType::ThingDefault,
            EntityType::TagDefault,
            EntityType::ProcessDefault,
        ] {
            for (edge_id, new_spacers) in EdgeSpacerBuilder::build_edge_desc_container_spacers(
                ctx,
                state.taffy_tree,
                target_entity_type,
                Some(&lca_node_id),
                &position_to_container_ids,
                &edge_description_taffy_nodes,
            ) {
                edge_spacer_taffy_nodes
                    .entry(edge_id)
                    .or_default()
                    .edge_desc_container_spacer_taffy_node_ids
                    .extend(new_spacers.edge_desc_container_spacer_taffy_node_ids);
            }
        }

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
        let rank_to_container: BTreeMap<NodeRank, taffy::NodeId> = rank_to_taffy_ids
            .into_iter()
            .map(|(rank, taffy_ids)| {
                let container = state
                    .taffy_tree
                    .new_with_children(child_container_style.clone(), &taffy_ids)
                    .unwrap_or_else(|e| {
                        panic!(
                            "Expected to create rank child container node for {node_id}. \
                             Error: {e}"
                        )
                    });
                (rank, container)
            })
            .collect();
        let rank_container_ids = TaffyContainerBuilder::rank_containers_interleave(
            rank_to_container,
            position_to_container_ids,
        );

        let node_shape = ctx
            .node_shapes
            .get(&ir_node_id)
            .unwrap_or_else(|| panic!("There was no node shape for {ir_node_id}."));

        match node_shape {
            NodeShape::Rect(_node_shape_rect) => {
                let mut wrapper_children = vec![taffy_text_node_id];
                wrapper_children.extend(rank_container_ids);

                let wrapper_node_id = state
                    .taffy_tree
                    .new_with_children(wrapper_style, &wrapper_children)
                    .unwrap_or_else(|e| {
                        panic!("Expected to create wrapper node for {node_id}. Error: {e}")
                    });

                state.node_id_to_taffy.insert(
                    ir_node_id.clone(),
                    NodeToTaffyNodeIds::Wrapper {
                        wrapper_node_id,
                        text_node_id: taffy_text_node_id,
                    },
                );
                let (envelope_node_id, new_label_leaves) = TaffyEnvelopeBuilder::build(
                    state.taffy_tree,
                    &ir_node_id,
                    wrapper_node_id,
                    ctx.node_face_edges,
                    ctx.rank_dir,
                );
                state.edge_label_leaves.extend(new_label_leaves);
                state
                    .node_id_to_envelope_taffy_node
                    .insert(ir_node_id.clone(), envelope_node_id);
                state.taffy_id_to_node.insert(wrapper_node_id, ir_node_id);

                (
                    envelope_node_id,
                    edge_spacer_taffy_nodes,
                    edge_description_taffy_nodes,
                )
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

                let circle_node_id = state
                    .taffy_tree
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

                let label_wrapper_node_id = state
                    .taffy_tree
                    .new_with_children(label_wrapper_style, &[circle_node_id, taffy_text_node_id])
                    .unwrap_or_else(|e| {
                        panic!("Expected to create label wrapper node for {node_id}. Error: {e}")
                    });

                let mut wrapper_children = vec![label_wrapper_node_id];
                wrapper_children.extend(rank_container_ids);

                let wrapper_node_id = state
                    .taffy_tree
                    .new_with_children(wrapper_style, &wrapper_children)
                    .unwrap_or_else(|e| {
                        panic!("Expected to create wrapper node for {node_id}. Error: {e}")
                    });

                state.node_id_to_taffy.insert(
                    ir_node_id.clone(),
                    NodeToTaffyNodeIds::WrapperCircle {
                        wrapper_node_id,
                        label_wrapper_node_id,
                        circle_node_id,
                        text_node_id: taffy_text_node_id,
                    },
                );
                let (envelope_node_id, new_label_leaves) = TaffyEnvelopeBuilder::build(
                    state.taffy_tree,
                    &ir_node_id,
                    wrapper_node_id,
                    ctx.node_face_edges,
                    ctx.rank_dir,
                );
                state.edge_label_leaves.extend(new_label_leaves);
                state
                    .node_id_to_envelope_taffy_node
                    .insert(ir_node_id.clone(), envelope_node_id);
                state.taffy_id_to_node.insert(wrapper_node_id, ir_node_id);

                (
                    envelope_node_id,
                    edge_spacer_taffy_nodes,
                    edge_description_taffy_nodes,
                )
            }
        }
    }

    /// Builds a process node whose steps use the git-graph lane layout.
    ///
    /// The process `wrapper_node` is a flex column holding the process label
    /// followed by one step row per step (ordered by the step's row in the
    /// graph). Each step row places the step circle in its lane and the step
    /// label in a shared, left-aligned text column. Returns the envelope node
    /// wrapping the process.
    fn process_node_step_graph_build(
        ctx: TaffyBuildCtx<'_>,
        state: &mut TaffyBuildState<'_>,
        node_id: &Id<'static>,
        ir_node_id: &NodeId<'static>,
        entity_type: &EntityType,
        process_step_graph: &ProcessStepGraph<'static>,
    ) -> taffy::NodeId {
        let TaffyWrapperNodeStyles {
            wrapper_style,
            text_style,
            child_container_style: _,
        } = TaffyContainerBuilder::taffy_wrapper_node_styles(ctx.node_layouts, node_id);

        // Process label.
        let process_text_node_id =
            Self::text_leaf_build(ctx, state, node_id, entity_type, ir_node_id, text_style);

        let lane_count = process_step_graph.lane_count;

        // Steps ordered by row (main-axis position).
        let mut steps: Vec<(&NodeId<'static>, u32)> = process_step_graph
            .step_placements
            .iter()
            .map(|(step_node_id, placement)| (step_node_id, placement.row))
            .collect();
        steps.sort_by_key(|(_step_node_id, row)| *row);

        let mut wrapper_children = vec![process_text_node_id];
        for (step_node_id, _row) in &steps {
            let lane = process_step_graph
                .step_placements
                .get(*step_node_id)
                .map(|placement| placement.lane)
                .unwrap_or_default();
            let step_wrapper_node_id =
                Self::process_step_graph_leaf_build(ctx, state, step_node_id, lane, lane_count);
            wrapper_children.push(step_wrapper_node_id);
        }

        let wrapper_node_id = state
            .taffy_tree
            .new_with_children(wrapper_style, &wrapper_children)
            .unwrap_or_else(|e| {
                panic!("Expected to create process wrapper node for {node_id}. Error: {e}")
            });

        state.node_id_to_taffy.insert(
            ir_node_id.clone(),
            NodeToTaffyNodeIds::Wrapper {
                wrapper_node_id,
                text_node_id: process_text_node_id,
            },
        );
        let (envelope_node_id, new_label_leaves) = TaffyEnvelopeBuilder::build(
            state.taffy_tree,
            ir_node_id,
            wrapper_node_id,
            ctx.node_face_edges,
            ctx.rank_dir,
        );
        state.edge_label_leaves.extend(new_label_leaves);
        state
            .node_id_to_envelope_taffy_node
            .insert(ir_node_id.clone(), envelope_node_id);
        state
            .taffy_id_to_node
            .insert(wrapper_node_id, ir_node_id.clone());

        envelope_node_id
    }

    /// Builds a single process step row for the git-graph layout.
    ///
    /// The row is a flex row of `[lane_gutter, text]`. The `lane_gutter` has a
    /// fixed width of `lane_count * LANE_WIDTH` and holds the step's circle
    /// offset to its lane, so the text always begins at the same x across steps
    /// (a single left-aligned column). Records the step as a
    /// [`NodeToTaffyNodeIds::LeafWithCircle`].
    fn process_step_graph_leaf_build(
        ctx: TaffyBuildCtx<'_>,
        state: &mut TaffyBuildState<'_>,
        step_node_id: &NodeId<'static>,
        lane: ProcessStepLane,
        lane_count: u32,
    ) -> taffy::NodeId {
        let step_id: &Id<'static> = step_node_id.as_ref();
        let entity_type = ctx
            .entity_types
            .get(step_id)
            .and_then(|entity_types| entity_types.first())
            .cloned()
            .unwrap_or(EntityType::ProcessStepDefault);

        let radius = match ctx.node_shapes.get(step_node_id) {
            Some(NodeShape::Circle(node_shape_circle)) => node_shape_circle.radius(),
            _ => LANE_WIDTH / 2.0 - 4.0,
        };
        let diameter = radius * 2.0;

        // Circle centred within its lane: intra-lane centring + lane offset.
        let circle_margin_left = ((LANE_WIDTH - diameter) / 2.0).max(0.0) + lane.value() as f32 * LANE_WIDTH;
        let circle_node_id = state
            .taffy_tree
            .new_leaf(Style {
                size: Size {
                    width: taffy::style::Dimension::length(diameter),
                    height: taffy::style::Dimension::length(diameter),
                },
                flex_shrink: 0.0,
                margin: Rect {
                    left: LengthPercentageAuto::length(circle_margin_left),
                    right: LengthPercentageAuto::length(0.0),
                    top: LengthPercentageAuto::length(0.0),
                    bottom: LengthPercentageAuto::length(0.0),
                },
                ..Default::default()
            })
            .unwrap_or_else(|e| {
                panic!("Expected to create circle leaf node for {step_id}. Error: {e}")
            });

        // Fixed-width gutter holding the lane circle, so the text column aligns.
        let gutter_width = lane_count as f32 * LANE_WIDTH;
        let lane_gutter_node_id = state
            .taffy_tree
            .new_with_children(
                Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    flex_shrink: 0.0,
                    size: Size {
                        width: taffy::style::Dimension::length(gutter_width),
                        height: taffy::style::Dimension::auto(),
                    },
                    ..Default::default()
                },
                &[circle_node_id],
            )
            .unwrap_or_else(|e| {
                panic!("Expected to create lane gutter node for {step_id}. Error: {e}")
            });

        let taffy_text_node_id =
            Self::text_leaf_build(ctx, state, step_id, &entity_type, step_node_id, Style::default());

        let step_wrapper_node_id = state
            .taffy_tree
            .new_with_children(
                Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: Some(AlignItems::Center),
                    gap: Size::length(4.0f32),
                    ..Default::default()
                },
                &[lane_gutter_node_id, taffy_text_node_id],
            )
            .unwrap_or_else(|e| {
                panic!("Expected to create step wrapper node for {step_id}. Error: {e}")
            });

        state.node_id_to_taffy.insert(
            step_node_id.clone(),
            NodeToTaffyNodeIds::ProcessStepGraphLeaf {
                wrapper_node_id: step_wrapper_node_id,
                circle_node_id,
                text_node_id: taffy_text_node_id,
            },
        );
        state
            .taffy_id_to_node
            .insert(taffy_text_node_id, step_node_id.clone());

        step_wrapper_node_id
    }

    /// Creates the text leaf for a diagram node.
    ///
    /// For `DiagramLod::Normal` nodes with a description, builds a markdown
    /// content sub-tree via `MdNodeBuilder` and wraps it in `wrapper_style`
    /// to preserve padding and margin. Records the `MdNodeTaffyIds` in the
    /// accumulator.
    ///
    /// For all other cases, falls back to a single `TaffyNodeCtx::DiagramNode`
    /// leaf.
    fn text_leaf_build(
        ctx: TaffyBuildCtx<'_>,
        state: &mut TaffyBuildState<'_>,
        node_id: &Id<'static>,
        entity_type: &EntityType,
        ir_node_id: &NodeId<'static>,
        fallback_style: Style,
    ) -> taffy::NodeId {
        if ctx.lod == DiagramLod::Normal && ctx.thing_descs.get(ir_node_id.as_ref()).is_some() {
            let markdown = ctx
                .node_md_text(node_id)
                .unwrap_or_else(|| node_id.as_str());
            let blocks = MdBlocksParser::parse(markdown);
            let md_ids = MdNodeBuilder::build(state.taffy_tree, &blocks, ctx.char_width);
            let text_node_id = state
                .taffy_tree
                .new_with_children(fallback_style, &[md_ids.content_node_id])
                .expect("Expected to create markdown wrapper node");
            state.md_node_taffy_ids.insert(ir_node_id.clone(), md_ids);
            return text_node_id;
        }

        state
            .taffy_tree
            .new_leaf_with_context(
                fallback_style,
                TaffyNodeCtx::DiagramNode(DiagramNodeCtx {
                    entity_id: node_id.clone(),
                    entity_type: entity_type.clone(),
                }),
            )
            .unwrap_or_else(|e| {
                panic!("Expected to create text leaf node for {node_id}. Error: {e}")
            })
    }
}
