//! Sub-pages within the "Theme" group tab.

use serde::{Deserialize, Serialize};

/// Identifies which sub-page within the "Theme" group is active.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, enum_iterator::Sequence)]
#[serde(rename_all = "snake_case")]
pub enum EditorPageTheme {
    #[default]
    /// Theme: base styles (node/edge defaults + per-entity overrides).
    BaseStyles,
    /// Theme: type-based styles.
    TypesStyles,
    /// Theme: process-step-selected styles.
    ProcessStepStyles,
    /// Theme: thing-dependencies focus styles.
    DependenciesStyles,
    /// Theme: tag-things focus styles.
    TagsFocus,
    /// Theme: style aliases sub-page.
    StyleAliases,
}

impl EditorPageTheme {
    /// A human-readable label for each sub-page, suitable for rendering in a
    /// sub-tab bar.
    ///
    /// e.g. `"Theme: Aliases"`, `"Theme: Base"`.
    pub fn label(&self) -> &'static str {
        match self {
            Self::BaseStyles => "Theme: Base",
            Self::TypesStyles => "Theme: Types",
            Self::ProcessStepStyles => "Theme: Step Styles",
            Self::DependenciesStyles => "Theme: Deps",
            Self::TagsFocus => "Theme: Tags",
            Self::StyleAliases => "Theme: Aliases",
        }
    }
}
