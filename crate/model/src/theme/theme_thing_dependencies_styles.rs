use serde::{Deserialize, Serialize};

use crate::theme::ThemeStyles;

/// Styles when a `thing` is focused to show its dependencies.
///
/// Depending on which button is pressed, when a `thing` is focused, these same
/// styles may be used to show:
///
/// * Predecessors / successors linked to this `thing`.
/// * Immediate dependencies vs transitive (maybe closest `n` neighbours).
///
/// # Example
///
/// ```yaml
/// theme_thing_dependencies_styles:
///   things_included_styles: # <-- this is a `ThemeStyles`
///     node_defaults:
///       visibility: "visible"
///   edge_defaults:
///       visibility: "visible"
///
///   things_excluded_styles: # <-- this is a `ThemeStyles`
///     node_defaults:
///       visibility: "hidden"
///     edge_defaults:
///       visibility: "hidden"
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThemeThingDependenciesStyles {
    /// Styles applied to things that are included in the dependency view.
    #[serde(default, skip_serializing_if = "ThemeStyles::is_empty")]
    pub things_included_styles: ThemeStyles,

    /// Styles applied to things that are excluded from the dependency view.
    #[serde(default, skip_serializing_if = "ThemeStyles::is_empty")]
    pub things_excluded_styles: ThemeStyles,
}

impl ThemeThingDependenciesStyles {
    /// Returns a new `ThemeThingDependenciesStyles` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if all fields are at their default values.
    pub fn is_empty(&self) -> bool {
        self.things_included_styles.is_empty() && self.things_excluded_styles.is_empty()
    }
}
