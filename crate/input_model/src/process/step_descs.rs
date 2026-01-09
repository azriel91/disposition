use std::ops::{Deref, DerefMut};

use disposition_model_common::{Id, Map};
use serde::{Deserialize, Serialize};

use crate::process::ProcessStepId;

/// Descriptions for each step in a process.
///
/// This is intended to take markdown text, providing detailed documentation
/// for each step.
///
/// # Example
///
/// ```yaml
/// processes:
///   proc_app_dev:
///     # ..
///     step_descs: # <-- this is a `StepDescs`
///       proc_app_dev_step_repository_clone: |-
///         ```bash
///         git clone https://github.com/azriel91/web_app.git
///         ```
///
///       proc_app_dev_step_project_build: |-
///         Develop the app:
///
///         * Always link to issue.
///         * Open PR.
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct StepDescs<'id>(Map<ProcessStepId<'id>, String>);

impl<'id> StepDescs<'id> {
    /// Returns a new `StepDescs` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `StepDescs` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<ProcessStepId<'id>, String> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this contains a description for the process step with
    /// the given ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'id>>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl<'id> Deref for StepDescs<'id> {
    type Target = Map<ProcessStepId<'id>, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for StepDescs<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<ProcessStepId<'id>, String>> for StepDescs<'id> {
    fn from(inner: Map<ProcessStepId<'id>, String>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(ProcessStepId<'id>, String)> for StepDescs<'id> {
    fn from_iter<I: IntoIterator<Item = (ProcessStepId<'id>, String)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
