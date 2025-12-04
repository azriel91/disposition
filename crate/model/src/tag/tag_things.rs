use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{common::Map, tag::TagId, thing::ThingId};

/// Things associated with each tag.
///
/// This allows selection / highlighting of things that are related to each
/// other when a tag is focused.
///
/// It is structured as a map from tag ID to a list of thing IDs, because
/// specifying the `things` for each tag is more natural than specifying the
/// tags associated with each thing.
///
/// # Example
///
/// ```yaml
/// tag_things:
///   tag_app_development:
///     - t_github_user_repo
///     - t_localhost
///   tag_deployment:
///     - t_aws_ecr_repo
///     - t_github_user_repo
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct TagThings(Map<TagId, Vec<ThingId>>);

impl TagThings {
    /// Returns a new `TagThings` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `TagThings` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<TagId, Vec<ThingId>> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for TagThings {
    type Target = Map<TagId, Vec<ThingId>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TagThings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<TagId, Vec<ThingId>>> for TagThings {
    fn from(inner: Map<TagId, Vec<ThingId>>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(TagId, Vec<ThingId>)> for TagThings {
    fn from_iter<I: IntoIterator<Item = (TagId, Vec<ThingId>)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
