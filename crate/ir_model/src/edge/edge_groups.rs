use std::ops::{Deref, DerefMut};

use disposition_model_common::{edge::EdgeGroupId, Id, Map};
use serde::{Deserialize, Serialize};

use crate::edge::EdgeGroup;

/// Map of edge group IDs to their edge groups.
///
/// Each edge group contains the individual edges derived from the
/// input's `thing_dependencies` and `thing_interactions`. The edges
/// are the expanded form of the `cyclic` and `sequence` edge kinds.
///
/// # Example
///
/// ```yaml
/// edge_groups:
///   edge_t_localhost__t_github_user_repo:
///     - from: t_github_user_repo
///       to: t_localhost
///     - from: t_localhost
///       to: t_github_user_repo
///   edge_t_localhost__t_github_user_repo__push:
///     - from: t_localhost
///       to: t_github_user_repo
///   edge_t_localhost__t_localhost__within:
///     - from: t_localhost
///       to: t_localhost
///   edge_t_github_user_repo__t_github_user_repo__within:
///     - from: t_github_user_repo
///       to: t_github_user_repo
///   edge_t_github_user_repo__t_aws_ecr_repo__push:
///     - from: t_github_user_repo
///       to: t_aws_ecr_repo
///   edge_t_aws_ecr_repo__t_aws_ecs_cluster__push:
///     - from: t_aws_ecr_repo
///       to: t_aws_ecs_cluster
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct EdgeGroups<'id>(Map<EdgeGroupId<'id>, EdgeGroup<'id>>);

impl<'id> EdgeGroups<'id> {
    /// Returns a new `EdgeGroups` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `EdgeGroups` map with the given preallocated capacity.
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

    /// Returns true if this contains edge groups for the given ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'id>>,
    {
        self.0.contains_key(id.as_ref())
    }

    /// Converts this `EdgeGroups` into one with a `'static` lifetime.
    ///
    /// If any inner `Cow` is borrowed, this will clone the string to create
    /// an owned version.
    pub fn into_static(self) -> EdgeGroups<'static> {
        EdgeGroups(
            self.0
                .into_iter()
                .map(|(edge_group_id, edge_group)| {
                    (edge_group_id.into_static(), edge_group.into_static())
                })
                .collect(),
        )
    }
}

impl<'id> Deref for EdgeGroups<'id> {
    type Target = Map<EdgeGroupId<'id>, EdgeGroup<'id>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for EdgeGroups<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<EdgeGroupId<'id>, EdgeGroup<'id>>> for EdgeGroups<'id> {
    fn from(inner: Map<EdgeGroupId<'id>, EdgeGroup<'id>>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(EdgeGroupId<'id>, EdgeGroup<'id>)> for EdgeGroups<'id> {
    fn from_iter<I: IntoIterator<Item = (EdgeGroupId<'id>, EdgeGroup<'id>)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
