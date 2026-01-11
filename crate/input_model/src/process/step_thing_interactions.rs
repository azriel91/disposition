use std::ops::{Deref, DerefMut};

use disposition_model_common::{edge::EdgeGroupId, Id, Map};
use serde::{Deserialize, Serialize};

use crate::process::ProcessStepId;

/// Thing interactions that should be actively highlighted when a step is
/// focused.
///
/// The things associated with all of the interaction IDs in the list should be
/// highlighted when viewing this step.
///
/// # Example
///
/// ```yaml
/// processes:
///   proc_app_dev: # <-- this is a `ProcessDiagram`
///     step_thing_interactions: # <-- this is a `StepThingInteractions`
///       proc_app_dev_step_repository_clone: [edge_t_localhost__t_github_user_repo__pull]
///       proc_app_dev_step_project_build: [edge_t_localhost__t_localhost__within]
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct StepThingInteractions<'id>(Map<ProcessStepId<'id>, Vec<EdgeGroupId<'id>>>);

impl<'id> StepThingInteractions<'id> {
    /// Returns a new `StepThingInteractions` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `StepThingInteractions` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<ProcessStepId<'id>, Vec<EdgeGroupId<'id>>> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this contains thing interactions for a process step with
    /// the given ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'id>>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl<'id> Deref for StepThingInteractions<'id> {
    type Target = Map<ProcessStepId<'id>, Vec<EdgeGroupId<'id>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for StepThingInteractions<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<ProcessStepId<'id>, Vec<EdgeGroupId<'id>>>> for StepThingInteractions<'id> {
    fn from(inner: Map<ProcessStepId<'id>, Vec<EdgeGroupId<'id>>>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(ProcessStepId<'id>, Vec<EdgeGroupId<'id>>)> for StepThingInteractions<'id> {
    fn from_iter<I: IntoIterator<Item = (ProcessStepId<'id>, Vec<EdgeGroupId<'id>>)>>(
        iter: I,
    ) -> Self {
        Self(Map::from_iter(iter))
    }
}
