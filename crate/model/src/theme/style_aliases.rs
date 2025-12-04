use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{
    common::Map,
    theme::{StyleAlias, ThemeStyles},
};

/// A map of style aliases to their style property definitions.
///
/// Style aliases allow grouping of style properties under a single name,
/// which can then be applied to nodes and edges using `style_aliases_applied`.
///
/// These are available to all other `theme_*` data.
///
/// # Example
///
/// ```yaml
/// theme_default:
///   style_aliases: # <-- this is a `StyleAliases`
///     padding_none:
///       padding_top: "0"
///       padding_bottom: "0"
///       padding_left: "0"
///       padding_right: "0"
///     padding_normal:
///       padding_top: "4"
///       padding_bottom: "4"
///       padding_left: "4"
///       padding_right: "4"
///     shade_light:
///       fill_shade_hover: "200"
///       fill_shade_normal: "300"
///       fill_shade_focus: "400"
///       fill_shade_active: "500"
///       stroke_shade_hover: "300"
///       stroke_shade_normal: "400"
///       stroke_shade_focus: "500"
///       stroke_shade_active: "600"
///       text_shade: "900"
///     stroke_dashed_animated:
///       stroke_style: "dashed"
///       stroke_width: "2"
///       animate: "[stroke-dashoffset-move_2s_linear_infinite]"
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct StyleAliases(Map<StyleAlias, ThemeStyles>);

impl StyleAliases {
    /// Returns a new empty `StyleAliases` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `StyleAliases` map with the given preallocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<StyleAlias, ThemeStyles> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for StyleAliases {
    type Target = Map<StyleAlias, ThemeStyles>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StyleAliases {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<StyleAlias, ThemeStyles>> for StyleAliases {
    fn from(inner: Map<StyleAlias, ThemeStyles>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(StyleAlias, ThemeStyles)> for StyleAliases {
    fn from_iter<I: IntoIterator<Item = (StyleAlias, ThemeStyles)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
