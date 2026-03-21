use std::fmt;

use serde::{Deserialize, Serialize};

/// The rank of a node, used for layout ordering within a diagram.
///
/// Nodes with lower rank values are positioned earlier (left/top) and nodes
/// with higher rank values are positioned later (right/bottom) along the
/// flex direction axis.
///
/// Rank is determined by dependency edges -- a node that is depended upon
/// (the `to` node of a dependency edge) receives a higher rank than the
/// source node.
///
/// # Examples
///
/// Valid values: `0`, `1`, `2`, `3`
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(
    Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct NodeRank(u32);

impl NodeRank {
    /// Creates a new `NodeRank` with the given value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_ir_model::node::NodeRank;
    ///
    /// let rank = NodeRank::new(2);
    /// assert_eq!(rank.value(), 2);
    /// ```
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the rank value.
    pub fn value(self) -> u32 {
        self.0
    }
}

impl fmt::Display for NodeRank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for NodeRank {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
