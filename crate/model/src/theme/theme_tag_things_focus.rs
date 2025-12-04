use serde::{Deserialize, Serialize};

use crate::theme::StyleSet;

/// Styles when a tag is focused, applied to all tags uniformly.
///
/// When a tag is focused, things and edges associated with the tag are
/// highlighted. This struct defines the styles applied to things that are
/// included in or excluded from the tag.
///
/// For tag-specific styling, use `ThemeTagThingsFocusSpecific`.
///
/// # Example
///
/// ```yaml
/// things_included_styles:
///   node_defaults:
///     opacity: "1.0"
///   edge_defaults:
///     opacity: "1.0"
///
/// things_excluded_styles:
///   node_defaults:
///     opacity: "0.3"
///   edge_defaults:
///     opacity: "0.3"
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThemeTagThingsFocus {
    /// Styles applied to things that are associated with the focused tag.
    #[serde(default, skip_serializing_if = "StyleSet::is_empty")]
    pub things_included_styles: StyleSet,

    /// Styles applied to things that are not associated with the focused tag.
    #[serde(default, skip_serializing_if = "StyleSet::is_empty")]
    pub things_excluded_styles: StyleSet,
}

impl ThemeTagThingsFocus {
    /// Returns a new `ThemeTagThingsFocus` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if all fields are at their default values.
    pub fn is_empty(&self) -> bool {
        self.things_included_styles.is_empty() && self.things_excluded_styles.is_empty()
    }
}
