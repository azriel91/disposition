use std::ops::{Deref, DerefMut};

use disposition_model_common::{Id, Map};
use serde::{Deserialize, Serialize};

use crate::process::{ProcessDiagram, ProcessId};

/// Processes are groupings of interactions between things sequenced over time.
///
/// We want to make it easy to see which things are involved (in each step of) a
/// process. By highlighting the things / edges when a user focuses on a step in
/// a process, it brings clarity to the user.
///
/// # Example
///
/// ```yaml
/// processes:
///   proc_app_dev:
///     name: "App Development"
///     desc: |-
///       Development of the web application.
///     steps:
///       proc_app_dev_step_repository_clone: "Clone repository"
///       proc_app_dev_step_project_build: "Build project"
///     step_descs:
///       proc_app_dev_step_repository_clone: |-
///         ```bash
///         git clone https://github.com/azriel91/web_app.git
///         ```
///     step_thing_interactions:
///       proc_app_dev_step_repository_clone: [edge_t_localhost__t_github_user_repo__pull]
///       proc_app_dev_step_project_build: [edge_t_localhost__t_localhost__within]
///
///   proc_app_release:
///     name: "App Release"
///     steps:
///       proc_app_release_step_tag_and_push: "Tag and push"
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Processes(Map<ProcessId<'static>, ProcessDiagram>);

impl Processes {
    /// Returns a new `Processes` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `Processes` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<ProcessId<'static>, ProcessDiagram> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this contains a process with the given ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'static>>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl Deref for Processes {
    type Target = Map<ProcessId<'static>, ProcessDiagram>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Processes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<ProcessId<'static>, ProcessDiagram>> for Processes {
    fn from(inner: Map<ProcessId<'static>, ProcessDiagram>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(ProcessId<'static>, ProcessDiagram)> for Processes {
    fn from_iter<I: IntoIterator<Item = (ProcessId<'static>, ProcessDiagram)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
