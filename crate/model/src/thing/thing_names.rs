use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{common::Map, thing::ThingId};

/// Things in the diagram and their display labels.
///
/// This map defines the `ThingId`s and their display names.
///
/// `ThingId`s are recommended to be namespace-aware, i.e. for things that nest,
/// the ID of the nested `thing` should be prefixed with the ID of the parent
/// `thing`.
///
/// Example:
///
/// * `my_repo`: Repository directory.
/// * `my_repo_src`: `src` directory within the repo.
/// * `my_repo_target`: `target` directory within the repo.
///
/// # Example
///
/// ```yaml
/// things:
///   t_aws: "☁️ Amazon Web Services"
///   t_aws_iam: "🖊️ Identity and Access Management"
///   t_aws_ecr: "🗄️ Elastic Container Registry"
///   t_aws_ecr_repo: "💽 web_app repo"
///   t_github: "🐙 GitHub"
///   t_github_user_repo: "azriel91/web_app"
///   t_localhost: "🧑‍💻 Localhost"
///   t_localhost_repo: "📂 ~/work/web_app"
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThingNames(Map<ThingId, String>);

impl ThingNames {
    /// Returns a new `ThingNames` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ThingNames` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<ThingId, String> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for ThingNames {
    type Target = Map<ThingId, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ThingNames {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<ThingId, String>> for ThingNames {
    fn from(inner: Map<ThingId, String>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(ThingId, String)> for ThingNames {
    fn from_iter<I: IntoIterator<Item = (ThingId, String)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
