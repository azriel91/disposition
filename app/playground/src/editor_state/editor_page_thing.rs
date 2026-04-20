//! Sub-pages within the "Things" group tab.
//!
//! Note: entity tooltips have moved to the "Entity" group tab
//! ([`EditorPageEntity`](crate::editor_state::EditorPageEntity)).

use serde::{Deserialize, Serialize};

/// Identifies which sub-page within the "Things" group is active.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, enum_iterator::Sequence)]
#[serde(rename_all = "snake_case")]
pub enum EditorPageThing {
    /// Things: names (ThingId -> display label).
    #[default]
    Names,
    /// Things: clipboard text per ThingId.
    CopyText,
    /// Things: entity descriptions.
    EntityDescs,
}

impl EditorPageThing {
    /// A human-readable label for each sub-page, suitable for rendering in a
    /// sub-tab bar.
    ///
    /// e.g. `"Things: Names"`, `"Things: Copy Text"`.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Names => "Things: Names",
            Self::CopyText => "Things: Copy Text",
            Self::EntityDescs => "Things: Descriptions",
        }
    }
}
