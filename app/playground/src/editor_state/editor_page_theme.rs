//! Sub-pages within the "Theme" group tab.

use serde::{Deserialize, Serialize};

/// Identifies which sub-page within the "Theme" group is active.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, enum_iterator::Sequence)]
#[serde(rename_all = "snake_case")]
pub enum EditorPageTheme {
    /// Theme: style aliases sub-page.
    #[default]
    StyleAliases,
    /// Theme: base styles (node/edge defaults + per-entity overrides).
    BaseStyles,
    /// Theme: process-step-selected styles.
    ProcessStepStyles,
    /// Theme: type-based styles.
    TypesStyles,
    /// Theme: thing-dependencies focus styles.
    DependenciesStyles,
    /// Theme: tag-things focus styles.
    TagsFocus,
}

impl EditorPageTheme {
    /// A human-readable label for each sub-page, suitable for rendering in a
    /// sub-tab bar.
    ///
    /// e.g. `"Theme: Aliases"`, `"Theme: Base"`.
    pub fn label(&self) -> &'static str {
        match self {
            Self::StyleAliases => "Theme: Aliases",
            Self::BaseStyles => "Theme: Base",
            Self::ProcessStepStyles => "Theme: Step Styles",
            Self::TypesStyles => "Theme: Types",
            Self::DependenciesStyles => "Theme: Deps",
            Self::TagsFocus => "Theme: Tags",
        }
    }
}
