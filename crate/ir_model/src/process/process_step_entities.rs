use std::ops::{Deref, DerefMut};

use disposition_model_common::{Id, Map};
use serde::{Deserialize, Serialize};

use crate::node::NodeId;

/// Map from process step node IDs to the entity IDs they interact with.
///
/// Each process step can reference one or more entities (typically edge group
/// IDs from `thing_interactions`) that are activated when the step is focused.
///
/// This information is used to conditionally attach CSS animations to edges
/// based on which process step currently has focus.
///
/// # Example
///
/// ```yaml
/// process_step_entities:
///   proc_app_dev_step_repository_clone:
///     - edge_ix_t_localhost__t_github_user_repo__pull
///   proc_app_dev_step_project_build:
///     - edge_ix_t_localhost__t_localhost__within
///   proc_app_release_step_tag_and_push:
///     - edge_ix_t_localhost__t_github_user_repo__push
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProcessStepEntities<'id>(Map<NodeId<'id>, Vec<Id<'id>>>);

impl<'id> ProcessStepEntities<'id> {
    /// Returns a new `ProcessStepEntities` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ProcessStepEntities` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<NodeId<'id>, Vec<Id<'id>>> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Converts this `ProcessStepEntities` into one with a `'static` lifetime.
    ///
    /// If any inner `Cow` is borrowed, this will clone the string to create
    /// an owned version.
    pub fn into_static(self) -> ProcessStepEntities<'static> {
        ProcessStepEntities(
            self.0
                .into_iter()
                .map(|(node_id, ids)| {
                    (
                        node_id.into_static(),
                        ids.into_iter().map(Id::into_static).collect(),
                    )
                })
                .collect(),
        )
    }
}

impl<'id> Deref for ProcessStepEntities<'id> {
    type Target = Map<NodeId<'id>, Vec<Id<'id>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for ProcessStepEntities<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<NodeId<'id>, Vec<Id<'id>>>> for ProcessStepEntities<'id> {
    fn from(inner: Map<NodeId<'id>, Vec<Id<'id>>>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(NodeId<'id>, Vec<Id<'id>>)> for ProcessStepEntities<'id> {
    fn from_iter<I: IntoIterator<Item = (NodeId<'id>, Vec<Id<'id>>)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
