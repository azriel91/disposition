use serde::{Deserialize, Serialize};

use crate::theme::{EdgeDefaults, NodeDefaults};

/// A set of styles containing both node and edge defaults.
///
/// This is a unified type used throughout the theme system for applying
/// styles to entities. It can represent:
///
/// * Styles for a specific entity type (e.g., organisations, services, docker
///   images)
/// * Styles for focused/unfocused things during user interaction
/// * Base styles applied to specific entities by ID
///
/// # Example
///
/// ```yaml
/// node_defaults:
///   style_aliases_applied: [shade_light]
///   shape_color: "slate"
///   stroke_style: "solid"
///   stroke_width: "1"
///   visibility: "visible"
/// edge_defaults:
///   stroke_width: "1"
///   visibility: "visible"
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct StyleSet {
    /// Node style properties.
    #[serde(default, skip_serializing_if = "NodeDefaults::is_empty")]
    pub node_defaults: NodeDefaults,

    /// Edge style properties.
    #[serde(default, skip_serializing_if = "EdgeDefaults::is_empty")]
    pub edge_defaults: EdgeDefaults,
}

impl StyleSet {
    /// Returns a new `StyleSet` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if all fields are at their default values.
    pub fn is_empty(&self) -> bool {
        self.node_defaults.is_empty() && self.edge_defaults.is_empty()
    }
}
