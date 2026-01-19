use std::ops::{Deref, DerefMut};

use disposition_model_common::{Id, Map};
use serde::{Deserialize, Serialize};

use crate::node::NodeId;

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
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct NodeNames<'id>(Map<NodeId<'id>, String>);

impl<'id> NodeNames<'id> {
    /// Returns a new `NodeNames` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `NodeNames` map with the given preallocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<NodeId<'id>, String> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Converts this `NodeNames` into one with a `'static` lifetime.
    ///
    /// If any inner `Cow` is borrowed, this will clone the string to create
    /// an owned version.
    pub fn into_static(self) -> NodeNames<'static> {
        NodeNames(
            self.0
                .into_iter()
                .map(|(node_id, name)| (node_id.into_static(), name))
                .collect(),
        )
    }

    /// Returns true if this contains a name for a node with the given ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'id>>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl<'id> Deref for NodeNames<'id> {
    type Target = Map<NodeId<'id>, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for NodeNames<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<NodeId<'id>, String>> for NodeNames<'id> {
    fn from(inner: Map<NodeId<'id>, String>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(NodeId<'id>, String)> for NodeNames<'id> {
    fn from_iter<I: IntoIterator<Item = (NodeId<'id>, String)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
