use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{common::Map, node::NodeId};

/// Text to copy to clipboard when a node's copy button is clicked.
///
/// This allows nodes to have different copy text than their display label.
/// Only nodes that have a copy button will be included in this map (typically
/// `thing` nodes).
///
/// # Example
///
/// ```yaml
/// node_copy_text:
///   # things
///   t_aws: "☁️ Amazon Web Services"
///   t_aws_iam: "🖊️ Identity and Access Management"
///   t_localhost: "🧑‍💻 Localhost"
///   t_localhost_repo: "~/work/web_app"
///   t_localhost_repo_src: "~/work/web_app/src"
///   t_localhost_repo_target: "~/work/web_app/target"
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct NodeCopyText(Map<NodeId, String>);

impl NodeCopyText {
    /// Returns a new `NodeCopyText` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `NodeCopyText` map with the given preallocated capacity.
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

impl Deref for NodeCopyText {
    type Target = Map<NodeId, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NodeCopyText {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<NodeId, String>> for NodeCopyText {
    fn from(inner: Map<NodeId, String>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(NodeId, String)> for NodeCopyText {
    fn from_iter<I: IntoIterator<Item = (NodeId, String)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
