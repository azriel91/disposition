use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{common::Map, tag::TagId, theme::StyleSet};

/// Tag-specific styles when a particular tag is focused.
///
/// While `ThemeTagThingsFocus` applies the same styles to all focused tags,
/// this map allows styling things differently per tag.
///
/// Each entry maps a `TagId` to the styles that should be applied when
/// that specific tag is focused.
///
/// # Example
///
/// ```yaml
/// tag_aws:
///   node_defaults:
///     shape_color: "yellow"
///
/// tag_github:
///   node_defaults:
///     shape_color: "neutral"
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThemeTagThingsFocusSpecific(Map<TagId, StyleSet>);

impl ThemeTagThingsFocusSpecific {
    /// Returns a new empty `ThemeTagThingsFocusSpecific` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ThemeTagThingsFocusSpecific` map with the given
    /// preallocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<TagId, StyleSet> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for ThemeTagThingsFocusSpecific {
    type Target = Map<TagId, StyleSet>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ThemeTagThingsFocusSpecific {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<TagId, StyleSet>> for ThemeTagThingsFocusSpecific {
    fn from(inner: Map<TagId, StyleSet>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(TagId, StyleSet)> for ThemeTagThingsFocusSpecific {
    fn from_iter<I: IntoIterator<Item = (TagId, StyleSet)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
