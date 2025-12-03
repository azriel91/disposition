use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::common::Map;

/// A collection of style properties grouped under a single alias.
///
/// Style aliases allow defining a set of style properties that can be
/// applied together to nodes and edges by referencing the alias name.
///
/// # Example
///
/// ```yaml
/// padding_normal:
///   padding_top: "4"
///   padding_bottom: "4"
///   padding_left: "4"
///   padding_right: "4"
///
/// shade_light:
///   fill_shade_hover: "200"
///   fill_shade_normal: "300"
///   fill_shade_focus: "400"
///   fill_shade_active: "500"
///   stroke_shade_hover: "300"
///   stroke_shade_normal: "400"
///   stroke_shade_focus: "500"
///   stroke_shade_active: "600"
///   text_shade: "900"
///
/// stroke_dashed_animated:
///   stroke_style: "dashed"
///   stroke_width: "2"
///   animate: "[stroke-dashoffset-move_2s_linear_infinite]"
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct StyleAlias(Map<String, String>);

impl StyleAlias {
    /// Returns a new empty `StyleAlias`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `StyleAlias` with the given preallocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<String, String> {
        self.0
    }
}

impl Deref for StyleAlias {
    type Target = Map<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StyleAlias {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<String, String>> for StyleAlias {
    fn from(inner: Map<String, String>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(String, String)> for StyleAlias {
    fn from_iter<I: IntoIterator<Item = (String, String)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
