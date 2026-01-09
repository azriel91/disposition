use std::ops::{Deref, DerefMut};

use disposition_model_common::Map;
use serde::{Deserialize, Serialize};

use crate::node::NodeId;

/// Order that nodes should appear in the final SVG, and their tab indices.
///
/// This map defines the rendering order and tab index for all node types
/// including:
///
/// * Tags
/// * Process steps
/// * Processes
/// * Things
///
/// # Notes
///
/// 1. This is flat because we need each of these nodes to be siblings.
/// 2. Tags and process steps need to come before things for the sibling
///    selector to work.
/// 3. Process steps must come before processes so that focusing on a process
///    step can be used to style a process node.
/// 4. Tab indices are explicitly calculated to be what a user would expect --
///    things in order of declaration, then each process and its steps, then
///    tags.
///
/// # Example
///
/// ```yaml
/// node_ordering:
///   # tags
///   tag_app_development: 30
///   tag_deployment: 31
///
///   # process steps
///   proc_app_dev_step_repository_clone: 20
///   proc_app_dev_step_project_build: 21
///
///   # processes
///   proc_app_dev: 19
///
///   # things
///   t_aws: 1
///   t_aws_iam: 2
///   t_localhost: 3
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct NodeOrdering<'id>(Map<NodeId<'id>, u32>);

impl<'id> NodeOrdering<'id> {
    /// Returns a new `NodeOrdering` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `NodeOrdering` map with the given preallocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<NodeId<'id>, u32> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'id> Deref for NodeOrdering<'id> {
    type Target = Map<NodeId<'id>, u32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for NodeOrdering<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<NodeId<'id>, u32>> for NodeOrdering<'id> {
    fn from(inner: Map<NodeId<'id>, u32>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(NodeId<'id>, u32)> for NodeOrdering<'id> {
    fn from_iter<I: IntoIterator<Item = (NodeId<'id>, u32)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
