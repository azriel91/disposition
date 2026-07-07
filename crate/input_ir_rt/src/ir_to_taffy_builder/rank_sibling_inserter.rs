use std::collections::BTreeMap;

use disposition_ir_model::{
    entity::{EntityType, EntityTypes},
    node::{NodeId, NodeNestingInfo, NodeRank, NodeRanks},
};
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

    /// Computes a node's LOCAL index among only the siblings that share
    /// `rank` **and** `target_entity_type`, in `node_ranks`' iteration
    /// (insertion) order.
    ///
    /// Unlike the sibling index derived from `NodeNestingInfo::nesting_path`
    /// (which counts a node's position among ALL siblings at a hierarchy
    /// level, regardless of rank or entity type), this counts only nodes
    /// whose own rank equals `rank` and whose entity types contain
    /// `target_entity_type` -- i.e. the position the node occupies within
    /// `rank_to_taffy_ids[rank]`'s bucket before any spacer / description-
    /// container insertions shift it.
    ///
    /// The entity-type filter is required because `node_ranks` (from
    /// `NodeRanksNested::ranks_for`) is shared across every entity type at
    /// that hierarchy level -- e.g. the diagram root's `NodeRanks` holds tag,
    /// process, *and* thing nodes together -- while `rank_to_taffy_ids` is
    /// built separately per entity type (`TagsContainer`, `ProcessesContainer`,
    /// `ThingsContainer`; see `taffy_node_hierarchy.md`). Without this filter,
    /// unrelated same-rank siblings of a different entity type (e.g. tags
    /// with no dependency edges, defaulting to rank 0) would inflate the
    /// computed local index past the target entity type's own bucket length.
    ///
    /// `node_ranks`' iteration order, once filtered to `target_entity_type`,
    /// matches `rank_to_taffy_ids[rank]`'s initial insertion order because
    /// both are ultimately derived from the same declaration-order traversal
    /// (`NodeRanksCalculator::ranks_compute` and the taffy child-node builders
    /// both fold over the same per-entity-type node list in the same order)
    /// -- see `node_ranks.md` and `edge_spacers.md`.
    ///
    /// Returns `None` if `node_id` is absent from `node_ranks`, its recorded
    /// rank does not equal `rank`, or it does not match `target_entity_type`.
    ///
    /// # Examples
    ///
    /// `node_ranks` (insertion order) `= [tag_x: 0, t_aws: 1, t_github: 0,
    /// t_localhost: 0]`, `rank = NodeRank::new(0)`, `target_entity_type =
    /// ThingDefault`, `node_id = t_localhost` -- `tag_x` is skipped (not a
    /// `ThingDefault`), `t_aws` is skipped (rank 1 != 0), `t_github` is local
    /// index 0, `t_localhost` is local index 1 -- returns `Some(1)`.
    pub(crate) fn rank_local_sibling_index_compute<'id>(
        node_ranks: &NodeRanks<'id>,
        rank: NodeRank,
        entity_types: &EntityTypes<'id>,
        target_entity_type: &EntityType,
        node_id: &NodeId<'id>,
    ) -> Option<usize> {
        node_ranks
            .iter()
            .filter(|&(_, node_rank)| *node_rank == rank)
            .filter(|&(candidate_id, _)| {
                entity_types
                    .get(candidate_id.as_ref())
                    .map(|types| types.contains(target_entity_type))
                    .unwrap_or(false)
            })
            .position(|(candidate_id, _)| candidate_id == node_id)
    }

    /// Inserts `node_id` into `rank_to_taffy_ids[rank]` at the effective
    /// index derived from `base_index`, accounting for nodes already
    /// inserted at or before that base index by earlier calls (tracked in
    /// `prior_base_indices`, keyed the same way as `rank_to_taffy_ids`).
    ///
    /// This ensures that when multiple insertions target the same rank,
    /// each new node is placed after any existing nodes at or before its
    /// intended position, keeping insertion order stable regardless of the
    /// order calls arrive in.
    ///
    /// `prior_base_indices` records each previously-inserted node's own
    /// `base_index` -- **not** its final position in `rank_to_taffy_ids`,
    /// which shifts every time an earlier item is inserted. Counting by
    /// `base_index` (see [`Self::effective_insertion_index`]) rather than by
    /// final position is required for correctness: once two or more items
    /// have already been inserted, a later item's final position can drift
    /// arbitrarily far from its own `base_index`, so a scheme that summed
    /// "extra insertion" markers over a position-aligned window (the
    /// previous implementation) under-counted prior insertions once they
    /// started compounding -- e.g. three insertions at consecutive base
    /// indices `1`, `2`, `3` into a 4-element list should land between
    /// element pairs `(0,1)`, `(1,2)`, `(2,3)` respectively, but the old
    /// position-aligned window silently dropped the second insertion from
    /// the third's count once it had itself shifted to a position outside
    /// the window, producing `[0, e1, 1, e2, e3, 2, 3]` (`e2`/`e3` both
    /// wedged between original elements `1` and `2`) instead of the correct
    /// `[0, e1, 1, e2, 2, e3, 3]`.
    pub(crate) fn node_insert(
        rank_to_taffy_ids: &mut BTreeMap<NodeRank, Vec<taffy::NodeId>>,
        prior_base_indices: &mut BTreeMap<NodeRank, Vec<usize>>,
        rank: NodeRank,
        base_index: usize,
        node_id: taffy::NodeId,
    ) {
        let taffy_ids = rank_to_taffy_ids.entry(rank).or_default();
        let prior_base_indices_at_rank = prior_base_indices.entry(rank).or_default();

        let effective_index = Self::effective_insertion_index(
            base_index,
            taffy_ids.len(),
            prior_base_indices_at_rank,
        );

        if effective_index >= taffy_ids.len() {
            taffy_ids.push(node_id);
        } else {
            taffy_ids.insert(effective_index, node_id);
        }

        prior_base_indices_at_rank.push(base_index);
    }

    /// Computes the effective insertion index: `base_index` plus the number
    /// of previously-inserted nodes at this rank whose own `base_index` was
    /// less than or equal to this one, capped at `current_len`.
    ///
    /// Counting by `<=` (rather than `<`) makes ties stack in call order: two
    /// insertions sharing the same `base_index` land adjacent to each other,
    /// in the order they were inserted.
    fn effective_insertion_index(
        base_index: usize,
        current_len: usize,
        prior_base_indices: &[usize],
    ) -> usize {
        let inserted_at_or_before = prior_base_indices
            .iter()
            .filter(|&&prior_base_index| prior_base_index <= base_index)
            .count();

        (base_index + inserted_at_or_before).min(current_len)
    }
}
