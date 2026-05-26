/// Sort key for ordering edge descriptions that share the same insertion
/// position within an `edge_description_container`.
///
/// Descriptions are ordered first by `middle_sibling_node_index` (edges
/// between nearby siblings appear first), then by `edge_id` as a
/// deterministic tiebreaker.
///
/// Both fields must be comparable, so the struct derives `Ord` and `PartialOrd`
/// for use as a [`std::collections::BTreeMap`] key.
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct SiblingIndexMiddleAndEdgeId {
    /// Average of the sibling indices of the two divergent ancestors of the
    /// edge at the LCA depth.
    ///
    /// Lower values sort the edge description closer to the beginning of the
    /// container (i.e. edges between siblings that are visually closer to the
    /// start of their rank row come first).
    ///
    /// Example valid values: `0`, `3`.
    pub sibling_index_middle: usize,

    /// String representation of the edge ID used as a deterministic tiebreaker
    /// when two edges have the same `middle_sibling_node_index`.
    ///
    /// Example valid values: `"edge_dep_alice_charlie__0"`,
    /// `"edge_dep_bob_charlie__0"`.
    pub edge_id: String,
}
