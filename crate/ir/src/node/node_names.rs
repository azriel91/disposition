use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{common::Map, node::NodeId};

/// All nodes in the diagram and their display labels.
///
/// This map contains display names for all node types including:
/// * Things
/// * Tags
/// * Processes
/// * Process steps
///
/// # Example
///
/// ```yaml
/// nodes:
///   # things
///   t_aws: "☁️ Amazon Web Services"
///   t_aws_iam: "🖊️ Identity and Access Management"
///   t_github: "🐙 GitHub"
///   t_localhost: "🧑‍💻 Localhost"
///
///   # tags
///   tag_app_development: "Application Development"
///   tag_deployment: "Deployment"
///
///   # processes
///   proc_app_dev: "App Development"
///   proc_app_release: "App Release"
///
///   # steps
///   proc_app_dev_step_repository_clone: "Clone repository"
///   proc_app_dev_step_project_build: "Build project"
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct NodeNames(Map<NodeId, String>);

impl NodeNames {
    /// Returns a new `NodeNames` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `NodeNames` map with the given preallocated capacity.
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

impl Deref for NodeNames {
    type Target = Map<NodeId, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NodeNames {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<NodeId, String>> for NodeNames {
    fn from(inner: Map<NodeId, String>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(NodeId, String)> for NodeNames {
    fn from_iter<I: IntoIterator<Item = (NodeId, String)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
