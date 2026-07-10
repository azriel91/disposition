use disposition_ir_model::node::{NodeNestingInfo, NodeRank, NodeRanksNested};

use crate::ir_to_taffy_builder::LcaDepthCalculator;

/// Calculates the ranks of two nodes' divergent ancestors at their LCA level.
///
/// The divergent ancestors are the first nodes in each endpoint's ancestor
/// chain where the chains differ. Their ranks determine the visual rank span
/// that an edge between the two nodes crosses.
pub(crate) struct DivergentAncestorRanksCalculator;

impl DivergentAncestorRanksCalculator {
    /// Returns the ranks of the divergent ancestors as `(rank_low, rank_high)`.
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
    pub(crate) fn divergent_ancestor_ranks<'id>(
        info_from: &NodeNestingInfo<'id>,
        info_to: &NodeNestingInfo<'id>,
        node_ranks_nested: &NodeRanksNested<'id>,
    ) -> Option<(NodeRank, NodeRank)> {
        let (rank_from, rank_to) =
            Self::divergent_ancestor_ranks_from_to(info_from, info_to, node_ranks_nested)?;

        let (rank_low, rank_high) = if rank_from < rank_to {
            (rank_from, rank_to)
        } else {
            (rank_to, rank_from)
        };
        Some((rank_low, rank_high))
    }

    /// Returns the ranks of the divergent ancestors as `(rank_from, rank_to)`,
    /// preserving which rank belongs to which endpoint.
    ///
    /// Unlike [`Self::divergent_ancestor_ranks`], the ranks are not reordered
    /// into `(low, high)`, so callers can tell on which side of a container the
    /// LCA gap lies (e.g. whether the gap is at a higher or lower rank than the
    /// container's divergent ancestor).
    pub(crate) fn divergent_ancestor_ranks_from_to<'id>(
        info_from: &NodeNestingInfo<'id>,
        info_to: &NodeNestingInfo<'id>,
        node_ranks_nested: &NodeRanksNested<'id>,
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

        Some((rank_from, rank_to))
    }
}
