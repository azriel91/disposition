use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{
    common::{Id, Map},
    theme::ThemeStyles,
};

/// Base styles when the diagram has no user interaction.
///
/// The keys in this map can be:
///
/// * `node_defaults`: Applies to all things.
/// * `edge_defaults`: Applies to all edges.
/// * `thing_id`: Applies to the particular thing.
/// * `edge_id`: Applies to the particular edge.
/// * `tag_id`: Applies to the tag.
///
/// # Example
///
/// ```yaml
/// base_styles:
///   node_defaults:
///     style_aliases_applied: [shade_light]
///     shape_color: "slate"
///     stroke_style: "solid"
///     stroke_width: "1"
///     visibility: "visible"
///   edge_defaults:
///     stroke_width: "1"
///     visibility: "visible"
///   edge_t_localhost__t_github_user_repo__pull:
///     style_aliases_applied: [shade_light]
///     shape_color: "blue"
///   t_aws:
///     shape_color: "yellow"
///   t_github:
///     shape_color: "neutral"
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct BaseStyles(Map<Id, ThemeStyles>);

impl BaseStyles {
    /// Returns a new empty `BaseStyles` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `BaseStyles` map with the given preallocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<Id, ThemeStyles> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for BaseStyles {
    type Target = Map<Id, ThemeStyles>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BaseStyles {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<Id, ThemeStyles>> for BaseStyles {
    fn from(inner: Map<Id, ThemeStyles>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(Id, ThemeStyles)> for BaseStyles {
    fn from_iter<I: IntoIterator<Item = (Id, ThemeStyles)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
