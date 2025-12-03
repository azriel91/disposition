use serde::{Deserialize, Serialize};

use crate::theme::{EdgeDefaults, NodeDefaults};

/// Styles for a specific entity type.
///
/// These styles are applied to all entities of a particular type, allowing
/// common styling for things like organisations, services, docker images, etc.
///
/// # Example
///
/// ```yaml
/// type_organisation:
///   node_defaults:
///     style_aliases_applied: [shade_pale]
///     stroke_style: "dotted"
///
/// type_service:
///   node_defaults:
///     stroke_style: "dashed"
///
/// type_docker_image:
///   node_defaults:
///     shape_color: "sky"
///
/// type_edge_dependency_sequence_request_default:
///   edge_defaults:
///     style_aliases_applied: [shade_dark]
///     stroke_style: solid
///     shape_color: "neutral"
///     stroke_width: "1"
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct TypeStyles {
    /// Node style properties for this type.
    #[serde(default, skip_serializing_if = "NodeDefaults::is_empty")]
    pub node_defaults: NodeDefaults,

    /// Edge style properties for this type.
    #[serde(default, skip_serializing_if = "EdgeDefaults::is_empty")]
    pub edge_defaults: EdgeDefaults,
}

impl TypeStyles {
    /// Returns a new `TypeStyles` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if all fields are at their default values.
    pub fn is_empty(&self) -> bool {
        self.node_defaults.is_empty() && self.edge_defaults.is_empty()
    }
}
