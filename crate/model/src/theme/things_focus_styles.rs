use serde::{Deserialize, Serialize};

use crate::theme::ThemeStyles;

/// Styles for things that are included or excluded from focus.
///
/// This is used when a thing or tag is focused to differentiate between
/// things that are related (included) and things that are not (excluded).
///
/// # Example
///
/// ```yaml
/// things_included_styles:
///   node_defaults:
///     visibility: "visible"
///     opacity: "1.0"
///
/// things_excluded_styles:
///   node_defaults:
///     visibility: "hidden"
///   edge_defaults:
///     visibility: "hidden"
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThingsFocusStyles {
    /// Styles applied to things that are included in the focus.
    #[serde(default, skip_serializing_if = "ThemeStyles::is_empty")]
    pub things_included_styles: ThemeStyles,

    /// Styles applied to things that are excluded from the focus.
    #[serde(default, skip_serializing_if = "ThemeStyles::is_empty")]
    pub things_excluded_styles: ThemeStyles,
}

impl ThingsFocusStyles {
    /// Returns a new `ThingsFocusStyles` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if all fields are at their default values.
    pub fn is_empty(&self) -> bool {
        self.things_included_styles.is_empty() && self.things_excluded_styles.is_empty()
    }
}
