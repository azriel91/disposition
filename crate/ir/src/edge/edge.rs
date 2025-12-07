use serde::{Deserialize, Serialize};

use crate::node::NodeId;

/// A single directed edge between two nodes.
///
/// An edge represents a connection from one node to another. Multiple edges may
/// be grouped together in an [`EdgeGroup`] to and are styled together.
///
/// [`EdgeGroup`]: crate::edge::EdgeGroup
///
/// # Example
///
/// ```yaml
/// edge_groups:
///   edge_t_localhost__t_github_user_repo:  # <-- this is an `EdgeGroup`
///     - from: t_github_user_repo  # <-- this is an `Edge`
///       to: t_localhost
///     - from: t_localhost
///       to: t_github_user_repo
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Edge {
    /// The source node ID where this edge originates.
    pub from: NodeId,

    /// The target node ID where this edge points to.
    pub to: NodeId,
}

impl Edge {
    /// Creates a new `Edge` from source to target.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_model_common::{id, Id};
    /// use disposition_ir::{edge::Edge, node::NodeId};
    ///
    /// let from: NodeId = id!("node_a").into();
    /// let to: NodeId = id!("node_b").into();
    /// let edge = Edge::new(from.clone(), to.clone());
    ///
    /// assert_eq!(edge.from, from);
    /// assert_eq!(edge.to, to);
    /// ```
    pub fn new(from: NodeId, to: NodeId) -> Self {
        Self { from, to }
    }

    /// Returns whether this edge is a self-loop (from and to are the same
    /// node).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_model_common::{id, Id};
    /// use disposition_ir::{edge::Edge, node::NodeId};
    ///
    /// let node: NodeId = id!("node_a").into();
    /// let edge = Edge::new(node.clone(), node.clone());
    ///
    /// assert!(edge.is_self_loop());
    /// ```
    pub fn is_self_loop(&self) -> bool {
        self.from == self.to
    }

    /// Returns a reversed copy of this edge (swaps from and to).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_model_common::{id, Id};
    /// use disposition_ir::{edge::Edge, node::NodeId};
    ///
    /// let from: NodeId = id!("node_a").into();
    /// let to: NodeId = id!("node_b").into();
    /// let edge = Edge::new(from.clone(), to.clone());
    /// let reversed = edge.reversed();
    ///
    /// assert_eq!(reversed.from, to);
    /// assert_eq!(reversed.to, from);
    /// ```
    pub fn reversed(&self) -> Self {
        Self {
            from: self.to.clone(),
            to: self.from.clone(),
        }
    }
}
