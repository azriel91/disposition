use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{common::Map, layout::NodeLayout, node::NodeId};

/// Map of node IDs to their layout configurations.
///
/// This map defines how each node's children should be arranged. Container
/// nodes use `NodeLayout::Flex` to specify flex layout parameters, while
/// leaf nodes use `NodeLayout::None`.
///
/// The layout cascades: if not specified, a node inherits from its parent
/// or uses the default layout.
///
/// # Example
///
/// ```yaml
/// node_layout:
///   # Root container
///   _root:
///     flex:
///       direction: "column_reverse"
///       wrap: true
///       gap: "4"
///
///   # Processes container (groups all processes horizontally)
///   _processes_container:
///     flex:
///       direction: "row"
///       wrap: true
///       gap: "4"
///
///   # Individual processes (steps stacked vertically)
///   proc_app_dev:
///     flex:
///       direction: "column"
///       wrap: false
///       gap: "2"
///
///   # Process steps are leaves
///   proc_app_dev_step_repository_clone: none
///   proc_app_dev_step_project_build: none
///
///   # Tags container
///   _tags_container:
///     flex:
///       direction: "row"
///       wrap: true
///       gap: "2"
///
///   # Tags are leaves
///   tag_app_development: none
///   tag_deployment: none
///
///   # Things container
///   _things_container:
///     flex:
///       direction: "row"
///       wrap: true
///       gap: "4"
///
///   # Top-level things
///   t_aws:
///     flex:
///       direction: "column"
///       wrap: false
///       gap: "2"
///   t_aws_iam_ecs_policy: none
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct NodeLayouts(Map<NodeId, NodeLayout>);

impl NodeLayouts {
    /// Returns a new `NodeLayouts` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `NodeLayouts` map with the given preallocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<NodeId, NodeLayout> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for NodeLayouts {
    type Target = Map<NodeId, NodeLayout>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NodeLayouts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<NodeId, NodeLayout>> for NodeLayouts {
    fn from(inner: Map<NodeId, NodeLayout>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(NodeId, NodeLayout)> for NodeLayouts {
    fn from_iter<I: IntoIterator<Item = (NodeId, NodeLayout)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
