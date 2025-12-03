use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{common::Map, entity_type::EntityTypeId, theme::TypeStyles};

/// Styles applied to things / edges of a particular `type` specified in
/// `entity_types`.
///
/// This map contains both built-in default types and custom user-defined types.
///
/// Built-in types that may be overridden:
///
/// * `type_thing_default`
/// * `type_tag_default`
/// * `type_process_default`
/// * `type_process_step_default`
/// * `type_edge_dependency_sequence_request_default`
/// * `type_edge_dependency_sequence_response_default`
/// * `type_edge_dependency_cyclic_default`
/// * `type_edge_interaction_sequence_request_default`
/// * `type_edge_interaction_sequence_response_default`
/// * `type_edge_interaction_cyclic_default`
///
/// # Example
///
/// ```yaml
/// theme_types_styles:
///   type_thing_default:
///     node_defaults:
///       style_aliases_applied: [shade_light]
///       stroke_style: "solid"
///       shape_color: "slate"
///       stroke_width: "1"
///   type_tag_default:
///     node_defaults:
///       style_aliases_applied: [shade_medium]
///       stroke_style: "solid"
///       shape_color: "emerald"
///       stroke_width: "1"
///   type_organisation:
///     node_defaults:
///       style_aliases_applied: [shade_pale]
///       stroke_style: "dotted"
///   type_service:
///     node_defaults:
///       stroke_style: "dashed"
///   type_docker_image:
///     node_defaults:
///       shape_color: "sky"
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThemeTypesStyles(Map<EntityTypeId, TypeStyles>);

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
    pub fn into_inner(self) -> Map<EntityTypeId, TypeStyles> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for ThemeTypesStyles {
    type Target = Map<EntityTypeId, TypeStyles>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ThemeTypesStyles {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<EntityTypeId, TypeStyles>> for ThemeTypesStyles {
    fn from(inner: Map<EntityTypeId, TypeStyles>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(EntityTypeId, TypeStyles)> for ThemeTypesStyles {
    fn from_iter<I: IntoIterator<Item = (EntityTypeId, TypeStyles)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
