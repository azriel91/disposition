use std::ops::{Deref, DerefMut};

use disposition_model_common::Map;
use serde::{Deserialize, Serialize};

use crate::tag::TagId;

/// Tags are labels that can be associated with things, so that the things can
/// be highlighted when the tag is focused.
///
/// # Example
///
/// ```yaml
/// tags:
///   tag_app_development: "Application Development"
///   tag_deployment: "Deployment"
///   tag_infrastructure: "Infrastructure"
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct TagNames(Map<TagId, String>);

impl TagNames {
    /// Returns a new `TagNames` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `TagNames` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<TagId, String> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for TagNames {
    type Target = Map<TagId, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TagNames {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<TagId, String>> for TagNames {
    fn from(inner: Map<TagId, String>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(TagId, String)> for TagNames {
    fn from_iter<I: IntoIterator<Item = (TagId, String)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
