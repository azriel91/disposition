use std::ops::{Deref, DerefMut};

use disposition_model_common::{Id, Map};
use serde::{Deserialize, Serialize};

use crate::node::NodeId;

/// Hierarchy of all nodes as a recursive tree structure.
///
/// The `NodeHierarchy` is a tree structure stored as a map of `NodeId` to
/// `NodeHierarchy`. This structure is strictly unidirectional (no cycles).
///
/// This differs from the input schema's `thing_hierarchy` in that it includes
/// all node types:
/// * Tags (at the top level for CSS peer selector ordering)
/// * Processes (containing their steps)
/// * Things (same as input `thing_hierarchy`)
///
/// The order of node declaration is important -- `process` nodes must come
/// earlier than `thing` nodes in the DOM structure for the peer/sibling CSS
/// selectors to work correctly.
///
/// # Example
///
/// ```yaml
/// node_hierarchy:
///   # Tags before everything else (required for peer selector to target processes/things/edges)
///   tag_app_development: {}
///   tag_deployment: {}
///
///   # Processes before things/edges (required for peer selector to target things/edges)
///   proc_app_dev:
///     proc_app_dev_step_repository_clone: {}
///     proc_app_dev_step_project_build: {}
///   proc_app_release:
///     proc_app_release_step_crate_version_update: {}
///     proc_app_release_step_pull_request_open: {}
///
///   # Things (same hierarchy as input `thing_hierarchy`)
///   t_aws:
///     t_aws_iam:
///       t_aws_iam_ecs_policy: {}
///     t_aws_ecr:
///       t_aws_ecr_repo:
///         t_aws_ecr_repo_image_1: {}
///         t_aws_ecr_repo_image_2: {}
///   t_github:
///     t_github_user_repo: {}
///   t_localhost:
///     t_localhost_repo:
///       t_localhost_repo_src: {}
///       t_localhost_repo_target:
///         t_localhost_repo_target_file_zip: {}
///         t_localhost_repo_target_dist_dir: {}
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct NodeHierarchy(Map<NodeId, NodeHierarchy>);

impl NodeHierarchy {
    /// Returns a new empty `NodeHierarchy`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `NodeHierarchy` with the given preallocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<NodeId, NodeHierarchy> {
        self.0
    }

    /// Returns true if this hierarchy node has no children.
    pub fn is_leaf(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of direct children of this hierarchy node.
    pub fn children_count(&self) -> usize {
        self.0.len()
    }

    /// Recursively counts all descendant nodes in this hierarchy.
    pub fn total_descendants(&self) -> usize {
        self.0
            .values()
            .map(|child| 1 + child.total_descendants())
            .sum()
    }

    /// Returns true if the hierarchy is empty (no children).
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this contains the hierarchy for a node with the given
    /// ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl Deref for NodeHierarchy {
    type Target = Map<NodeId, NodeHierarchy>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NodeHierarchy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<NodeId, NodeHierarchy>> for NodeHierarchy {
    fn from(inner: Map<NodeId, NodeHierarchy>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(NodeId, NodeHierarchy)> for NodeHierarchy {
    fn from_iter<I: IntoIterator<Item = (NodeId, NodeHierarchy)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}

impl IntoIterator for NodeHierarchy {
    type IntoIter = <Map<NodeId, NodeHierarchy> as IntoIterator>::IntoIter;
    type Item = (NodeId, NodeHierarchy);

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'node_hierarchy> IntoIterator for &'node_hierarchy NodeHierarchy {
    type IntoIter = <&'node_hierarchy Map<NodeId, NodeHierarchy> as IntoIterator>::IntoIter;
    type Item = (&'node_hierarchy NodeId, &'node_hierarchy NodeHierarchy);

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'node_hierarchy> IntoIterator for &'node_hierarchy mut NodeHierarchy {
    type IntoIter = <&'node_hierarchy mut Map<NodeId, NodeHierarchy> as IntoIterator>::IntoIter;
    type Item = (&'node_hierarchy NodeId, &'node_hierarchy mut NodeHierarchy);

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}
