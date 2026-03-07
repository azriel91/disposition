//! Editor state that is serialized into the URL hash.
//!
//! This module defines the [`EditorState`] struct, which captures the full
//! state of the playground editor: both the [`InputDiagram`] being edited and
//! the currently active editor page. The struct implements
//! [`FromStr`](std::str::FromStr) and [`Display`](std::fmt::Display) so that
//! `dioxus_router` can round-trip it through the URL hash
//! fragment.

use std::fmt;

use disposition::input_model::InputDiagram;
use serde::{Deserialize, Serialize};

/// Full state of the playground editor, persisted in the URL hash.
///
/// When serialized (via [`Display`](std::fmt::Display)), the struct is written
/// as YAML. When deserialized (via [`FromStr`](std::str::FromStr)), the YAML
/// is parsed back.
///
/// For backward compatibility with older URLs that contain only a raw
/// [`InputDiagram`] YAML (without the wrapping `EditorState`),
/// [`FromStr`](std::str::FromStr)
/// will fall back to parsing the string as an `InputDiagram` directly.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorState {
    /// The currently active editor page / tab.
    #[serde(default)]
    pub page: EditorPage,

    /// The input diagram being edited.
    #[serde(default)]
    pub input_diagram: InputDiagram<'static>,
}

/// Identifies which editor page (tab) is currently active.
///
/// The YAML representation uses `snake_case` names so that the URL hash is
/// human-readable.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditorPage {
    /// Things: names (ThingId -> display label).
    #[default]
    ThingNames,
    /// Things: clipboard text per ThingId.
    ThingCopyText,
    /// Things: entity descriptions.
    ThingEntityDescs,
    /// Things: entity tooltips.
    ThingEntityTooltips,
    /// Thing layout: interactive tree editor for `thing_hierarchy`.
    ThingLayout,
    /// Thing dependencies: edge groups with
    /// [`EdgeGroup`](disposition::input_model::edge::EdgeGroup) entries.
    ThingDependencies,
    /// Thing interactions: edge groups representing runtime communication.
    ThingInteractions,
    /// Processes: process diagrams with steps and step-thing-interaction
    /// mappings.
    Processes,
    /// Tags: tag names and the things associated with each tag.
    Tags,
    /// Theme: style aliases sub-page.
    ThemeStyleAliases,
    /// Theme: base styles (node/edge defaults + per-entity overrides).
    ThemeBaseStyles,
    /// Theme: process-step-selected styles.
    ThemeProcessStepStyles,
    /// Theme: type-based styles.
    ThemeTypesStyles,
    /// Theme: thing-dependencies focus styles.
    ThemeDependenciesStyles,
    /// Theme: tag-things focus styles.
    ThemeTagsFocus,
    /// Raw YAML text editor.
    Text,
}

impl EditorPage {
    /// The sub-pages within the Theme group.
    pub const THEME_SUB_PAGES: &'static [EditorPage] = &[
        Self::ThemeStyleAliases,
        Self::ThemeBaseStyles,
        Self::ThemeProcessStepStyles,
        Self::ThemeTypesStyles,
        Self::ThemeDependenciesStyles,
        Self::ThemeTagsFocus,
    ];
    /// The sub-pages within the Things group.
    pub const THINGS_SUB_PAGES: &'static [EditorPage] = &[
        Self::ThingNames,
        Self::ThingCopyText,
        Self::ThingEntityDescs,
        Self::ThingEntityTooltips,
    ];
    /// Pages that appear as top-level tabs (Things and Theme pages are
    /// grouped under single parent tabs).
    pub const TOP_LEVEL: &'static [EditorPageOrGroup] = &[
        EditorPageOrGroup::ThingsGroup,
        EditorPageOrGroup::Page(Self::ThingLayout),
        EditorPageOrGroup::Page(Self::ThingDependencies),
        EditorPageOrGroup::Page(Self::ThingInteractions),
        EditorPageOrGroup::Page(Self::Processes),
        EditorPageOrGroup::Page(Self::Tags),
        EditorPageOrGroup::ThemeGroup,
        EditorPageOrGroup::Page(Self::Text),
    ];

    /// A human-readable label for each page, suitable for rendering in a tab
    /// bar.
    pub fn label(&self) -> &'static str {
        match self {
            Self::ThingNames => "Things: Names",
            Self::ThingCopyText => "Things: Copy Text",
            Self::ThingEntityDescs => "Things: Descriptions",
            Self::ThingEntityTooltips => "Things: Tooltips",
            Self::ThingLayout => "Layout",
            Self::ThingDependencies => "Dependencies",
            Self::ThingInteractions => "Interactions",
            Self::Processes => "Processes",
            Self::Tags => "Tags",
            Self::ThemeStyleAliases => "Theme: Aliases",
            Self::ThemeBaseStyles => "Theme: Base",
            Self::ThemeProcessStepStyles => "Theme: Step Styles",
            Self::ThemeTypesStyles => "Theme: Types",
            Self::ThemeDependenciesStyles => "Theme: Deps",
            Self::ThemeTagsFocus => "Theme: Tags",
            Self::Text => "Text",
        }
    }

    /// Returns `true` if this page belongs to the Things group.
    pub fn is_things(&self) -> bool {
        matches!(
            self,
            Self::ThingNames
                | Self::ThingCopyText
                | Self::ThingEntityDescs
                | Self::ThingEntityTooltips
        )
    }

    /// Returns `true` if this page belongs to the Theme group.
    pub fn is_theme(&self) -> bool {
        matches!(
            self,
            Self::ThemeStyleAliases
                | Self::ThemeBaseStyles
                | Self::ThemeProcessStepStyles
                | Self::ThemeTypesStyles
                | Self::ThemeDependenciesStyles
                | Self::ThemeTagsFocus
        )
    }
}

