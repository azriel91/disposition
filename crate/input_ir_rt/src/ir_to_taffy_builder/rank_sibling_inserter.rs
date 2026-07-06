use std::collections::BTreeMap;

use disposition_ir_model::node::{NodeNestingInfo, NodeRank};
use disposition_taffy_model::taffy;

use super::edge_spacer_builder::LcaDepthCalculator;

/// Inserts taffy nodes into a rank's sibling list at a sibling-index-derived
/// position, accounting for nodes already inserted at or before that
/// position by earlier calls.
///
/// Shared by [`EdgeSpacerBuilder`] (same-level cross-rank spacers) and
/// [`EdgeDescriptionBuilder`] (same-rank edge description containers), both
/// of which need to place a new taffy node between two divergent ancestors'
/// sibling subtrees within the same rank's children `Vec`.
///
/// [`EdgeSpacerBuilder`]: super::edge_spacer_builder::EdgeSpacerBuilder
/// [`EdgeDescriptionBuilder`]: super::edge_description_builder::EdgeDescriptionBuilder
pub(crate) struct RankSiblingInserter;

impl RankSiblingInserter {
    /// Computes the base insertion index from the nesting info of two nodes.
    ///
    /// Finds the depth at which the two ancestor chains diverge, then uses
    /// the sibling indices at that depth to compute a midpoint position.
    /// Returns `(from_index + to_index) / 2 + 1`, so the new node lands
    /// immediately after the midpoint sibling.
    ///
    /// # Examples
    ///
    /// `nesting_path_from = [0]`, `nesting_path_to = [1]` (divergent at depth
    /// 0) -- returns `(0 + 1) / 2 + 1 = 1`.
    pub(crate) fn insertion_base_index_compute(
        nesting_info_from: &NodeNestingInfo<'_>,
        nesting_info_to: &NodeNestingInfo<'_>,
    ) -> usize {
        let lca_depth = LcaDepthCalculator::calculate(nesting_info_from, nesting_info_to);

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

        (from_index + to_index) / 2 + 1
    }

    /// Inserts `node_id` into `rank_to_taffy_ids[rank]` at the effective
    /// index derived from `base_index`, accounting for nodes already
    /// inserted at or before that index by earlier calls (tracked in
    /// `insertion_counts`, keyed the same way as `rank_to_taffy_ids`).
    ///
    /// This ensures that when multiple insertions target the same rank,
    /// each new node is placed after any existing nodes at or before its
    /// intended position, keeping insertion order stable regardless of the
    /// order calls arrive in.
    pub(crate) fn node_insert(
        rank_to_taffy_ids: &mut BTreeMap<NodeRank, Vec<taffy::NodeId>>,
        insertion_counts: &mut BTreeMap<NodeRank, Vec<usize>>,
        rank: NodeRank,
        base_index: usize,
        node_id: taffy::NodeId,
    ) {
        let taffy_ids = rank_to_taffy_ids.entry(rank).or_default();
        let counts = insertion_counts.entry(rank).or_default();

        if counts.len() < taffy_ids.len() + 1 {
            counts.resize(taffy_ids.len() + 1, 0);
        }

        let effective_index = Self::effective_insertion_index(base_index, taffy_ids.len(), counts);

        if effective_index >= taffy_ids.len() {
            taffy_ids.push(node_id);
        } else {
            taffy_ids.insert(effective_index, node_id);
        }

        if counts.len() <= effective_index {
            counts.resize(effective_index + 1, 0);
        }
        counts.insert(effective_index, 1);
    }

    /// Computes the effective insertion index, accounting for previously
    /// inserted nodes at or before the base insertion index.
    fn effective_insertion_index(
        base_index: usize,
        current_len: usize,
        insertion_counts: &[usize],
    ) -> usize {
        let inserted_at_or_before: usize = insertion_counts
            .iter()
            .take(base_index.min(insertion_counts.len()))
            .sum();

        (base_index + inserted_at_or_before).min(current_len)
    }
}
