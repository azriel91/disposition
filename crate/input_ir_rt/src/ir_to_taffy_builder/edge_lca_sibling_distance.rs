/// Information about the distance between two nodes' lowest common ancestor.
///
/// Technically this is the sibling distance of the first divergent children of
/// the LCA.
///
/// # Examples
///
/// ```yaml
/// # node hierarchy
/// outer:
///   a: { a0: { a1: {} } }
///   b: {}
///   c: { c0: {} }
/// ```
///
/// In this example, when comparing `a1` and `c0`:
///
/// 1. The LCA is `outer`.
/// 2. The `NodeNestingInfo::ancestor_chain` for `a1` is `[outer, a, a0, a1]`.
/// 3. The `NodeNestingInfo::ancestor_chain` for `c0` is `[outer, c, c0]`.
/// 4. The LCA depth is `1` (shared `outer`).
/// 5. At index `1`, the sibling nodes to compare are `a` and `c` (index 1 of
///    both ancestor chains).
/// 6. The sibling distance between `a` and `c` is `2`, which is the LCA sibling
///    distance between `a1` and `c0`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EdgeLcaSiblingDistance {
    /// The node hierarchy depth of the lowest common ancestor (LCA).
    ///
    /// # Examples
    ///
    /// * `[a, a01]` and `[c, c01]` -> LCA depth `0` (diverge immediately).
    /// * `[outer, a, a01]` and `[outer, b]` -> LCA depth `1` (share `outer`).
    /// * `[outer, inner, x]` and `[outer, inner, y]` -> LCA depth `2` (share
    ///   `outer` and `inner`).
    pub lca_depth: usize,
    /// The distance between the LCA and the sibling edge.
    pub distance: usize,
}
