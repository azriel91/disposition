use std::ops::{Deref, DerefMut};

use disposition_model_common::Map;
use serde::{Deserialize, Serialize};

use crate::theme::{CssClassPartials, IdOrDefaults};

/// CSS utility class partials for each element. `Map<IdOrDefaults,
/// CssClassPartials>` newtype.
///
/// This is used throughout the theme configuration to define styles for
/// nodes and edges. The keys can be `node_defaults`, `edge_defaults`, or
/// specific entity IDs.
///
/// # Example
///
/// A `ThemeStyles` map can appear under various parent keys. For example,
/// under `theme_types_styles`:
///
/// ```yaml
/// theme_types_styles:
///   type_thing_default: # <-- this is a `ThemeStyles`
///     node_defaults:
///       style_aliases_applied: [shade_light]
///       shape_color: "slate"
///       stroke_style: "solid"
///       stroke_width: "1"
///       visibility: "visible"
///     edge_defaults:
///       stroke_width: "1"
///       visibility: "visible"
///     t_aws:
///       shape_color: "yellow"
///     edge_t_localhost__t_github_user_repo__pull:
///       style_aliases_applied: [shade_light]
///       shape_color: "blue"
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(bound(
    deserialize = "IdOrDefaults<'id>: Deserialize<'de>, CssClassPartials<'id>: Deserialize<'de>"
))]
pub struct ThemeStyles<'id>(Map<IdOrDefaults<'id>, CssClassPartials<'id>>);

impl<'id> ThemeStyles<'id> {
    /// Returns a new `ThemeStyles` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ThemeStyles` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<IdOrDefaults<'id>, CssClassPartials<'id>> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'id> Deref for ThemeStyles<'id> {
    type Target = Map<IdOrDefaults<'id>, CssClassPartials<'id>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for ThemeStyles<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<IdOrDefaults<'id>, CssClassPartials<'id>>> for ThemeStyles<'id> {
    fn from(inner: Map<IdOrDefaults<'id>, CssClassPartials<'id>>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(IdOrDefaults<'id>, CssClassPartials<'id>)> for ThemeStyles<'id> {
    fn from_iter<I: IntoIterator<Item = (IdOrDefaults<'id>, CssClassPartials<'id>)>>(
        iter: I,
    ) -> Self {
        Self(Map::from_iter(iter))
    }
}
