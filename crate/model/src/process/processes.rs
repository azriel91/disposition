use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{
    common::Map,
    process::{ProcessDiagram, ProcessId},
};

/// Processes are groupings of interactions between things sequenced over time.
///
/// We want to make it easy to see which things are involved (in each step of) a
/// process. By highlighting the things / edges when a user focuses on a step in
/// a process, it brings clarity to the user.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Processes(Map<ProcessId, ProcessDiagram>);

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
    pub fn into_inner(self) -> Map<ProcessId, ProcessDiagram> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for Processes {
    type Target = Map<ProcessId, ProcessDiagram>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Processes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<ProcessId, ProcessDiagram>> for Processes {
    fn from(inner: Map<ProcessId, ProcessDiagram>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(ProcessId, ProcessDiagram)> for Processes {
    fn from_iter<I: IntoIterator<Item = (ProcessId, ProcessDiagram)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
