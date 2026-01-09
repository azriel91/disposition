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
///   edge_t_aws_ecr_repo__t_aws_ecs_service__push:
///     - from: t_aws_ecr_repo
///       to: t_aws_ecs_service
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct EdgeGroups(Map<EdgeGroupId<'static>, EdgeGroup>);

impl EdgeGroups {
    /// Returns a new `EdgeGroups` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `EdgeGroups` map with the given preallocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<EdgeGroupId<'static>, EdgeGroup> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this contains edge groups for the given ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'static>>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl Deref for EdgeGroups {
    type Target = Map<EdgeGroupId<'static>, EdgeGroup>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EdgeGroups {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<EdgeGroupId<'static>, EdgeGroup>> for EdgeGroups {
    fn from(inner: Map<EdgeGroupId<'static>, EdgeGroup>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(EdgeGroupId<'static>, EdgeGroup)> for EdgeGroups {
    fn from_iter<I: IntoIterator<Item = (EdgeGroupId<'static>, EdgeGroup)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
