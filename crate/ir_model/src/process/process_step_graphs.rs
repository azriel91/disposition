use std::ops::{Deref, DerefMut};

use disposition_model_common::{Id, Map};
use serde::{Deserialize, Serialize};

use crate::{node::NodeId, process::ProcessStepGraph};

/// Map of process node IDs to their git-graph layout.
///
/// Each process with steps has a [`ProcessStepGraph`] describing the row/lane
/// placement of its step circles and the connector edges between them.
///
/// # Example
///
/// ```yaml
/// process_step_graphs:
///   proc_app_dev:
///     lane_count: 1
///     step_placements:
///       proc_app_dev_step_repository_clone:
///         row: 0
///         lane: 0
///       proc_app_dev_step_project_build:
///         row: 1
///         lane: 0
///     edges:
///       - from: proc_app_dev_step_repository_clone
///         to: proc_app_dev_step_project_build
///         lane: 0
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProcessStepGraphs<'id>(Map<NodeId<'id>, ProcessStepGraph<'id>>);

impl<'id> ProcessStepGraphs<'id> {
    /// Returns a new empty `ProcessStepGraphs` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ProcessStepGraphs` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<NodeId<'id>, ProcessStepGraph<'id>> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this contains a graph for a process with the given ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'id>>,
    {
        self.0.contains_key(id.as_ref())
    }

    /// Converts this `ProcessStepGraphs` into one with a `'static` lifetime.
    pub fn into_static(self) -> ProcessStepGraphs<'static> {
        ProcessStepGraphs(
            self.0
                .into_iter()
                .map(|(node_id, graph)| (node_id.into_static(), graph.into_static()))
                .collect(),
        )
    }
}

impl<'id> Deref for ProcessStepGraphs<'id> {
    type Target = Map<NodeId<'id>, ProcessStepGraph<'id>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for ProcessStepGraphs<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<NodeId<'id>, ProcessStepGraph<'id>>> for ProcessStepGraphs<'id> {
    fn from(inner: Map<NodeId<'id>, ProcessStepGraph<'id>>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(NodeId<'id>, ProcessStepGraph<'id>)> for ProcessStepGraphs<'id> {
    fn from_iter<I: IntoIterator<Item = (NodeId<'id>, ProcessStepGraph<'id>)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
