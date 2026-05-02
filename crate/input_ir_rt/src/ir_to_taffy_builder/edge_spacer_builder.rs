use std::collections::BTreeMap;

use disposition_ir_model::{
    edge::{Edge, EdgeGroups, EdgeId},
    entity::{EntityType, EntityTypes},
    node::{NodeHierarchy, NodeId, NodeNestingInfo, NodeNestingInfos, NodeRank, NodeRanksNested},
};
use disposition_model_common::{edge::EdgeGroupId, Map};
use disposition_taffy_model::{
    taffy::{self, Size, Style, TaffyTree},
    EdgeSpacerCtx, EdgeSpacerTaffyNodes, TaffyNodeCtx,
};
use taffy::AlignSelf;

use crate::EdgeIdGenerator;

pub use self::{
    edge_spacer_build_decider::EdgeSpacerBuildDecider,
    edge_spacer_build_decision::EdgeSpacerBuildDecision,
    edge_spacer_build_decision_build::EdgeSpacerBuildDecisionBuild,
    edge_spacer_build_decision_skip::EdgeSpacerBuildDecisionSkip,
    lca_depth_calculator::LcaDepthCalculator,
};

mod edge_spacer_build_decider;
mod edge_spacer_build_decision;
mod edge_spacer_build_decision_build;
mod edge_spacer_build_decision_skip;
mod lca_depth_calculator;

const EDGE_SPACER_LENGTH: f32 = 5.0;

/// Builds spacer taffy nodes for edges that cross multiple ranks.
///
/// When edges connect nodes at different ranks, spacer nodes are
/// inserted at each intermediate rank level. These spacer nodes
/// participate in the flex layout so that the edge path can be
/// routed through their positions, reducing the chance of edges
/// being drawn over other nodes.
pub(crate) struct EdgeSpacerBuilder;

// === EdgeSpacerBuilder === //

