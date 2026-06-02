use std::ops::{Deref, DerefMut};

use disposition_model_common::{Id, Map};
use serde::{Deserialize, Serialize};

use crate::{node::NodeId, process::ProcessStepRank};

/// Map of process step node IDs to their computed ranks.
///
/// Ranks are determined from process step dependencies in the diagram. Steps
/// that depend on another step (the `to` node of a process step edge) have
/// higher ranks than the step they depend on. Steps without any dependencies
/// default to rank `0`.
///
/// # Example
///
/// ```yaml
/// process_step_ranks:
///   proc_app_dev_step_repository_clone: 0
///   proc_app_dev_step_project_build: 1
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProcessStepRanks<'id>(Map<NodeId<'id>, ProcessStepRank>);

impl<'id> ProcessStepRanks<'id> {
    /// Returns a new empty `ProcessStepRanks` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ProcessStepRanks` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<NodeId<'id>, ProcessStepRank> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this contains a rank for a process step with the given
    /// ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'id>>,
    {
        self.0.contains_key(id.as_ref())
    }

    /// Converts this `ProcessStepRanks` into one with a `'static` lifetime.
    pub fn into_static(self) -> ProcessStepRanks<'static> {
        ProcessStepRanks(
            self.0
                .into_iter()
                .map(|(node_id, rank)| (node_id.into_static(), rank))
                .collect(),
        )
    }
}

impl<'id> Deref for ProcessStepRanks<'id> {
    type Target = Map<NodeId<'id>, ProcessStepRank>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for ProcessStepRanks<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<NodeId<'id>, ProcessStepRank>> for ProcessStepRanks<'id> {
    fn from(inner: Map<NodeId<'id>, ProcessStepRank>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(NodeId<'id>, ProcessStepRank)> for ProcessStepRanks<'id> {
    fn from_iter<I: IntoIterator<Item = (NodeId<'id>, ProcessStepRank)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
