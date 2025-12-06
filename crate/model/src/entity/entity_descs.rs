use std::ops::{Deref, DerefMut};

use disposition_model_common::{Id, Map};
use serde::{Deserialize, Serialize};

/// Descriptions to render next to entities (things, edges, edge groups).
///
/// This is intended to take markdown text.
///
/// # Example
///
/// ```yaml
/// entity_descs:
///   # things
///   t_localhost: "User's computer"
///
///   # edge groups
///   edge_t_localhost__t_github_user_repo__pull: |-
///     `git pull`
///   edge_t_localhost__t_github_user_repo__push: |-
///     `git push`
///
///   # edges
///   edge_t_localhost__t_github_user_repo__pull__0: |-
///     `git pull`
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct EntityDescs(Map<Id, String>);

impl EntityDescs {
    /// Returns a new `EntityDescs` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `EntityDescs` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<Id, String> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for EntityDescs {
    type Target = Map<Id, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EntityDescs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<Id, String>> for EntityDescs {
    fn from(inner: Map<Id, String>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(Id, String)> for EntityDescs {
    fn from_iter<I: IntoIterator<Item = (Id, String)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
