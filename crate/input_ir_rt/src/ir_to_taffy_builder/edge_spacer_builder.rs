use std::collections::BTreeMap;

use disposition_ir_model::{
    edge::{Edge, EdgeGroups, EdgeId},
    entity::{EntityType, EntityTypes},
    node::{NodeHierarchy, NodeId, NodeRank, NodeRanks},
};
use disposition_model_common::{edge::EdgeGroupId, Id, Map};
use disposition_taffy_model::{
    taffy::{self, Size, Style, TaffyTree},
    EdgeSpacerCtx, EdgeSpacerTaffyNodes, TaffyNodeCtx,
};
use taffy::AlignSelf;

const EDGE_SPACER_LENGTH: f32 = 5.0;

/// Builds spacer taffy nodes for edges that cross multiple ranks.
///
/// When edges connect nodes at different ranks, spacer nodes are
/// inserted at each intermediate rank level. These spacer nodes
/// participate in the flex layout so that the edge path can be
/// routed through their positions, reducing the chance of edges
/// being drawn over other nodes.
pub(crate) struct EdgeSpacerBuilder;

// === Supporting types === //

/// Information about a node's position in the hierarchy.
///
/// # Examples
///
/// A node `b` nested inside `a` at position 0 would have:
///
/// ```text
/// NodeNestingInfo {
///     nesting_path: [0, 0],
///     ancestor_chain: [NodeId("a"), NodeId("b")],
/// }
/// ```
#[derive(Clone, Debug)]
struct NodeNestingInfo {
    /// Sequence of sibling indices at each level from root to this node.
    ///
    /// For example, `[2, 0]` means "third top-level node, first child".
    nesting_path: Vec<usize>,
    /// Sequence of `NodeId`s from root to this node (inclusive).
    ///
    /// For example, for node `c01` inside `c0`, this would be
    /// `[NodeId("c0"), NodeId("c01")]`.
    ancestor_chain: Vec<NodeId<'static>>,
}

// === EdgeSpacerBuilder === //

impl EdgeSpacerBuilder {
    /// Inserts spacer taffy nodes for all cross-rank edges.
    ///
    /// This modifies `rank_to_taffy_ids` in-place by inserting spacer
    /// leaf nodes at the computed positions within each intermediate rank.
    ///
    /// Returns a map from edge ID to the spacer taffy node IDs at each rank.
    pub(crate) fn build(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        edge_groups: &EdgeGroups<'static>,
        node_hierarchy: &NodeHierarchy<'static>,
        node_ranks: &NodeRanks<'static>,
        entity_types: &EntityTypes<'static>,
        target_entity_type: &EntityType,
        rank_to_taffy_ids: &mut BTreeMap<NodeRank, Vec<taffy::NodeId>>,
    ) -> Map<EdgeId<'static>, EdgeSpacerTaffyNodes> {
        // === Compute nesting info for all nodes === //
        let node_nesting_info_map = Self::node_nesting_info_map_build(node_hierarchy);

        // === Find cross-rank edges and compute spacer placements === //
        let mut edge_spacer_taffy_nodes: Map<EdgeId<'static>, EdgeSpacerTaffyNodes> = Map::new();

        // Track how many spacers have been inserted at each (rank, position)
        // so subsequent insertions account for prior spacers.
        let mut rank_spacer_counts: BTreeMap<NodeRank, Vec<usize>> = BTreeMap::new();

        for (edge_group_id, edge_group) in edge_groups.iter() {
            for (edge_index, edge) in edge_group.iter().enumerate() {
                let edge_id = Self::edge_id_generate(edge_group_id, edge_index);

                let spacer_nodes = Self::edge_spacers_build(
                    taffy_tree,
                    edge,
                    &edge_id,
                    &node_nesting_info_map,
                    node_ranks,
                    entity_types,
                    target_entity_type,
                    rank_to_taffy_ids,
                    &mut rank_spacer_counts,
                );

                if let Some(spacer_nodes) = spacer_nodes {
                    edge_spacer_taffy_nodes.insert(edge_id, spacer_nodes);
                }
            }
        }

