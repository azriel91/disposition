use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::common::{Id, Map};

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
///   # node descriptions
///   proc_app_release_step_crate_version_update: |-
///     ```bash
///     sd -s 'version = "0.3.0"' 'version = "0.3.0"' $(fd -tf -F toml) README.md src/lib.rs
///     ```
///   proc_app_release_step_pull_request_open: |-
///     Create a pull request as usual.
///   proc_app_release_step_tag_and_push: |-
///     When the PR is merged, tag the commit and push the tag to GitHub.
///
///     ```bash
///     git tag 0.3.0
///     git push origin 0.3.0
///     ```
///
///   # edge group descriptions
///   edge_t_localhost__t_github_user_repo__pull: "Fetch from GitHub"
///   edge_t_localhost__t_github_user_repo__push: "Push to GitHub"
///
///   # edge descriptions
///   edge_t_localhost__t_github_user_repo__pull__0: "`git pull`"
///   edge_t_localhost__t_github_user_repo__push__0: "`git push`"
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
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
