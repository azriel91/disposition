use std::ops::{Deref, DerefMut};

use disposition_model_common::{Id, Map};
use serde::{Deserialize, Serialize};

use crate::process::ProcessStepId;

/// Steps in a process and their display labels.
///
/// This map defines the `ProcessStepId`s and their display names, representing
/// the ordered sequence of steps within a process.
///
/// # Example
///
/// ```yaml
/// processes:
///   proc_app_dev:
///     steps: # <-- this is a `ProcessSteps`
///       proc_app_dev_step_repository_clone: "Clone repository"
///       proc_app_dev_step_project_build: "Build project"
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProcessSteps(Map<ProcessStepId, String>);

impl ProcessSteps {
    /// Returns a new `ProcessSteps` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ProcessSteps` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<ProcessStepId, String> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this contains a process step with the given ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl Deref for ProcessSteps {
    type Target = Map<ProcessStepId, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ProcessSteps {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<ProcessStepId, String>> for ProcessSteps {
    fn from(inner: Map<ProcessStepId, String>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(ProcessStepId, String)> for ProcessSteps {
    fn from_iter<I: IntoIterator<Item = (ProcessStepId, String)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
