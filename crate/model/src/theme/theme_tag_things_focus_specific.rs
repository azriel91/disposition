use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{common::Map, tag::TagId, theme::TypeStyles};

/// Tag-specific styles when a particular tag is focused.
///
/// While `ThemeTagThingsFocus` applies the same styles to all focused tags,
/// this map allows styling things differently per tag.
///
/// The key is the tag ID, and the value contains the node/edge styles to apply
/// when that specific tag is focused.
///
/// # Example
///
/// ```yaml
/// theme_tag_things_focus_specific:
///   tag_app_development:
///     node_defaults:
///       style_aliases_applied: [stroke_dashed_animated]
///   tag_deployment:
///     node_defaults:
///       shape_color: "emerald"
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThemeTagThingsFocusSpecific(Map<TagId, TypeStyles>);

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
    pub fn into_inner(self) -> Map<TagId, TypeStyles> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for ThemeTagThingsFocusSpecific {
    type Target = Map<TagId, TypeStyles>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ThemeTagThingsFocusSpecific {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<TagId, TypeStyles>> for ThemeTagThingsFocusSpecific {
    fn from(inner: Map<TagId, TypeStyles>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(TagId, TypeStyles)> for ThemeTagThingsFocusSpecific {
    fn from_iter<I: IntoIterator<Item = (TagId, TypeStyles)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
