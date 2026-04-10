use std::ops::{Deref, DerefMut};

use disposition_model_common::{Id, Map};
use serde::{Deserialize, Serialize};

use crate::node::{NodeId, NodeRank};

/// Map of node IDs to their computed ranks.
///
/// Ranks are determined from dependency edges in the diagram. Nodes that are
/// depended upon (the `to` node of a dependency edge) have higher ranks than
/// their source nodes. Nodes without any dependency edges default to rank `0`.
///
/// # Example
///
/// ```yaml
/// node_ranks:
///   t_localhost: 0
///   t_github_user_repo: 1
///   t_aws_ecr_repo: 2
///   t_aws_ecs_cluster: 3
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct NodeRanks<'id>(Map<NodeId<'id>, NodeRank>);

impl<'id> NodeRanks<'id> {
    /// Returns a new empty `NodeRanks` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `NodeRanks` map with the given preallocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<NodeId<'id>, NodeRank> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this contains a rank for a node with the given ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'id>>,
    {
        self.0.contains_key(id.as_ref())
    }

    /// Converts this `NodeRanks` into one with a `'static` lifetime.
    pub fn into_static(self) -> NodeRanks<'static> {
        NodeRanks(
            self.0
                .into_iter()
                .map(|(node_id, rank)| (node_id.into_static(), rank))
                .collect(),
        )
    }
}

impl<'id> Deref for NodeRanks<'id> {
    type Target = Map<NodeId<'id>, NodeRank>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for NodeRanks<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<NodeId<'id>, NodeRank>> for NodeRanks<'id> {
    fn from(inner: Map<NodeId<'id>, NodeRank>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(NodeId<'id>, NodeRank)> for NodeRanks<'id> {
    fn from_iter<I: IntoIterator<Item = (NodeId<'id>, NodeRank)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