/// A top-level tab can either be a single [`EditorPage`] or a group
/// (which contains its own sub-tabs).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorPageOrGroup {
    /// A single editor page.
    Page(EditorPage),
    /// The "Things" group, rendered with sub-tabs.
    ThingsGroup,
    /// The "Theme" group, rendered with sub-tabs.
    ThemeGroup,
}

impl EditorPageOrGroup {
    /// Label for the top-level tab.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Page(page) => page.label(),
            Self::ThingsGroup => "Things",
            Self::ThemeGroup => "Theme",
        }
    }

    /// Returns `true` if the given [`EditorPage`] is "inside" this
    /// top-level entry.
    pub fn contains(&self, page: &EditorPage) -> bool {
        match self {
            Self::Page(p) => p == page,
            Self::ThingsGroup => page.is_things(),
            Self::ThemeGroup => page.is_theme(),
        }
    }
}

// === Display / FromStr: used by dioxus_router for the URL hash fragment === //

impl fmt::Display for EditorState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let yaml = serde_saphyr::to_string(self).map_err(|_| fmt::Error)?;
        // `serde_saphyr::to_string` may include a trailing newline; trim it
        // so the URL hash stays tidy.
        f.write_str(yaml.trim())
    }
}

impl std::str::FromStr for EditorState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Self::default());
        }

        // Try to parse as a full `EditorState`.
        serde_saphyr::from_str::<EditorState>(s)
            .map_err(|e| format!("Failed to parse URL hash: {e}"))
    }
}

