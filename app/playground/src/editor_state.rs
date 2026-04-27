//! Editor state that is serialized into the URL hash.
//!
//! This module defines the [`EditorState`] struct, which captures the full
//! state of the playground editor: both the [`InputDiagram`] being edited and
//! the currently active editor page. The struct implements
//! [`FromStr`](std::str::FromStr) and [`Display`](std::fmt::Display) so that
//! `dioxus_router` can round-trip it through the URL hash
//! fragment.

mod editor_page;
mod editor_page_entity;
mod editor_page_theme;
mod editor_page_thing;

pub use self::{
    editor_page::EditorPage, editor_page_entity::EditorPageEntity,
    editor_page_theme::EditorPageTheme, editor_page_thing::EditorPageThing,
};

use std::fmt;

use disposition::input_model::InputDiagram;
use serde::{Deserialize, Serialize};

/// Full state of the playground editor, persisted in the URL hash.
///
/// When serialized (via [`Display`](std::fmt::Display)), the struct is written
/// as YAML. When deserialized (via [`FromStr`](std::str::FromStr)), the YAML
/// is parsed back.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorState {
    /// The currently active editor page / tab.
    #[serde(default)]
    pub page: EditorPage,

    /// The `data-input-diagram-field` value of the field to focus on page
    /// load.
    ///
    /// When a URL is opened with this field set, the editor will:
    ///
    /// 1. Expand any collapsible cards whose field ID is a prefix of this value
    ///    (so that nested fields are visible).
    /// 2. Focus the DOM element matching
    ///    `[data-input-diagram-field="{value}"]`.
    /// 3. Clear the field from the URL after focusing, so that subsequent edits
    ///    do not keep re-focusing.
    ///
    /// # Examples
    ///
    /// * `Some("proc_app_dev")`: focus the process card with ID `proc_app_dev`.
    /// * `Some("proc_app_dev_step_2")`: expand the `proc_app_dev` card, then
    ///   focus the third step row inside it.
    /// * `None`: no field to focus (the default).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus_field: Option<String>,

    /// Whether the SVG preview should be expanded to fill the entire
    /// viewport, hiding the editor panels and navigation.
    ///
    /// Defaults to `false`. Only serialized when `true`, so most URLs
    /// remain compact.
    ///
    /// # Examples
    ///
    /// * `true`: the SVG preview fills the page on load.
    /// * `false`: normal editor layout (the default).
    #[serde(default, skip_serializing_if = "is_false")]
    pub svg_preview_expanded: bool,

    /// The input diagram being edited.
    #[serde(default)]
    pub input_diagram: InputDiagram<'static>,
}

/// Helper for `skip_serializing_if` on `bool` fields.
fn is_false(v: &bool) -> bool {
    !v
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

    #[test]
    fn round_trip_all_pages() {
        for page in enum_iterator::all::<EditorPage>() {
            let state = EditorState {
                page: page.clone(),
                focus_field: None,
                svg_preview_expanded: false,
                input_diagram: InputDiagram::default(),
            };
            let yaml = state.to_string();
            let parsed: EditorState = yaml.parse().unwrap_or_else(|e| {
                panic!("Failed to round-trip page {:?}: {e}", page);
            });
            assert_eq!(state.page, parsed.page, "Page mismatch for {:?}", page);
        }
    }

    #[test]
    fn top_level_index_returns_correct_indices() {
        let top_level = EditorPage::top_level_pages();
        assert_eq!(
            EditorPage::Thing(EditorPageThing::Names).top_level_index(),
            Some(0)
        );
        assert_eq!(
            EditorPage::Thing(EditorPageThing::CopyText).top_level_index(),
            Some(0)
        );
        assert_eq!(EditorPage::ThingLayout.top_level_index(), Some(1));
        assert_eq!(
            EditorPage::Text.top_level_index(),
            Some(top_level.len() - 1)
        );
        assert_eq!(
            EditorPage::Theme(EditorPageTheme::BaseStyles).top_level_index(),
            EditorPage::Theme(EditorPageTheme::StyleAliases).top_level_index(),
        );
    }

    #[test]
    fn focus_field_none_not_serialized() {
        let state = EditorState {
            page: EditorPage::default(),
            focus_field: None,
            svg_preview_expanded: false,
            input_diagram: InputDiagram::default(),
        };
        let yaml = state.to_string();
        assert!(
            !yaml.contains("focus_field"),
            "focus_field: None should be omitted from serialized YAML, got:\n{yaml}"
        );
    }

    #[test]
    fn focus_field_some_round_trips() {
        let state = EditorState {
            page: EditorPage::Processes,
            focus_field: Some(String::from("proc_app_dev_step_2")),
            svg_preview_expanded: false,
            input_diagram: InputDiagram::default(),
        };
        let yaml = state.to_string();
        assert!(
            yaml.contains("focus_field"),
            "focus_field: Some(_) should appear in serialized YAML, got:\n{yaml}"
        );
        let parsed: EditorState = yaml.parse().expect("parse failed");
        assert_eq!(
            state.focus_field, parsed.focus_field,
            "focus_field must survive round-trip"
        );
        assert_eq!(state.page, parsed.page);
    }

    #[test]
    fn focus_field_missing_in_yaml_defaults_to_none() {
        let yaml_input = "\
page: thing_dependencies
input_diagram:
  things: {}
";
        let state: EditorState = yaml_input.parse().expect("parse failed");
        assert_eq!(
            state.focus_field, None,
            "Missing focus_field should default to None"
        );
    }

    #[test]
    fn svg_preview_expanded_false_not_serialized() {
        let state = EditorState {
            page: EditorPage::default(),
            focus_field: None,
            svg_preview_expanded: false,
            input_diagram: InputDiagram::default(),
        };
        let yaml = state.to_string();
        assert!(
            !yaml.contains("svg_preview_expanded"),
            "svg_preview_expanded: false should be omitted from serialized YAML, got:\n{yaml}"
        );
    }

    #[test]
    fn svg_preview_expanded_true_round_trips() {
        let state = EditorState {
            page: EditorPage::Processes,
            focus_field: None,
            svg_preview_expanded: true,
            input_diagram: InputDiagram::default(),
        };
        let yaml = state.to_string();
        assert!(
            yaml.contains("svg_preview_expanded"),
            "svg_preview_expanded: true should appear in serialized YAML, got:\n{yaml}"
        );
        let parsed: EditorState = yaml.parse().expect("parse failed");
        assert_eq!(
            state.svg_preview_expanded, parsed.svg_preview_expanded,
            "svg_preview_expanded must survive round-trip"
        );
    }

    #[test]
    fn svg_preview_expanded_missing_in_yaml_defaults_to_false() {
        let yaml_input = "\
page: thing_dependencies
input_diagram:
  things: {}
";
        let state: EditorState = yaml_input.parse().expect("parse failed");
        assert!(
            !state.svg_preview_expanded,
            "Missing svg_preview_expanded should default to false"
        );
    }

    #[test]
    fn top_level_pages_covers_all_variants() {
        let top_level = EditorPage::top_level_pages();
        // Every variant from enum_iterator must map to some top-level
        // entry.
        for page in enum_iterator::all::<EditorPage>() {
            assert!(
                top_level.iter().any(|tl| tl.same_top_level(&page)),
                "Page {:?} has no top-level entry",
                page
            );
        }
    }
}
