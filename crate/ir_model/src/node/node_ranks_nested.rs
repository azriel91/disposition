use disposition_model_common::Map;
use serde::{Deserialize, Serialize};

use crate::node::{NodeId, NodeNestingInfos, NodeRank, NodeRanks};

/// Hierarchy-aware node rank maps.
///
/// Holds a [`NodeRanks`] for the root level (direct children of the diagram
/// root) and for each container node that has direct children. Ranks are
/// computed independently per level.
///
/// Dependency edges that cross container boundaries are attributed to the
/// lowest common ancestor (LCA) of the two endpoints. At that level, the
/// first divergent sibling ancestors are used to determine the rank
/// relationship.
///
/// # Fields
///
/// * `root` -- ranks for the top-level nodes (direct children of the diagram
///   root).
/// * `containers` -- ranks for each container node's direct children; keyed by
///   the container `NodeId`. Only containers with at least one child are
///   included.
///
/// # Example
///
/// For a hierarchy and edges:
///
/// ```yaml
/// node_hierarchy:
///   a: { a_child: {} }
///   b: { b_child_0: {}, b_child_1: {} }
///   c: { c_child: {} }
///
/// edges:
///   edge_a_b:                     { from: a,         to: b         }
///   edge_b_child_0__b_child_1:    { from: b_child_0, to: b_child_1 }
///   edge_b_child_0__c_child:      { from: b_child_0, to: c_child   }
/// ```
///
/// The resulting `NodeRanksNested` would be:
///
/// ```yaml
/// node_ranks_nested:
///   root:
///     a: 0
///     b: 1
///     c: 2   # lifted from edge_b_child_0__c_child (LCA = root, b -> c)
///   containers:
///     a:
///       a_child: 0
///     b:
///       b_child_0: 0
///       b_child_1: 1
///     c:
///       c_child: 0  # edge_b_child_0__c_child is at root level; ignored here
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct NodeRanksNested<'id> {
    /// Ranks for the top-level nodes (direct children of the diagram root).
    ///
    /// Computed from dependency edges whose LCA is the diagram root (i.e.
    /// edges between top-level nodes or edges whose endpoints' first divergent
    /// sibling ancestors are both top-level nodes).
    #[serde(default, skip_serializing_if = "NodeRanks::is_empty")]
    pub root: NodeRanks<'id>,

    /// Ranks for each container node's direct children.
    ///
    /// Only container nodes with at least one direct child are included.
    /// For each container, the ranks are computed from dependency edges whose
    /// LCA is that container -- i.e. edges between siblings of that container.
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub containers: Map<NodeId<'id>, NodeRanks<'id>>,
}

impl<'id> NodeRanksNested<'id> {
    /// Returns a new empty `NodeRanksNested`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if both `root` and `containers` are empty.
    pub fn is_empty(&self) -> bool {
        self.root.is_empty() && self.containers.is_empty()
    }

    /// Returns the [`NodeRanks`] for the given container, if any.
    ///
    /// Pass `None` to retrieve the root-level ranks. Pass `Some(container_id)`
    /// to retrieve the ranks for direct children of the given container.
    ///
    /// Always returns `Some` for `None` (root) regardless of whether `root`
    /// is empty. Returns `None` when a container is not found.
    pub fn ranks_for(&self, container: Option<&NodeId<'id>>) -> Option<&NodeRanks<'id>> {
        match container {
            None => Some(&self.root),
            Some(container_id) => self.containers.get(container_id),
        }
    }

    /// Returns the [`NodeRank`] for the given node using its parent container.
    ///
    /// Looks up the node's parent container from `node_nesting_infos` (the
    /// second-to-last element of its `ancestor_chain`, or `None` for root-level
    /// nodes), then retrieves the rank from the corresponding [`NodeRanks`].
    ///
    /// Returns `None` if the node is not found in `node_nesting_infos` or has
    /// no rank entry in its container's [`NodeRanks`].
    pub fn node_rank_for(
        &self,
        node_id: &NodeId<'id>,
        node_nesting_infos: &NodeNestingInfos<'id>,
    ) -> Option<NodeRank> {
        let nesting_info = node_nesting_infos.get(node_id)?;
        let chain = &nesting_info.ancestor_chain;
        let parent = chain.len().checked_sub(2).map(|i| &chain[i]);
        self.ranks_for(parent)?.get(node_id).copied()
    }

    /// Converts this `NodeRanksNested` into one with a `'static` lifetime.
    ///
    /// If any inner `Cow` is borrowed, this will clone the string to create
    /// an owned version.
    pub fn into_static(self) -> NodeRanksNested<'static> {
        NodeRanksNested {
            root: self.root.into_static(),
            containers: self
                .containers
                .into_iter()
                .map(|(node_id, ranks)| (node_id.into_static(), ranks.into_static()))
                .collect(),
        }
    }
}
