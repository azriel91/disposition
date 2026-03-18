use std::ops::{Deref, DerefMut};

use disposition_model_common::{edge::EdgeGroupId, Id, Map};
use serde::{Deserialize, Serialize};

use crate::edge::EdgeGroup;

/// Dependencies between things can be one way, or cyclic.
///
/// Dependencies are static relationships between things, and should be rendered
/// as "on" or "off" depending on whether a `thing` is focused / targeted, and
/// whether the user wants to see:
///
/// * Predecessors / successors linked to this thing.
/// * Immediate dependencies vs transitive (maybe closest `n` neighbours).
///
/// * When B depends on A, it means A must exist before B.
/// * Changes to A means B is out of date.
///
/// How we render dependencies (forward / backward / undirected / bidirectional
/// arrows) can be defined separately from the meaning of the dependency.
///
/// # Example
///
/// ```yaml
/// thing_dependencies:
///   edge_t_localhost__t_github_user_repo__pull:
///     kind: cyclic
///     things:
///       - t_localhost
///       - t_github_user_repo
///   edge_t_localhost__t_github_user_repo__push:
///     kind: sequence
///     things:
///       - t_localhost
///       - t_github_user_repo
///   edge_t_localhost__t_localhost__within:
///     kind: cyclic
///     things:
///       - t_localhost
///   edge_t_github_user_repo__t_aws_ecr_repo__push:
///     kind: sequence
///     things:
///       - t_github_user_repo
///       - t_aws_ecr_repo
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThingDependencies<'id>(Map<EdgeGroupId<'id>, EdgeGroup<'id>>);

impl<'id> ThingDependencies<'id> {
    /// Returns a new `ThingDependencies` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ThingDependencies` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<EdgeGroupId<'id>, EdgeGroup<'id>> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this contains dependencies for a thing with the given
    /// ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'id>>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl<'id> Deref for ThingDependencies<'id> {
    type Target = Map<EdgeGroupId<'id>, EdgeGroup<'id>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for ThingDependencies<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<EdgeGroupId<'id>, EdgeGroup<'id>>> for ThingDependencies<'id> {
    fn from(inner: Map<EdgeGroupId<'id>, EdgeGroup<'id>>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(EdgeGroupId<'id>, EdgeGroup<'id>)> for ThingDependencies<'id> {
    fn from_iter<I: IntoIterator<Item = (EdgeGroupId<'id>, EdgeGroup<'id>)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
