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
pub struct TagNames<'id>(Map<TagId<'id>, String>);

impl<'id> TagNames<'id> {
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
    pub fn into_inner(self) -> Map<TagId<'id>, String> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this contains a tag with the given ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'id>>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl<'id> Deref for TagNames<'id> {
    type Target = Map<TagId<'id>, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for TagNames<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<TagId<'id>, String>> for TagNames<'id> {
    fn from(inner: Map<TagId<'id>, String>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(TagId<'id>, String)> for TagNames<'id> {
    fn from_iter<I: IntoIterator<Item = (TagId<'id>, String)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
