use std::ops::{Deref, DerefMut};

use disposition_model_common::Set;
use serde::{Deserialize, Serialize};

use crate::edge::Edge;

/// Directed edges between process steps, derived from process step
/// dependencies.
///
/// Each edge points from a prerequisite step (`from`) to a step that depends on
/// it (`to`). The `to` step is positioned after the `from` step.
///
/// # Example
///
/// ```yaml
/// process_step_edges:
///   - from: proc_app_dev_step_repository_clone
///     to: proc_app_dev_step_project_build
///   - from: proc_app_release_step_crate_version_update
///     to: proc_app_release_step_pull_request_open
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProcessStepEdges<'id>(Set<Edge<'id>>);

impl<'id> ProcessStepEdges<'id> {
    /// Returns a new empty `ProcessStepEdges` set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ProcessStepEdges` set with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Set::with_capacity(capacity))
    }

    /// Returns the underlying set.
    pub fn into_inner(self) -> Set<Edge<'id>> {
        self.0
    }

    /// Returns true if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of edges in this set.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Converts this `ProcessStepEdges` into one with a `'static` lifetime.
    ///
    /// If any inner `Cow` is borrowed, this will clone the string to create
    /// an owned version.
    pub fn into_static(self) -> ProcessStepEdges<'static> {
        ProcessStepEdges(self.0.into_iter().map(|edge| edge.into_static()).collect())
    }
}

impl<'id> Deref for ProcessStepEdges<'id> {
    type Target = Set<Edge<'id>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for ProcessStepEdges<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Set<Edge<'id>>> for ProcessStepEdges<'id> {
    fn from(inner: Set<Edge<'id>>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<Edge<'id>> for ProcessStepEdges<'id> {
    fn from_iter<I: IntoIterator<Item = Edge<'id>>>(iter: I) -> Self {
        Self(Set::from_iter(iter))
    }
}
