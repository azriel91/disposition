use std::ops::{Deref, DerefMut};

use disposition_model_common::{layout::FlexDirection, Id, Map};
use serde::{Deserialize, Serialize};

/// User-specified flex-direction overrides for things.
///
/// When a thing has children (i.e. it appears in `thing_hierarchy` with nested
/// entries), the layout engine needs to know whether to arrange those children
/// in a row or column. By default, the direction alternates based on nesting
/// depth (column at even depths, row at odd depths). Entries in this map
/// override that default for the specified thing.
///
/// Only things that act as containers (i.e. have children in the hierarchy)
/// benefit from a layout override. Leaf things are ignored.
///
/// # Note
///
/// This map uses [`Id`] keys, not [`ThingId`], so that layout overrides can be
/// applied to `NodeInbuilt` keys as well.
///
/// [`ThingId`]: crate::thing::ThingId
///
/// # Example
///
/// ```yaml
/// thing_layouts:
///   t_cloud: "row"
///   t_cloud_compute: "column"
///   t_cloud_storage: "column"
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThingLayouts<'id>(Map<Id<'id>, FlexDirection>);

impl<'id> ThingLayouts<'id> {
    /// Returns a new empty `ThingLayouts` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ThingLayouts` map with the given preallocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<Id<'id>, FlexDirection> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this contains a layout override for a thing with the
    /// given ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'id>>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl<'id> Deref for ThingLayouts<'id> {
    type Target = Map<Id<'id>, FlexDirection>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for ThingLayouts<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<Id<'id>, FlexDirection>> for ThingLayouts<'id> {
    fn from(inner: Map<Id<'id>, FlexDirection>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(Id<'id>, FlexDirection)> for ThingLayouts<'id> {
    fn from_iter<I: IntoIterator<Item = (Id<'id>, FlexDirection)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
