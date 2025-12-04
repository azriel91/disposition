use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{common::Map, edge::EdgeId, process::ProcessStepId};

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
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct StepThingInteractions(Map<ProcessStepId, Vec<EdgeId>>);

impl StepThingInteractions {
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
    pub fn into_inner(self) -> Map<ProcessStepId, Vec<EdgeId>> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for StepThingInteractions {
    type Target = Map<ProcessStepId, Vec<EdgeId>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StepThingInteractions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<ProcessStepId, Vec<EdgeId>>> for StepThingInteractions {
    fn from(inner: Map<ProcessStepId, Vec<EdgeId>>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(ProcessStepId, Vec<EdgeId>)> for StepThingInteractions {
    fn from_iter<I: IntoIterator<Item = (ProcessStepId, Vec<EdgeId>)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
