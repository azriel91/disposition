//! Sub-pages within the "Edges" group tab.

use serde::{Deserialize, Serialize};

/// Identifies which sub-page within the "Edges" group is active.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, enum_iterator::Sequence)]
#[serde(rename_all = "snake_case")]
pub enum EditorPageEdges {
    /// Edges: dependencies -- edge groups with
    /// [`EdgeGroup`](disposition::input_model::edge::EdgeGroup) entries.
    #[default]
    ThingDependencies,
    /// Edges: interactions -- edge groups representing runtime communication.
    ThingInteractions,
    /// Edges: labels rendered next to edges where they exit or enter a node.
    EdgeLabels,
}

impl EditorPageEdges {
    /// A human-readable label for each sub-page, suitable for rendering in a
    /// sub-tab bar.
    ///
    /// e.g. `"Edges: Dependencies"`, `"Edges: Interactions"`.
    pub fn label(&self) -> &'static str {
        match self {
            Self::ThingDependencies => "Edges: Dependencies",
            Self::ThingInteractions => "Edges: Interactions",
            Self::EdgeLabels => "Edges: Labels",
        }
    }
}
