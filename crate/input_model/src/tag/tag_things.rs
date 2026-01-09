use std::ops::{Deref, DerefMut};

use disposition_model_common::{Id, Map, Set};
use serde::{Deserialize, Serialize};

use crate::{tag::TagId, thing::ThingId};

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
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct TagThings<'id>(Map<TagId<'id>, Set<ThingId<'id>>>);

impl<'id> TagThings<'id> {
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
    pub fn into_inner(self) -> Map<TagId<'id>, Set<ThingId<'id>>> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this contains things for a tag with the given ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'id>>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl<'id> Deref for TagThings<'id> {
    type Target = Map<TagId<'id>, Set<ThingId<'id>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for TagThings<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<TagId<'id>, Set<ThingId<'id>>>> for TagThings<'id> {
    fn from(inner: Map<TagId<'id>, Set<ThingId<'id>>>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(TagId<'id>, Set<ThingId<'id>>)> for TagThings<'id> {
    fn from_iter<I: IntoIterator<Item = (TagId<'id>, Set<ThingId<'id>>)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
