use std::ops::{Deref, DerefMut};

use disposition_model_common::Map;
use serde::{Deserialize, Serialize};

use crate::theme::{CssClassPartials, StyleAlias};

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
///       padding: "0.0"
///       gap: "0.0"
///     padding_tight:
///       padding: "2.0"
///       gap: "2.0"
///     padding_normal:
///       padding: "4.0"
///       gap: "4.0"
///     padding_wide:
///       padding: "6.0"
///       gap: "6.0"
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
pub struct StyleAliases(Map<StyleAlias, CssClassPartials>);

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
    pub fn into_inner(self) -> Map<StyleAlias, CssClassPartials> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for StyleAliases {
    type Target = Map<StyleAlias, CssClassPartials>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StyleAliases {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<StyleAlias, CssClassPartials>> for StyleAliases {
    fn from(inner: Map<StyleAlias, CssClassPartials>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(StyleAlias, CssClassPartials)> for StyleAliases {
    fn from_iter<I: IntoIterator<Item = (StyleAlias, CssClassPartials)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
