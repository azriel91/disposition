use serde::{Deserialize, Serialize};

use crate::theme::StyleAliasId;

/// Default style properties for nodes.
///
/// These properties control the visual appearance of nodes (things) in the
/// diagram.
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
///   opacity: "1.0"
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct NodeDefaults {
    /// Vector of style aliases to apply.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub style_aliases_applied: Vec<StyleAliasId>,

    /// Used for both fill and stroke colors.
    ///
    /// Examples: "slate", "blue", "yellow", "neutral", "emerald", "sky",
    /// "violet"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape_color: Option<String>,

    /// Style of the stroke.
    ///
    /// Examples: "solid", "dashed", "dotted", or custom dasharray like
    /// "dasharray:0,80,12,2,4,2,2,2,1,2,1,120"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_style: Option<String>,

    /// Width of the stroke.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<String>,

    /// Visibility of the node.
    ///
    /// Examples: "visible", "hidden"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,

    /// Opacity of the node.
    ///
    /// Examples: "0.5", "1.0"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<String>,
}

impl NodeDefaults {
    /// Returns a new `NodeDefaults` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if all fields are at their default values.
    pub fn is_empty(&self) -> bool {
        self.style_aliases_applied.is_empty()
            && self.shape_color.is_none()
            && self.stroke_style.is_none()
            && self.stroke_width.is_none()
            && self.visibility.is_none()
            && self.opacity.is_none()
    }
}
