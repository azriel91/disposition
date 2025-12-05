use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{common::Map, edge::EdgeId};

/// Descriptions to render next to each edge arrow.
///
/// This is intended to take markdown text. Values can be `None` (`~` in YAML)
/// to indicate no description should be rendered.
///
/// # Example
///
/// ```yaml
/// thing_dependencies_descs:
///   edge_t_localhost__t_github_user_repo__pull: |-
///     `git pull`
///   edge_t_localhost__t_github_user_repo__push: |-
///     `git push`
///   edge_t_localhost__t_localhost__within: ~
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThingDependenciesDescs(Map<EdgeId, Option<String>>);

impl ThingDependenciesDescs {
    /// Returns a new `ThingDependenciesDescs` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ThingDependenciesDescs` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<EdgeId, Option<String>> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for ThingDependenciesDescs {
    type Target = Map<EdgeId, Option<String>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ThingDependenciesDescs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<EdgeId, Option<String>>> for ThingDependenciesDescs {
    fn from(inner: Map<EdgeId, Option<String>>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(EdgeId, Option<String>)> for ThingDependenciesDescs {
    fn from_iter<I: IntoIterator<Item = (EdgeId, Option<String>)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