impl EdgeSpacerBuilder {
    /// Inserts spacer taffy nodes for all cross-rank edges.
    ///
    /// This modifies `rank_to_taffy_ids` in-place by inserting spacer
    /// leaf nodes at the computed positions within each intermediate rank.
    ///
    /// Returns a map from edge ID to the spacer taffy node IDs at each rank.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        edge_groups: &EdgeGroups<'static>,
        node_nesting_infos: &NodeNestingInfos<'static>,
        node_ranks_nested: &NodeRanksNested<'static>,
        entity_types: &EntityTypes<'static>,
        target_entity_type: &EntityType,
        rank_to_taffy_ids: &mut BTreeMap<NodeRank, Vec<taffy::NodeId>>,
        lca_node_id: Option<&NodeId<'static>>,
    ) -> Map<EdgeId<'static>, EdgeSpacerTaffyNodes> {
        // === Find cross-rank edges and compute spacer placements === //
        let mut edge_spacer_taffy_nodes: Map<EdgeId<'static>, EdgeSpacerTaffyNodes> = Map::new();

        // Track how many spacers have been inserted at each (rank, position)
        // so subsequent insertions account for prior spacers.
        let mut rank_spacer_counts: BTreeMap<NodeRank, Vec<usize>> = BTreeMap::new();

        edge_groups.iter().for_each(|(edge_group_id, edge_group)| {
            edge_group
                .iter()
                .enumerate()
                .for_each(|(edge_index, edge)| {
                    let edge_id = EdgeIdGenerator::generate(edge_group_id, edge_index);

                    let spacer_nodes = Self::edge_spacers_build(
                        taffy_tree,
                        edge,
                        &edge_id,
                        node_nesting_infos,
                        node_ranks_nested,
                        entity_types,
                        target_entity_type,
                        rank_to_taffy_ids,
                        &mut rank_spacer_counts,
                        lca_node_id,
                    );

                    if let Some(spacer_nodes) = spacer_nodes {
                        edge_spacer_taffy_nodes.insert(edge_id, spacer_nodes);
                    }
                });
        });

        edge_spacer_taffy_nodes
    }

    /// Inserts spacer taffy nodes for edges that cross container boundaries.
    ///
    /// When an edge has one endpoint inside a container and the other
    /// outside (or at a different nesting depth), the edge path may
    /// need to route alongside the container's children to reach the
    /// deeply nested endpoint. This method identifies such edges and
    /// inserts spacer nodes at the positions of intermediate sibling
    /// children within the container.
    ///
    /// # Parameters
    ///
    /// * `taffy_tree`: The taffy tree to insert spacer nodes into.
    /// * `edge_groups`: All edge groups in the diagram.
    /// * `node_nesting_infos`: The precomputed nesting info map for all nodes.
    /// * `node_ranks`: Node ranks for all nodes.
    /// * `container_node_id`: The ID of the node which is a parent of the node
    ///   that the edge is connected to.
    /// * `container_node_hierarchy`: The children of `container_node_id`.
    /// * `rank_to_taffy_ids`: Mutable reference to the container's
    ///   rank-to-taffy-node mapping, for inserting spacer nodes.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build_cross_container_spacers(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        edge_groups: &EdgeGroups<'static>,
        node_nesting_infos: &NodeNestingInfos<'static>,
        node_ranks_nested: &NodeRanksNested<'static>,
        rank_to_taffy_ids: &mut BTreeMap<NodeRank, Vec<taffy::NodeId>>,
        container_node_id: &NodeId<'static>,
        container_node_hierarchy: &NodeHierarchy<'static>,
    ) -> Map<EdgeId<'static>, EdgeSpacerTaffyNodes> {
        // Collect direct child IDs of this container.
        let container_node_direct_child_ids: Vec<NodeId<'static>> = container_node_hierarchy
            .iter()
            .map(|(child_id, _)| child_id.clone())
            .collect();

        if container_node_direct_child_ids.len() <= 1 {
            // No siblings to route around.
            return Map::new();
        }

        let mut edge_spacer_taffy_nodes: Map<EdgeId<'static>, EdgeSpacerTaffyNodes> = Map::new();
        let mut rank_spacer_counts: BTreeMap<NodeRank, Vec<usize>> = BTreeMap::new();

        edge_groups.iter().for_each(|(edge_group_id, edge_group)| {
            edge_group
                .iter()
                .enumerate()
                .for_each(|(edge_index, edge)| {
                    Self::build_cross_container_spacers_for_edge(
                        taffy_tree,
                        edge_group_id,
                        node_nesting_infos,
                        node_ranks_nested,
                        rank_to_taffy_ids,
                        container_node_id,
                        &container_node_direct_child_ids,
                        &mut edge_spacer_taffy_nodes,
                        &mut rank_spacer_counts,
                        edge_index,
                        edge,
                    )
                });
        });

        edge_spacer_taffy_nodes
    }

    /// Builds cross-container spacers for a single edge.
    ///
    /// # Parameters
    ///
    /// * `taffy_tree`: The taffy tree to insert spacer nodes into.
    /// * `edge_groups`: All edge groups in the diagram.
    /// * `node_nesting_infos`: The precomputed nesting info map for all nodes.
    /// * `node_ranks`: Node ranks for all nodes.
    /// * `rank_to_taffy_ids`: Mutable reference to the container's
    ///   rank-to-taffy-node mapping, for inserting spacer nodes.
    /// * `container_node_id`: The ID of the node which is a parent of the node
    ///   that the edge is connected to.
    /// * `container_node_direct_child_ids`: The children of
    ///   `container_node_id`.
    /// * `edge_spacer_taffy_nodes`: Map to keep track of the spacer taffy nodes
    ///   inserted for each edge.
    /// * `rank_spacer_counts`: Map to keep track of the number of spacers
    ///   inserted for each rank within an edge group.
    /// * `edge_index`: Index of the edge within its edge group.
    /// * `edge`: Edge to build spacers for.
    #[allow(clippy::too_many_arguments)]
    fn build_cross_container_spacers_for_edge<'id>(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        edge_group_id: &EdgeGroupId<'static>,
        node_nesting_infos: &NodeNestingInfos<'static>,
        node_ranks_nested: &NodeRanksNested<'static>,
        rank_to_taffy_ids: &mut BTreeMap<NodeRank, Vec<taffy::NodeId>>,
        container_node_id: &NodeId<'static>,
        container_node_direct_child_ids: &Vec<NodeId<'static>>,
        edge_spacer_taffy_nodes: &mut Map<EdgeId<'static>, EdgeSpacerTaffyNodes>,
        rank_spacer_counts: &mut BTreeMap<NodeRank, Vec<usize>>,
        edge_index: usize,
        edge: &Edge<'id>,
    ) {
        let edge_spacer_build_decision = EdgeSpacerBuildDecider::decide(
            node_nesting_infos,
            container_node_id,
            container_node_direct_child_ids,
            edge,
        );
        let node_id_of_container_direct_child_that_contains_edge = match edge_spacer_build_decision
        {
            EdgeSpacerBuildDecision::Skip(_edge_spacer_build_decision_skip) => return,
            EdgeSpacerBuildDecision::Build(EdgeSpacerBuildDecisionBuild { target_child_id }) => {
                target_child_id
            }
        };

        // Insert spacers alongside each sibling of the target child.
        let spacer_style = Style {
            min_size: Size {
                width: taffy::Dimension::length(EDGE_SPACER_LENGTH),
                height: taffy::Dimension::length(EDGE_SPACER_LENGTH),
            },
            align_self: Some(AlignSelf::Stretch),
            ..Default::default()
        };

        let edge_id = EdgeIdGenerator::generate(edge_group_id, edge_index);

        let mut spacer_taffy_nodes = EdgeSpacerTaffyNodes::new();

        container_node_direct_child_ids
            .iter()
            .for_each(|sibling_id| {
                if sibling_id == node_id_of_container_direct_child_that_contains_edge {
                    return;
                }

                // Direct children of the container node may still have spacers if they are on
                // different ranks.
                let sibling_rank = node_ranks_nested
                    .ranks_for(Some(container_node_id))
                    .and_then(|r| r.get(sibling_id).copied())
                    .unwrap_or(NodeRank::new(0));

                // Create the taffy spacer node.
                let spacer_taffy_node_id = taffy_tree
                    .new_leaf_with_context(
                        spacer_style.clone(),
                        TaffyNodeCtx::EdgeSpacer(EdgeSpacerCtx {
                            edge_id: edge_id.clone(),
                            rank: sibling_rank,
                        }),
                    )
                    .expect("Expected to create cross-container spacer leaf node.");

                // Insert into rank_to_taffy_ids at the sibling's rank.
                let taffy_ids = rank_to_taffy_ids.entry(sibling_rank).or_default();
                let spacer_counts = rank_spacer_counts.entry(sibling_rank).or_default();

                if spacer_counts.len() < taffy_ids.len() + 1 {
                    spacer_counts.resize(taffy_ids.len() + 1, 0);
                }

                // Place the spacer at the end of the rank's children.
                taffy_ids.push(spacer_taffy_node_id);

                spacer_taffy_nodes
                    .cross_container_spacer_taffy_node_ids
                    .push(spacer_taffy_node_id);
            });

        if !spacer_taffy_nodes
            .cross_container_spacer_taffy_node_ids
            .is_empty()
        {
            edge_spacer_taffy_nodes
                .entry(edge_id)
                .or_default()
                .cross_container_spacer_taffy_node_ids
                .extend(spacer_taffy_nodes.cross_container_spacer_taffy_node_ids);
        }
    }

    /// Builds spacer taffy nodes for a single edge if it crosses ranks.
    ///
    /// To determine whether an edge visually crosses ranks, we cannot
    /// simply compare the raw ranks of the edge's `from` and `to` nodes,
    /// because nested nodes may have ranks computed from their own
    /// incoming edges rather than from their position in the visual
    /// layout. Instead, we find the lowest common ancestor (LCA) of the
    /// two endpoints in the node hierarchy, then compare the ranks of the
    /// children-of-LCA on each side. Those "divergent ancestor" ranks
    /// reflect the actual visual rank rows that the edge must cross.
    ///
    /// Returns `None` if the edge does not visually cross ranks, or if
    /// the edge's LCA does not match the requested `lca_node_id`.
    #[allow(clippy::too_many_arguments)]
    fn edge_spacers_build(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        edge: &Edge<'static>,
        edge_id: &EdgeId<'static>,
        node_nesting_infos: &NodeNestingInfos<'static>,
        node_ranks_nested: &NodeRanksNested<'static>,
        entity_types: &EntityTypes<'static>,
        target_entity_type: &EntityType,
        rank_to_taffy_ids: &mut BTreeMap<NodeRank, Vec<taffy::NodeId>>,
        rank_spacer_counts: &mut BTreeMap<NodeRank, Vec<usize>>,
        lca_node_id: Option<&NodeId<'static>>,
    ) -> Option<EdgeSpacerTaffyNodes> {
        let nesting_info_from = node_nesting_infos.get(&edge.from)?;
        let nesting_info_to = node_nesting_infos.get(&edge.to)?;

        // === Check that the edge's top-level ancestors match the target entity type
        // === //
        let lca_depth = LcaDepthCalculator::calculate(nesting_info_from, nesting_info_to);
        let divergent_from = nesting_info_from.ancestor_chain.get(lca_depth)?;
        let divergent_to = nesting_info_to.ancestor_chain.get(lca_depth)?;

        let from_matches = entity_types
            .get(divergent_from.as_ref())
            .map(|types| types.contains(target_entity_type))
            .unwrap_or(false);
        let to_matches = entity_types
            .get(divergent_to.as_ref())
            .map(|types| types.contains(target_entity_type))
            .unwrap_or(false);
        if !from_matches || !to_matches {
            return None;
        }

        // === Find divergent ancestors and their ranks === //
        let (rank_low, rank_high) =
            Self::divergent_ancestor_ranks(nesting_info_from, nesting_info_to, node_ranks_nested)?;

        // Only insert spacers for edges crossing ranks.
        if rank_low == rank_high {
            return None;
        }

        // If there are no intermediate ranks, no spacers needed.
        if rank_high.value() - rank_low.value() <= 1 {
            return None;
        }

        // Filter by the requested LCA level.
        match lca_node_id {
            None => {
                // Top-level: only insert spacers when the LCA is the root.
                if lca_depth > 0 {
                    return None;
                }
            }
            Some(expected_lca_node_id) => {
                // Nested: only insert spacers when the LCA matches the
                // expected parent node.
                if lca_depth == 0 {
                    return None;
                }
                let lca_ancestor = nesting_info_from.ancestor_chain.get(lca_depth - 1);
                match lca_ancestor {
                    Some(lca_ancestor) if lca_ancestor == expected_lca_node_id => {}
                    _ => return None,
                }
            }
        }

        // Compute the insertion index based on nesting info.
        let insertion_base_index =
            Self::insertion_base_index_compute(nesting_info_from, nesting_info_to);

        let spacer_style = Style {
            min_size: Size {
                width: taffy::Dimension::length(EDGE_SPACER_LENGTH),
                height: taffy::Dimension::length(EDGE_SPACER_LENGTH),
            },
            align_self: Some(AlignSelf::Stretch),
            ..Default::default()
        };

        let mut spacer_taffy_nodes = EdgeSpacerTaffyNodes::new();

        // Insert spacers at each intermediate rank exclusive of endpoints (low and high
        // rank both exclusive).
        let rank_low_plus_one = rank_low.value() + 1;
        (rank_low_plus_one..rank_high.value()).for_each(|rank_value| {
            let rank = NodeRank::new(rank_value);

            let spacer_taffy_node_id = taffy_tree
                .new_leaf_with_context(
                    spacer_style.clone(),
                    TaffyNodeCtx::EdgeSpacer(EdgeSpacerCtx {
                        edge_id: edge_id.clone(),
                        rank,
                    }),
                )
                .expect("Expected to create spacer leaf node.");

            // Determine actual insertion index accounting for existing spacers.
            let taffy_ids = rank_to_taffy_ids.entry(rank).or_default();
            let spacer_counts = rank_spacer_counts.entry(rank).or_default();

            // Ensure spacer_counts has enough entries.
            if spacer_counts.len() < taffy_ids.len() + 1 {
                spacer_counts.resize(taffy_ids.len() + 1, 0);
            }

            // The effective index accounts for previously inserted spacers.
            let effective_index = Self::effective_insertion_index(
                insertion_base_index,
                taffy_ids.len(),
                spacer_counts,
            );

            // Insert the spacer.
            if effective_index >= taffy_ids.len() {
                taffy_ids.push(spacer_taffy_node_id);
            } else {
                taffy_ids.insert(effective_index, spacer_taffy_node_id);
            }

            // Update spacer counts: increment count at this position.
            if spacer_counts.len() <= effective_index {
                spacer_counts.resize(effective_index + 1, 0);
            }
            spacer_counts.insert(effective_index, 1);

            spacer_taffy_nodes
                .rank_to_spacer_taffy_node_id
                .insert(rank, spacer_taffy_node_id);
        });

        Some(spacer_taffy_nodes)
    }

    // === Ancestor chain and LCA === //

    /// Finds the ranks of the "divergent ancestors" for an edge's two
    /// endpoints.
    ///
    /// The divergent ancestors are the first nodes in each endpoint's
    /// ancestor chain where the chains differ. Their ranks determine the
    /// visual rank span that the edge crosses.
    ///
    /// For example, given:
    ///
    /// ```text
    /// t_a0 (rank 0):
    ///   t_a01 (rank 0)
    /// t_b0 (rank 1)
    /// t_c0 (rank 2):
    ///   t_c01 (rank 1)
    /// ```
    ///
    /// An edge from `t_a01` to `t_c01` has ancestor chains `[t_a0, t_a01]`
    /// and `[t_c0, t_c01]`. The chains diverge at index 0, so the
    /// divergent ancestors are `t_a0` (rank 0) and `t_c0` (rank 2).
    /// The returned ranks are `(0, 2)`.
    ///
    /// Returns `None` if either endpoint is the same node as the other's
    /// ancestor (one chain is a prefix of the other), since no
    /// cross-rank spacer is meaningful in that case.
    fn divergent_ancestor_ranks(
        info_from: &NodeNestingInfo<'_>,
        info_to: &NodeNestingInfo<'_>,
        node_ranks_nested: &NodeRanksNested<'static>,
    ) -> Option<(NodeRank, NodeRank)> {
        let lca_depth = LcaDepthCalculator::calculate(info_from, info_to);
        let divergent_from = info_from.ancestor_chain.get(lca_depth)?;
        let divergent_to = info_to.ancestor_chain.get(lca_depth)?;

        let lca_container = lca_depth
            .checked_sub(1)
            .map(|i| &info_from.ancestor_chain[i]);
        let container_ranks = node_ranks_nested.ranks_for(lca_container)?;

        let rank_from = container_ranks
            .get(divergent_from)
            .copied()
            .unwrap_or(NodeRank::new(0));
        let rank_to = container_ranks
            .get(divergent_to)
            .copied()
            .unwrap_or(NodeRank::new(0));

        let (rank_low, rank_high) = if rank_from < rank_to {
            (rank_from, rank_to)
        } else {
            (rank_to, rank_from)
        };
        Some((rank_low, rank_high))
    }

    // === Insertion index computation === //

    /// Computes the base insertion index from the nesting info of two nodes.
    ///
    /// Finds the depth at which the two ancestor chains diverge, then
    /// uses the sibling indices at that depth to compute a midpoint
    /// position. Returns `(from_index + to_index) / 2 + 1`.
    ///
    /// When the ancestor chains share a common prefix (the nodes have a
    /// common ancestor), the comparison is done at the first level where
    /// the chains differ, ensuring the spacer is placed between the
    /// correct subtrees.
    fn insertion_base_index_compute(
        nesting_info_from: &NodeNestingInfo<'_>,
        nesting_info_to: &NodeNestingInfo<'_>,
    ) -> usize {
        let lca_depth = LcaDepthCalculator::calculate(nesting_info_from, nesting_info_to);

        // Get the sibling index at the divergence depth for each node.
        // This is the position of each node's subtree among the children
        // of their lowest common ancestor.
        let from_index = nesting_info_from
            .nesting_path
            .get(lca_depth)
            .copied()
            .unwrap_or(0);
        let to_index = nesting_info_to
            .nesting_path
            .get(lca_depth)
            .copied()
            .unwrap_or(0);

        // Mean index + 1 (the +1 is so the spacer goes *after* the midpoint).
        (from_index + to_index) / 2 + 1
    }

    /// Computes the effective insertion index, accounting for previously
    /// inserted spacers at or before the base insertion index.
    ///
    /// This ensures that when multiple edges insert spacers at the same
    /// rank, each new spacer is placed after any existing spacers at or
    /// before its intended position.
    fn effective_insertion_index(
        base_index: usize,
        current_len: usize,
        spacer_counts: &[usize],
    ) -> usize {
        // Count the number of spacers already inserted at or before the
        // base position.
        let spacers_at_or_before: usize = spacer_counts
            .iter()
            .take(base_index.min(spacer_counts.len()))
            .sum();

        let effective = base_index + spacers_at_or_before;
        effective.min(current_len)
    }
}
