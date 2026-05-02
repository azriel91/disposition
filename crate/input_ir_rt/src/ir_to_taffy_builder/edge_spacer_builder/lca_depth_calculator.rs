use std::ops::ControlFlow;

use disposition_ir_model::node::NodeNestingInfo;

/// Calculates the depth of the lowest common ancestor (LCA) of two nodes.
pub struct LcaDepthCalculator;

impl LcaDepthCalculator {
    /// Returns the depth of the lowest common ancestor (LCA) of two nodes.
    ///
    /// The LCA depth is the length of the longest common prefix of the two
    /// nodes' `ancestor_chain`s. A depth of `0` means they diverge at the
    /// top level (no shared ancestor within the hierarchy).
    ///
    /// # Examples
    ///
    /// * `[a, a01]` and `[c, c01]` -> LCA depth `0` (diverge immediately).
    /// * `[outer, a, a01]` and `[outer, b]` -> LCA depth `1` (share `outer`).
    /// * `[outer, inner, x]` and `[outer, inner, y]` -> LCA depth `2` (share
    ///   `outer` and `inner`).
    pub fn calculate(info_from: &NodeNestingInfo<'_>, info_to: &NodeNestingInfo<'_>) -> usize {
        let max_compare = info_from
            .ancestor_chain
            .len()
            .min(info_to.ancestor_chain.len());
        let mut depth = 0;
        let (ControlFlow::Continue(()) | ControlFlow::Break(())) =
            (0..max_compare).try_for_each(|i| {
                if info_from.ancestor_chain[i] == info_to.ancestor_chain[i] {
                    depth = i + 1;
                    ControlFlow::Continue(())
                } else {
                    ControlFlow::Break(())
                }
            });
        depth
    }
}
