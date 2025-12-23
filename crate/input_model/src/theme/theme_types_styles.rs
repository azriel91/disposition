use std::ops::{Deref, DerefMut};

use disposition_model_common::Map;
use serde::{Deserialize, Serialize};

use crate::{entity::EntityTypeId, theme::ThemeStyles};

/// Styles applied to things / edges of a particular `type` specified in
/// `entity_types`.
///
/// This map contains both built-in default types and custom user-defined types.
///
/// ## Built-in Types
///
/// * `type_organisation`: Parent container for services
/// * `type_service`: A deployable service
/// * `type_docker_image`: Docker container image
/// * `type_dependency_edge_sequence_forward_default`: Default edge style for
///   dependency sequence edges
///
/// ## Custom Types
///
/// Users can define their own types in `entity_types` and reference them here.
///
/// # Example
///
/// ```yaml
/// theme_types_styles:
///   type_organisation: # <-- this is a `ThemeStyles`
///     node_defaults:
///       style_aliases_applied: [shade_pale]
///       stroke_style: "dotted"
///
///   type_service:
///     node_defaults:
///       stroke_style: "dashed"
///
///   type_docker_image:
///     node_defaults:
///       shape_color: "sky"
///
///   type_dependency_edge_sequence_forward_default:
///     edge_defaults:
///       style_aliases_applied: [shade_dark]
///       stroke_style: solid
///       shape_color: "neutral"
///       stroke_width: "1"
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThemeTypesStyles(Map<EntityTypeId, ThemeStyles>);

impl ThemeTypesStyles {
    /// Returns a new empty `ThemeTypesStyles` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ThemeTypesStyles` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<EntityTypeId, ThemeStyles> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for ThemeTypesStyles {
    type Target = Map<EntityTypeId, ThemeStyles>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ThemeTypesStyles {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<EntityTypeId, ThemeStyles>> for ThemeTypesStyles {
    fn from(inner: Map<EntityTypeId, ThemeStyles>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(EntityTypeId, ThemeStyles)> for ThemeTypesStyles {
    fn from_iter<I: IntoIterator<Item = (EntityTypeId, ThemeStyles)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
