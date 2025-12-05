use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{common::Map, node::NodeId};

/// Rich level of detail descriptions for nodes.
///
/// This map contains detailed descriptions (typically markdown) for nodes that
/// need them, such as process steps. These descriptions provide additional
/// context when a node is focused or expanded.
///
/// # Example
///
/// ```yaml
/// node_descs:
///   proc_app_release_step_crate_version_update: |-
///     ```bash
///     sd -s 'version = "0.3.0"' 'version = "0.3.0"' $(fd -tf -F toml) README.md src/lib.rs
///     ```
///   proc_app_release_step_pull_request_open: |-
///     Create a pull request as usual.
///   proc_app_release_step_tag_and_push: |-
///     When the PR is merged, tag the commit and push the tag to GitHub.
///
///     ```bash
///     git tag 0.3.0
///     git push origin 0.3.0
///     ```
///
///     The build will push the new version to ECR automatically.
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct NodeDescs(Map<NodeId, String>);

impl NodeDescs {
    /// Returns a new `NodeDescs` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `NodeDescs` map with the given preallocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<NodeId, String> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for NodeDescs {
    type Target = Map<NodeId, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NodeDescs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<NodeId, String>> for NodeDescs {
    fn from(inner: Map<NodeId, String>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(NodeId, String)> for NodeDescs {
    fn from_iter<I: IntoIterator<Item = (NodeId, String)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
