use disposition_ir_model::node::NodeRank;

/// Grouping key for edge descriptions whose divergent ancestors share a rank
/// (cycle edges).
///
/// Descriptions with the same `rank` and `sibling_index_middle` describe
/// edges between the same pair of same-ranked siblings, and are grouped into
/// one shared `edge_description_container` inserted between those siblings.
/// Both fields must be comparable, so the struct derives `Ord` and
/// `PartialOrd` for use as a [`std::collections::BTreeMap`] key.
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct RankAndSiblingIndexMiddle {
    /// The rank shared by the edge's two divergent ancestors.
    ///
    /// Example valid values: `NodeRank::new(0)`, `NodeRank::new(2)`.
    pub rank: NodeRank,

    /// Average of the sibling indices of the two divergent ancestors at the
    /// LCA depth.
    ///
    /// Example valid values: `0`, `3`.
    pub sibling_index_middle: usize,
}