// === Tests === //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_default() {
        let state = EditorState::default();
        let yaml = state.to_string();
        let parsed: EditorState = yaml.parse().expect("parse failed");
        assert_eq!(state.input_diagram, parsed.input_diagram);
    }

    #[test]
    fn empty_string_yields_default() {
        let state: EditorState = "".parse().expect("parse failed");
        assert_eq!(state, EditorState::default());
    }

    /// Verify that YAML round-tripping through `EditorState` preserves
    /// `IndexMap` key ordering for maps like `thing_dependencies`.
    ///
    /// If the YAML serializer / deserializer loses map ordering, then
    /// redo after a reorder operation would appear to do nothing because
    /// the "sync incoming EditorState -> local signals" memo would
    /// overwrite the reordered diagram with a non-reordered copy.
    #[test]
    fn round_trip_preserves_map_order() {
        // Build a diagram where `thing_dependencies` has three entries
        // in a specific order: edge_c, edge_a, edge_b.
        let yaml_input = "\
page: thing_dependencies
input_diagram:
  thing_dependencies:
    edge_c:
      kind: sequence
      things:
        - t_a
        - t_b
    edge_a:
      kind: cyclic
      things:
        - t_b
        - t_c
    edge_b:
      kind: sequence
      things:
        - t_a
        - t_c
";
        let state: EditorState = yaml_input.parse().expect("parse original");
        let keys_before: Vec<String> = state
            .input_diagram
            .thing_dependencies
            .keys()
            .map(|k| k.to_string())
            .collect();
        assert_eq!(keys_before, vec!["edge_c", "edge_a", "edge_b"]);

        // Round-trip through Display -> FromStr (the path taken by the
        // URL hash serialization).
        let serialized = state.to_string();
        let parsed: EditorState = serialized.parse().expect("parse round-tripped");

        let keys_after: Vec<String> = parsed
            .input_diagram
            .thing_dependencies
            .keys()
            .map(|k| k.to_string())
            .collect();
        assert_eq!(
            keys_before, keys_after,
            "YAML round-trip must preserve IndexMap key order"
        );
        assert_eq!(state, parsed);
    }

    /// Verify that an `InputDiagram` with content in several fields
    /// survives a YAML round-trip through `EditorState` with exact
    /// order-sensitive equality, including after a reorder (move) of
    /// processes and thing dependencies.
    ///
    /// `IndexMap::PartialEq` is **order-insensitive**, so `==` treats
    /// two maps with the same entries in different orders as equal. The
    /// undo history and URL sync code must use the order-sensitive
    /// [`input_diagram_order_eq`] instead.
    ///
    /// This test verifies:
    /// 1. A reorder IS detected by `input_diagram_order_eq`.
    /// 2. The YAML round-trip preserves that ordering.
    #[test]
    fn round_trip_exact_equality_after_reorder() {
        use disposition::input_model::InputDiagram;
        use disposition_input_rt::{EdgeGroupCardOps, MapTarget, ProcessesPageOps};

        use crate::undo_history::input_diagram_order_eq;

        // Start with a diagram that has things, dependencies, and
        // processes -- enough to exercise most fields.
        let yaml_input = "\
things:
  t_a: Thing A
  t_b: Thing B
  t_c: Thing C
thing_dependencies:
  edge_ab:
    kind: sequence
    things:
      - t_a
      - t_b
  edge_bc:
    kind: sequence
    things:
      - t_b
      - t_c
  edge_ac:
    kind: cyclic
    things:
      - t_a
      - t_c
processes:
  proc_deploy:
    steps:
      step_build: Build
      step_test: Test
      step_ship: Ship
  proc_rollback:
    steps:
      step_revert: Revert
      step_verify: Verify
";
        let d_original: InputDiagram<'static> =
            serde_saphyr::from_str(yaml_input).expect("parse original");

        // Reorder: move the first dependency to the last position.
        let mut d_moved = d_original.clone();
        EdgeGroupCardOps::edge_group_move(&mut d_moved, MapTarget::Dependencies, 0, 2);

        // `IndexMap::PartialEq` is order-insensitive, so `==` says equal.
        assert_eq!(
            d_original, d_moved,
            "IndexMap PartialEq is order-insensitive (sanity check)"
        );

        // But our order-sensitive comparison detects the difference.
        assert!(
            !input_diagram_order_eq(&d_original, &d_moved),
            "input_diagram_order_eq must detect reorder"
        );

        // Reorder processes too.
        ProcessesPageOps::process_move(&mut d_moved, 0, 1);

        assert!(
            !input_diagram_order_eq(&d_original, &d_moved),
            "input_diagram_order_eq must detect process reorder"
        );

        // Wrap in EditorState and round-trip.
        let state = EditorState {
            page: EditorPage::ThingDependencies,
            input_diagram: d_moved.clone(),
        };
        let serialized = state.to_string();
        let parsed: EditorState = serialized.parse().expect("parse round-tripped");

        // The round-tripped diagram must be order-sensitively equal.
        assert!(
            input_diagram_order_eq(&d_moved, &parsed.input_diagram),
            "round-tripped InputDiagram must preserve map key order"
        );
        assert_eq!(state, parsed);

        // Also verify starting from d_original (pre-move).
        let state_orig = EditorState {
            page: EditorPage::ThingDependencies,
            input_diagram: d_original.clone(),
        };
        let serialized_orig = state_orig.to_string();
        let parsed_orig: EditorState = serialized_orig.parse().expect("parse round-tripped orig");

        assert!(
            input_diagram_order_eq(&d_original, &parsed_orig.input_diagram),
            "round-tripped InputDiagram must preserve original map key order"
        );
    }
}
