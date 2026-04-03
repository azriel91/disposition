use std::collections::BTreeMap;

use disposition_ir_model::{
    edge::{Edge, EdgeGroups, EdgeId},
    node::{NodeHierarchy, NodeId, NodeRank, NodeRanks},
};
use disposition_model_common::{edge::EdgeGroupId, Id, Map};
use disposition_taffy_model::{
    taffy::{self, Size, Style, TaffyTree},
    EdgeSpacerCtx, EdgeSpacerTaffyNodes, TaffyNodeCtx, TEXT_LINE_HEIGHT,
};

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
/// NodeNestingInfo { nesting_level: 1, nesting_path: [0, 0] }
/// ```
#[derive(Clone, Debug)]
struct NodeNestingInfo {
    /// Depth of this node in the hierarchy tree (0 = top-level).
    nesting_level: u32,
    /// Sequence of indices at each level from root to this node.
    ///
    /// For example, `[2, 0]` means "third top-level node, first child".
    nesting_path: Vec<usize>,
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
                    &node_nesting_info_map,
                    node_ranks,
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
    /// Returns `None` if the edge does not cross ranks (from and to are
    /// at the same rank) or if there are no intermediate ranks between them.
    fn edge_spacers_build(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        edge: &Edge<'static>,
        node_nesting_info_map: &Map<NodeId<'static>, NodeNestingInfo>,
        node_ranks: &NodeRanks<'static>,
        rank_to_taffy_ids: &mut BTreeMap<NodeRank, Vec<taffy::NodeId>>,
        rank_spacer_counts: &mut BTreeMap<NodeRank, Vec<usize>>,
    ) -> Option<EdgeSpacerTaffyNodes> {
        let rank_from = node_ranks
            .get(&edge.from)
            .copied()
            .unwrap_or(NodeRank::new(0));
        let rank_to = node_ranks
            .get(&edge.to)
            .copied()
            .unwrap_or(NodeRank::new(0));

        // Only insert spacers for edges crossing ranks.
        if rank_from == rank_to {
            return None;
        }

        let (rank_low, rank_high) = if rank_from < rank_to {
            (rank_from, rank_to)
        } else {
            (rank_to, rank_from)
        };

        // If there are no intermediate ranks, no spacers needed.
        if rank_high.value() - rank_low.value() <= 1 {
            return None;
        }

        // Compute the insertion index based on nesting info.
        let nesting_info_from = node_nesting_info_map.get(&edge.from);
        let nesting_info_to = node_nesting_info_map.get(&edge.to);

        let insertion_base_index =
            Self::insertion_base_index_compute(nesting_info_from, nesting_info_to);

        let spacer_style = Style {
            size: Size::from_lengths(5.0, TEXT_LINE_HEIGHT),
            ..Default::default()
        };

        let mut spacer_taffy_nodes = EdgeSpacerTaffyNodes::new();

        // Insert spacers at each intermediate rank (exclusive of endpoints).
        for rank_value in (rank_low.value() + 1)..rank_high.value() {
            let rank = NodeRank::new(rank_value);

            let spacer_taffy_node_id = taffy_tree
                .new_leaf_with_context(
                    spacer_style.clone(),
                    TaffyNodeCtx::EdgeSpacer(EdgeSpacerCtx {}),
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

    // === Nesting info === //

    /// Computes the nesting info for all nodes in the hierarchy.
    ///
    /// Walks the hierarchy tree recursively, recording each node's depth
    /// (nesting level) and index path.
    fn node_nesting_info_map_build(
        node_hierarchy: &NodeHierarchy<'static>,
    ) -> Map<NodeId<'static>, NodeNestingInfo> {
        let mut result = Map::new();
        Self::node_nesting_info_map_build_recursive(node_hierarchy, 0, &[], &mut result);
        result
    }

    /// Recursive helper for building the nesting info map.
    fn node_nesting_info_map_build_recursive(
        hierarchy: &NodeHierarchy<'static>,
        depth: u32,
        parent_path: &[usize],
        result: &mut Map<NodeId<'static>, NodeNestingInfo>,
    ) {
        for (index, (node_id, child_hierarchy)) in hierarchy.iter().enumerate() {
            let mut nesting_path = parent_path.to_vec();
            nesting_path.push(index);

            result.insert(
                node_id.clone(),
                NodeNestingInfo {
                    nesting_level: depth,
                    nesting_path: nesting_path.clone(),
                },
            );

            if !child_hierarchy.is_empty() {
                Self::node_nesting_info_map_build_recursive(
                    child_hierarchy,
                    depth + 1,
                    &nesting_path,
                    result,
                );
            }
        }
    }

    // === Insertion index computation === //

    /// Computes the base insertion index from the nesting info of two nodes.
    ///
    /// Takes the minimum nesting level of both nodes, gets the nesting
    /// path index at that level for each node, and returns
    /// `(from_index + to_index) / 2 + 1`.
    fn insertion_base_index_compute(
        nesting_info_from: Option<&NodeNestingInfo>,
        nesting_info_to: Option<&NodeNestingInfo>,
    ) -> usize {
        let (info_from, info_to) = match (nesting_info_from, nesting_info_to) {
            (Some(from), Some(to)) => (from, to),
            // If either node is not in the hierarchy, default to inserting
            // at position 1.
            _ => return 1,
        };

        let min_nesting_level = info_from.nesting_level.min(info_to.nesting_level) as usize;

        // Get the index at the minimum nesting level for each node.
        let from_index = info_from
            .nesting_path
            .get(min_nesting_level)
            .copied()
            .unwrap_or(0);
        let to_index = info_to
            .nesting_path
            .get(min_nesting_level)
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
