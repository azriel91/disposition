//! Sub-pages within the "Entity" group tab.

use serde::{Deserialize, Serialize};

/// Identifies which sub-page within the "Entity" group is active.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, enum_iterator::Sequence)]
#[serde(rename_all = "snake_case")]
pub enum EditorPageEntity {
    #[default]
    /// Entity: entity type assignments for common styling.
    EntityTypes,
    /// Entity: entity tooltip text shown on hover.
    EntityTooltips,
}

impl EditorPageEntity {
    /// A human-readable label for each sub-page, suitable for rendering in a
    /// sub-tab bar.
    ///
    /// e.g. `"Entity: Types"`, `"Entity: Tooltips"`.
    pub fn label(&self) -> &'static str {
        match self {
            Self::EntityTypes => "Entity: Types",
            Self::EntityTooltips => "Entity: Tooltips",
        }
    }
}
