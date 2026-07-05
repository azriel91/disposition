use disposition_ir_model::node::NodeRank;

use super::rank_and_sibling_index_middle::RankAndSiblingIndexMiddle;

/// Where an `edge_description_container` should be inserted, computed from
/// the edge's divergent ancestor ranks.
pub enum EdgeDescPosition {
    /// The divergent ancestors are at different ranks -- the container is
    /// inserted as a sibling of rank containers, interleaved after
    /// `rank_container[rank]`.
    ///
    /// This is always a concrete rank; the caller wraps it in `Some(..)` to
    /// match `EdgeDescriptionBuildResult::position_to_container_ids`'s
    /// `Option<NodeRank>` key, whose consumers (`rank_containers_interleave`,
    /// `build_edge_desc_container_spacers`) still support a `None` ("before
    /// all rank containers") position, kept for that map's contract even
    /// though this builder never produces it now.
    BetweenRanks(NodeRank),

    /// The divergent ancestors share a rank (a cycle edge) -- the container
    /// is inserted as a direct child of that shared rank, between the two
    /// siblings.
    SameRank(RankAndSiblingIndexMiddle),
}
