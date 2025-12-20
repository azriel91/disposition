use std::ops::{Deref, DerefMut};

use disposition_model_common::Map;
use serde::{Deserialize, Serialize};

use crate::theme::{TagIdOrDefaults, ThemeStyles};

/// Styles when a tag is focused, applied to all tags or specific tags.
///
/// When a tag is focused, things and edges associated with the tag are
/// highlighted. This map defines the styles applied to things based on
/// the focused tag.
///
/// The `tag_defaults` key applies styles to all tags uniformly.
/// Specific tag IDs can be used to override defaults for particular tags.
///
/// # Example
///
/// ```yaml
/// theme_tag_things_focus:
///   tag_defaults:
///     node_defaults:
///       style_aliases_applied: [shade_pale, stroke_dashed_animated]
///     node_excluded_defaults:
///       opacity: "0.5"
///
///   tag_app_development:
///     node_defaults:
///       style_aliases_applied: [stroke_dashed_animated]
///     node_excluded_defaults:
///       opacity: "0.3"
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThemeTagThingsFocus(Map<TagIdOrDefaults, ThemeStyles>);

impl ThemeTagThingsFocus {
    /// Returns a new empty `ThemeTagThingsFocus` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ThemeTagThingsFocus` map with the given
    /// preallocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<TagIdOrDefaults, ThemeStyles> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for ThemeTagThingsFocus {
    type Target = Map<TagIdOrDefaults, ThemeStyles>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ThemeTagThingsFocus {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<TagIdOrDefaults, ThemeStyles>> for ThemeTagThingsFocus {
    fn from(inner: Map<TagIdOrDefaults, ThemeStyles>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(TagIdOrDefaults, ThemeStyles)> for ThemeTagThingsFocus {
    fn from_iter<I: IntoIterator<Item = (TagIdOrDefaults, ThemeStyles)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
