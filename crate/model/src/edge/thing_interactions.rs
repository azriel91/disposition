use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{
    common::Map,
    edge::{EdgeId, EdgeKind},
};

/// Interactions between things can be one way, or cyclic.
///
/// Interactions have the same data structure as dependencies, but are
/// conceptually different: `thing_dependencies` is intended to represent
/// dependencies between software libraries, while interactions are
/// communication between applications.
///
/// There *are* ordering dependencies between interactions, but *when* it is
/// useful to render `thing_dependencies` and `thing_interactions` differ.
/// Dependencies are static at a point in time, so it is useful to render the
/// links between multiple `thing`s; interactions are present when a step in a
/// process is executing, so they are rendered when the step is focused.
///
/// IDs here can be the same as the ones in `thing_dependencies`.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThingInteractions(Map<EdgeId, EdgeKind>);

impl ThingInteractions {
    /// Returns a new `ThingInteractions` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ThingInteractions` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<EdgeId, EdgeKind> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for ThingInteractions {
    type Target = Map<EdgeId, EdgeKind>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ThingInteractions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<EdgeId, EdgeKind>> for ThingInteractions {
    fn from(inner: Map<EdgeId, EdgeKind>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(EdgeId, EdgeKind)> for ThingInteractions {
    fn from_iter<I: IntoIterator<Item = (EdgeId, EdgeKind)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
