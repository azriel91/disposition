use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{Id, Map};

/// Descriptions for entities (nodes, edges, and edge groups).
///
/// This map contains text (typically markdown) that provides additional
/// context about entities in the diagram. These descriptions can be displayed
/// when an entity is focused or expanded.
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
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct EntityDescs(Map<Id, String>);

impl EntityDescs {
    /// Returns a new `EntityDescs` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `EntityDescs` map with the given preallocated capacity.
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

    /// Returns true if this contains a description for an entity with the given
    /// ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id>,
    {
        self.0.contains_key(id.as_ref())
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
