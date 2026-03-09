//! Top-level editor page enum.
//!
//! Each variant corresponds to either a standalone page or a group that
//! contains sub-pages (e.g. `Thing` and `Theme`).

use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

use super::{EditorPageTheme, EditorPageThing};

/// Identifies which editor page (tab) is currently active.
///
/// Top-level variants map directly to top-level tabs. The `Thing` and
/// `Theme` variants wrap their own sub-page enums so that the type
/// system naturally represents the grouping.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Sequence)]
#[serde(rename_all = "snake_case")]
pub enum EditorPage {
    /// Things group: sub-pages for names, copy text, descriptions,
    /// tooltips.
    Thing(EditorPageThing),
    /// Thing layout: interactive tree editor for `thing_hierarchy`.
    ThingLayout,
    /// Thing dependencies: edge groups with
    /// [`EdgeGroup`](disposition::input_model::edge::EdgeGroup) entries.
    ThingDependencies,
    /// Thing interactions: edge groups representing runtime
    /// communication.
    ThingInteractions,
    /// Processes: process diagrams with steps and
    /// step-thing-interaction mappings.
    Processes,
    /// Tags: tag names and the things associated with each tag.
    Tags,
    /// Entity Types: entity type assignments for common styling.
    EntityTypes,
    /// Theme group: sub-pages for style aliases, base styles, etc.
    Theme(EditorPageTheme),
    /// Raw YAML text editor.
    Text,
}

impl Default for EditorPage {
    fn default() -> Self {
        Self::Thing(EditorPageThing::default())
    }
}

impl EditorPage {
    /// Returns the top-level tabs in display order.
    ///
    /// Each entry is the default (first) sub-page for grouped variants,
    /// or the standalone variant itself. The result is computed from the
    /// [`Sequence`] iterator, deduplicating by top-level discriminant,
    /// so new variants are picked up automatically.
    pub fn top_level_pages() -> Vec<EditorPage> {
        let mut pages = Vec::new();
        for page in enum_iterator::all::<EditorPage>() {
            if !pages.iter().any(|p: &EditorPage| p.same_top_level(&page)) {
                pages.push(page);
            }
        }
        pages
    }

    /// A human-readable label for each page, suitable for rendering in
    /// a tab bar.
    ///
    /// For grouped pages, this returns the sub-page label (e.g.
    /// `"Things: Names"`). Use [`top_level_label`](Self::top_level_label)
    /// for the group heading.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Thing(sub) => sub.label(),
            Self::ThingLayout => "Layout",
            Self::ThingDependencies => "Dependencies",
            Self::ThingInteractions => "Interactions",
            Self::Processes => "Processes",
            Self::Tags => "Tags",
            Self::EntityTypes => "Entity Types",
            Self::Theme(sub) => sub.label(),
            Self::Text => "Text",
        }
    }

    /// The label shown on the top-level tab.
    ///
    /// For `Thing(_)` this returns `"Things"`, for `Theme(_)` this
    /// returns `"Theme"`, otherwise delegates to [`label`](Self::label).
    pub fn top_level_label(&self) -> &'static str {
        match self {
            Self::Thing(_) => "Things",
            Self::Theme(_) => "Theme",
            other => other.label(),
        }
    }

    /// Returns `true` if this page belongs to the Things group.
    pub fn is_thing(&self) -> bool {
        matches!(self, Self::Thing(_))
    }

    /// Returns `true` if this page belongs to the Theme group.
    pub fn is_theme(&self) -> bool {
        matches!(self, Self::Theme(_))
    }

    /// Returns `true` if `self` falls under the same top-level tab as
    /// `other`.
    ///
    /// Two `Thing(_)` pages share a top-level tab, two `Theme(_)` pages
    /// share a top-level tab, and standalone pages match only
    /// themselves.
    pub fn same_top_level(&self, other: &EditorPage) -> bool {
        match (self, other) {
            (Self::Thing(_), Self::Thing(_)) => true,
            (Self::Theme(_), Self::Theme(_)) => true,
            _ => std::mem::discriminant(self) == std::mem::discriminant(other),
        }
    }

    /// Returns the 0-based index into
    /// [`top_level_pages`](Self::top_level_pages) that this page belongs
    /// to.
    ///
    /// e.g. any `Thing(_)` variant returns `Some(0)`, `ThingLayout`
    /// returns `Some(1)`.
    #[cfg(test)]
    pub fn top_level_index(&self) -> Option<usize> {
        Self::top_level_pages()
            .iter()
            .position(|entry| entry.same_top_level(self))
    }

    /// Returns the default page for a given top-level index (0-based).
    ///
    /// Returns `None` if `index` is out of range.
    pub fn default_page(index: usize) -> Option<EditorPage> {
        Self::top_level_pages().into_iter().nth(index)
    }
}
