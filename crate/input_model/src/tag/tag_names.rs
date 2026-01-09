use std::ops::{Deref, DerefMut};

use disposition_model_common::{Id, Map};
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
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct TagNames(Map<TagId<'static>, String>);

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
    pub fn into_inner(self) -> Map<TagId<'static>, String> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this contains a tag with the given ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'static>>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl Deref for TagNames {
    type Target = Map<TagId<'static>, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TagNames {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<TagId<'static>, String>> for TagNames {
    fn from(inner: Map<TagId<'static>, String>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(TagId<'static>, String)> for TagNames {
    fn from_iter<I: IntoIterator<Item = (TagId<'static>, String)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
