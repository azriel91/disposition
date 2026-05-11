use serde::{Deserialize, Serialize};

use crate::node::NodeId;

/// Information about a node's position in the hierarchy.
///
/// Captures where a node sits in the node hierarchy, including the path of
/// sibling indices from the root to the node, and the sequence of ancestor
/// `NodeId`s.
///
/// # Examples
///
/// A node `proc_app_dev_step_repository_clone` nested inside `proc_app_dev`
/// at position 0 would have:
///
/// ```yaml
/// proc_app_dev_step_repository_clone:
///   nesting_path:
///     - 2
///     - 0
///   ancestor_chain:
///     - proc_app_dev
///     - proc_app_dev_step_repository_clone
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct NodeNestingInfo<'id> {
    /// Sequence of sibling indices at each level from root to this node.
    ///
    /// For example, `[2, 0]` means "third top-level node, first child".
    pub nesting_path: Vec<usize>,
    /// Sequence of `NodeId`s from root to this node (inclusive).
    ///
    /// For example, for node `c01` inside `c0`, this would be
    /// `[NodeId("c0"), NodeId("c01")]`.
    pub ancestor_chain: Vec<NodeId<'id>>,
}

impl<'id> NodeNestingInfo<'id> {
    /// Converts this `NodeNestingInfo` into one with a `'static` lifetime.
    ///
    /// If any inner `Cow` is borrowed, this will clone the string to create
    /// an owned version.
    pub fn into_static(self) -> NodeNestingInfo<'static> {
        NodeNestingInfo {
            nesting_path: self.nesting_path,
            ancestor_chain: self
                .ancestor_chain
                .into_iter()
                .map(NodeId::into_static)
                .collect(),
        }
    }
}
