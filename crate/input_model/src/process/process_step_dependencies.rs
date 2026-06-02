use std::ops::{Deref, DerefMut};

use disposition_model_common::{Id, Map, Set};
use serde::{Deserialize, Serialize};

use crate::process::ProcessStepId;

/// Dependencies between process steps.
///
/// Each process step maps to the set of process steps it depends on (its
/// prerequisites). A step is positioned after all of the steps it depends on.
///
/// This provides a first-class way to define ordering between process steps,
/// instead of encoding process step IDs within `thing_dependencies`.
///
/// # Example
///
/// ```yaml
/// processes:
///   proc_app_dev: # <-- this is a `ProcessDiagram`
///     steps:
///       proc_app_dev_step_repository_clone: "Clone repository"
///       proc_app_dev_step_project_build: "Build project"
///     process_step_dependencies: # <-- this is a `ProcessStepDependencies`
///       proc_app_dev_step_project_build:
///         - proc_app_dev_step_repository_clone
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProcessStepDependencies<'id>(Map<ProcessStepId<'id>, Set<ProcessStepId<'id>>>);

impl<'id> ProcessStepDependencies<'id> {
    /// Returns a new `ProcessStepDependencies` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ProcessStepDependencies` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<ProcessStepId<'id>, Set<ProcessStepId<'id>>> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this contains dependencies for a process step with the
    /// given ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'id>>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl<'id> Deref for ProcessStepDependencies<'id> {
    type Target = Map<ProcessStepId<'id>, Set<ProcessStepId<'id>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for ProcessStepDependencies<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<ProcessStepId<'id>, Set<ProcessStepId<'id>>>> for ProcessStepDependencies<'id> {
    fn from(inner: Map<ProcessStepId<'id>, Set<ProcessStepId<'id>>>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(ProcessStepId<'id>, Set<ProcessStepId<'id>>)>
    for ProcessStepDependencies<'id>
{
    fn from_iter<I: IntoIterator<Item = (ProcessStepId<'id>, Set<ProcessStepId<'id>>)>>(
        iter: I,
    ) -> Self {
        Self(Map::from_iter(iter))
    }
}
