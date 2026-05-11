use std::ops::{Deref, DerefMut};

use disposition_model_common::{Id, Map};
use serde::{Deserialize, Serialize};

use crate::node::{NodeId, NodeNestingInfo};

/// Map of node IDs to their nesting information.
///
/// Captures the hierarchy position for each node, including the path of
/// sibling indices from the root and the sequence of ancestor `NodeId`s.
///
/// # Example
///
/// ```yaml
/// node_nesting_infos:
///   t_aws:
///     nesting_path:
///       - 5
///     ancestor_chain:
///       - t_aws
///   t_aws_iam:
///     nesting_path:
///       - 5
///       - 0
///     ancestor_chain:
///       - t_aws
///       - t_aws_iam
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct NodeNestingInfos<'id>(Map<NodeId<'id>, NodeNestingInfo<'id>>);

impl<'id> NodeNestingInfos<'id> {
    /// Returns a new empty `NodeNestingInfos` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `NodeNestingInfos` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<NodeId<'id>, NodeNestingInfo<'id>> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this contains nesting info for a node with the given ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'id>>,
    {
        self.0.contains_key(id.as_ref())
    }

    /// Converts this `NodeNestingInfos` into one with a `'static` lifetime.
    pub fn into_static(self) -> NodeNestingInfos<'static> {
        NodeNestingInfos(
            self.0
                .into_iter()
                .map(|(node_id, nesting_info)| (node_id.into_static(), nesting_info.into_static()))
                .collect(),
        )
    }
}

impl<'id> Deref for NodeNestingInfos<'id> {
    type Target = Map<NodeId<'id>, NodeNestingInfo<'id>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for NodeNestingInfos<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<NodeId<'id>, NodeNestingInfo<'id>>> for NodeNestingInfos<'id> {
    fn from(inner: Map<NodeId<'id>, NodeNestingInfo<'id>>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(NodeId<'id>, NodeNestingInfo<'id>)> for NodeNestingInfos<'id> {
    fn from_iter<I: IntoIterator<Item = (NodeId<'id>, NodeNestingInfo<'id>)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
