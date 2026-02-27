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

    /// UI state for the Things editor page (collapsed sections).
    #[serde(default, skip_serializing_if = "ThingsPageUiState::is_default")]
    pub things_ui: ThingsPageUiState,
}

/// UI state for the Things editor page.
///
/// Tracks which sections are collapsed so the state can be persisted in the
/// URL hash and restored on reload. Sections with more than 4 entries are
/// eligible for collapsing.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThingsPageUiState {
    /// Whether the "Thing Names" section is collapsed.
    #[serde(default, skip_serializing_if = "is_false")]
    pub thing_names_collapsed: bool,
    /// Whether the "Thing Copy Text" section is collapsed.
    #[serde(default, skip_serializing_if = "is_false")]
    pub copy_text_collapsed: bool,
    /// Whether the "Entity Descriptions" section is collapsed.
    #[serde(default, skip_serializing_if = "is_false")]
    pub entity_descs_collapsed: bool,
    /// Whether the "Entity Tooltips" section is collapsed.
    #[serde(default, skip_serializing_if = "is_false")]
    pub entity_tooltips_collapsed: bool,
}

/// Helper for `skip_serializing_if` on `bool` fields.
fn is_false(v: &bool) -> bool {
    !*v
}

impl ThingsPageUiState {
    /// Returns `true` when all fields are at their default (expanded) values.
    ///
    /// Used by `EditorState`'s `skip_serializing_if` so the field is omitted
    /// from the URL hash when nothing is collapsed.
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

/// Identifies which editor page (tab) is currently active.
///
/// The YAML representation uses `snake_case` names so that the URL hash is
/// human-readable.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditorPage {
    /// Things: names, descriptions, copy-text, hierarchy.
    #[default]
    Things,
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
    /// Pages that appear as top-level tabs (Theme pages are grouped under a
    /// single "Theme" parent tab).
    pub const TOP_LEVEL: &'static [EditorPageOrGroup] = &[
        EditorPageOrGroup::Page(Self::Things),
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
            Self::Things => "Things",
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

/// A top-level tab can either be a single [`EditorPage`] or the Theme group
/// (which contains its own sub-tabs).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorPageOrGroup {
    /// A single editor page.
    Page(EditorPage),
    /// The "Theme" group, rendered with sub-tabs.
    ThemeGroup,
}

impl EditorPageOrGroup {
    /// Label for the top-level tab.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Page(page) => page.label(),
            Self::ThemeGroup => "Theme",
        }
    }

    /// Returns `true` if the given [`EditorPage`] is "inside" this
    /// top-level entry.
    pub fn contains(&self, page: &EditorPage) -> bool {
        match self {
            Self::Page(p) => p == page,
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
}
