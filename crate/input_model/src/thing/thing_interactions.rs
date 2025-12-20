use std::ops::{Deref, DerefMut};

use disposition_model_common::{edge::EdgeGroupId, Id, Map};
use serde::{Deserialize, Serialize};

use crate::edge::EdgeKind;

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
///
/// # Example
///
/// ```yaml
/// thing_interactions:
///   edge_t_localhost__t_github_user_repo__pull:
///     cyclic:
///       - t_localhost
///       - t_github_user_repo
///   edge_t_localhost__t_github_user_repo__push:
///     sequence:
///       - t_localhost
///       - t_github_user_repo
///   edge_t_github_user_repo__t_aws_ecr_repo__push:
///     sequence:
///       - t_github_user_repo
///       - t_aws_ecr_repo
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThingInteractions(Map<EdgeGroupId, EdgeKind>);

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
    pub fn into_inner(self) -> Map<EdgeGroupId, EdgeKind> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this contains thing interactions for a thing with the
    /// given ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl Deref for ThingInteractions {
    type Target = Map<EdgeGroupId, EdgeKind>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ThingInteractions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<EdgeGroupId, EdgeKind>> for ThingInteractions {
    fn from(inner: Map<EdgeGroupId, EdgeKind>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(EdgeGroupId, EdgeKind)> for ThingInteractions {
    fn from_iter<I: IntoIterator<Item = (EdgeGroupId, EdgeKind)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