        edge_spacer_taffy_nodes
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
    /// the two endpoints share a non-root common ancestor (in which case
    /// the spacer would need to be inserted at a nested level, which is
    /// not yet supported).
    #[allow(clippy::too_many_arguments)]
    fn edge_spacers_build(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        edge: &Edge<'static>,
        edge_id: &EdgeId<'static>,
        node_nesting_info_map: &Map<NodeId<'static>, NodeNestingInfo>,
        node_ranks: &NodeRanks<'static>,
        entity_types: &EntityTypes<'static>,
        target_entity_type: &EntityType,
        rank_to_taffy_ids: &mut BTreeMap<NodeRank, Vec<taffy::NodeId>>,
        rank_spacer_counts: &mut BTreeMap<NodeRank, Vec<usize>>,
    ) -> Option<EdgeSpacerTaffyNodes> {
        let nesting_info_from = node_nesting_info_map.get(&edge.from)?;
        let nesting_info_to = node_nesting_info_map.get(&edge.to)?;

        // === Check that the edge's top-level ancestors match the target entity type
        // === //
        let lca_depth = Self::lca_depth(nesting_info_from, nesting_info_to);
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
            Self::divergent_ancestor_ranks(nesting_info_from, nesting_info_to, node_ranks)?;

        // Only insert spacers for edges crossing ranks.
        if rank_low == rank_high {
            return None;
        }

        // If there are no intermediate ranks, no spacers needed.
        if rank_high.value() - rank_low.value() <= 1 {
            return None;
        }

        // Only insert spacers at the top level when the LCA is the root.
        // When endpoints share a non-root common ancestor, the spacer
        // would need to go inside that ancestor's child container, which
        // is not available in `rank_to_taffy_ids`.
        // Note: `lca_depth` was already computed above for entity type
        // filtering.
        if lca_depth > 0 {
            // TODO: support nested spacer insertion.
            return None;
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

        // Insert spacers at each intermediate rank (exclusive of endpoints).
        for rank_value in (rank_low.value() + 1)..rank_high.value() {
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
        }

        Some(spacer_taffy_nodes)
    }

    // === Ancestor chain and LCA === //

    /// Returns the depth of the lowest common ancestor (LCA) of two nodes.
    ///
    /// The LCA depth is the length of the longest common prefix of the two
    /// nodes' `ancestor_chain`s. A depth of `0` means they diverge at the
    /// top level (no shared ancestor within the hierarchy).
    ///
    /// # Examples
    ///
    /// * `[a, a01]` and `[c, c01]` -> LCA depth `0` (diverge immediately).
    /// * `[outer, a, a01]` and `[outer, b, b01]` -> LCA depth `1` (share
    ///   `outer`).
    /// * `[outer, inner, x]` and `[outer, inner, y]` -> LCA depth `2` (share
    ///   `outer` and `inner`).
    fn lca_depth(info_from: &NodeNestingInfo, info_to: &NodeNestingInfo) -> usize {
        let max_compare = info_from
            .ancestor_chain
            .len()
            .min(info_to.ancestor_chain.len());
        let mut depth = 0;
        for i in 0..max_compare {
            if info_from.ancestor_chain[i] == info_to.ancestor_chain[i] {
                depth = i + 1;
            } else {
                break;
            }
        }
        depth
    }

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
        info_from: &NodeNestingInfo,
        info_to: &NodeNestingInfo,
        node_ranks: &NodeRanks<'static>,
    ) -> Option<(NodeRank, NodeRank)> {
        let lca_depth = Self::lca_depth(info_from, info_to);

        // The divergent ancestor for each endpoint is the node at
        // `ancestor_chain[lca_depth]` -- the first node after the shared
        // prefix.
        let divergent_from = info_from.ancestor_chain.get(lca_depth)?;
        let divergent_to = info_to.ancestor_chain.get(lca_depth)?;

        let rank_from = node_ranks
            .get(divergent_from)
            .copied()
            .unwrap_or(NodeRank::new(0));
        let rank_to = node_ranks
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

    // === Nesting info === //

    /// Computes the nesting info for all nodes in the hierarchy.
    ///
    /// Walks the hierarchy tree recursively, recording each node's depth
    /// (nesting level), index path, and ancestor chain.
    fn node_nesting_info_map_build(
        node_hierarchy: &NodeHierarchy<'static>,
    ) -> Map<NodeId<'static>, NodeNestingInfo> {
        let mut result = Map::new();
        Self::node_nesting_info_map_build_recursive(node_hierarchy, &[], &[], &mut result);
        result
    }

    /// Recursive helper for building the nesting info map.
    fn node_nesting_info_map_build_recursive(
        hierarchy: &NodeHierarchy<'static>,
        parent_path: &[usize],
        parent_ancestor_chain: &[NodeId<'static>],
        result: &mut Map<NodeId<'static>, NodeNestingInfo>,
    ) {
        for (index, (node_id, child_hierarchy)) in hierarchy.iter().enumerate() {
            let mut nesting_path = parent_path.to_vec();
            nesting_path.push(index);

            let mut ancestor_chain = parent_ancestor_chain.to_vec();
            ancestor_chain.push(node_id.clone());

            result.insert(
                node_id.clone(),
                NodeNestingInfo {
                    nesting_path: nesting_path.clone(),
                    ancestor_chain: ancestor_chain.clone(),
                },
            );

            if !child_hierarchy.is_empty() {
                Self::node_nesting_info_map_build_recursive(
                    child_hierarchy,
                    &nesting_path,
                    &ancestor_chain,
                    result,
                );
            }
        }
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
        nesting_info_from: &NodeNestingInfo,
        nesting_info_to: &NodeNestingInfo,
    ) -> usize {
        let lca_depth = Self::lca_depth(nesting_info_from, nesting_info_to);

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

    // === Edge ID generation === //

    /// Generates an `EdgeId` from an edge group ID and edge index.
    ///
    /// Format: `"{edge_group_id}__{edge_index}"`
    fn edge_id_generate(
        edge_group_id: &EdgeGroupId<'static>,
        edge_index: usize,
    ) -> EdgeId<'static> {
        let edge_id_str = format!("{edge_group_id}__{edge_index}");
        Id::try_from(edge_id_str)
            .expect("edge ID should be valid")
            .into()
    }
}
