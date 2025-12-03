use serde::{Deserialize, Serialize};

use crate::theme::things_focus_styles::FocusStyleSet;

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
///   things_excluded_styles:
///     node_defaults:
///       visibility: "hidden"
///     edge_defaults:
///       visibility: "hidden"
///   things_included_styles:
///     node_defaults:
///       visibility: "visible"
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThemeThingDependenciesStyles {
    /// Styles applied to things that are included in the dependency view.
    #[serde(default, skip_serializing_if = "FocusStyleSet::is_empty")]
    pub things_included_styles: FocusStyleSet,

    /// Styles applied to things that are excluded from the dependency view.
    #[serde(default, skip_serializing_if = "FocusStyleSet::is_empty")]
    pub things_excluded_styles: FocusStyleSet,
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
