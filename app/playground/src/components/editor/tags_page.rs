//! Tags editor page.
//!
//! Allows editing:
//! - `tags`: `TagId` -> display name
//! - `tag_things`: `TagId` -> `Set<ThingId>`
//!
//! `TagNameRow` supports keyboard shortcuts:
//!
//! - **Up / Down** (on row): move focus to the previous / next row.
//! - **Alt+Up / Alt+Down**: move the entry up or down in the list.
//! - **Enter** (on row): focus the first input inside the row for editing.
//! - **Escape** (on row): focus the parent section / tab.
//! - **Tab / Shift+Tab** (inside an input or remove button): cycle through
//!   interactive elements within the same row. Wraps from last to first / first
//!   to last.
//! - **Esc** (inside an input or remove button): return focus to the parent
//!   row.
//! - **Space** (inside an input or remove button): stop propagation.
//!
//! `TagThingsCard` supports keyboard shortcuts:
//!
//! - **ArrowUp / ArrowDown**: navigate between sibling cards.
//! - **ArrowRight**: expand the card (when collapsed).
//! - **ArrowLeft**: collapse the card (when expanded).
//! - **Space**: toggle expand/collapse.
//! - **Enter**: expand + focus the first input inside the card.
//! - **Escape**: focus the parent section / tab.
//! - **Tab / Shift+Tab** (inside a field): cycle through focusable fields
//!   within the card. Wraps from last to first / first to last.
//! - **Esc** (inside a field): return focus to the card wrapper.
//!
//! The heavy lifting is delegated to submodules:
//! - [`tag_things_card`]: collapsible card for a single tag's thing set.
//! - [`tags_page_ops`]: mutation helpers for the page-level tag maps.

mod tag_things_card;
mod tags_page_ops;

use dioxus::{
    hooks::use_signal,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal},
};
use disposition::input_model::InputDiagram;

use crate::components::editor::{
    common::{RenameRefocus, ADD_BTN, INPUT_CLASS, SECTION_HEADING},
    datalists::list_ids,
    id_value_row::IdValueRow,
    id_value_row_container::IdValueRowContainer,
};

use self::{tag_things_card::TagThingsCard, tags_page_ops::TagsPageOps};

// === TagThingsCard constants === //

/// The `data-*` attribute placed on each `TagThingsCard` wrapper.
///
/// Used by [`keyboard_nav`](crate::components::editor::keyboard_nav) helpers
/// to locate the nearest ancestor card.
pub(crate) const DATA_ATTR: &str = "data-tag-things-card";

/// The `data-*` attribute that holds the card's ID value (for post-rename
/// focus).
pub(crate) const DATA_ID_ATTR: &str = "data-tag-things-card-id";

// === TagThingsCard CSS === //

/// CSS classes for the focusable tag-things card wrapper.
///
/// Extends the standard card styling with focus ring and transitions.
pub(crate) const TAG_THINGS_CARD_CLASS: &str = "\
    rounded-lg \
    border \
    border-gray-700 \
    bg-gray-900 \
    p-3 \
    mb-2 \
    flex \
    flex-col \
    gap-2 \
    focus:outline-none \
    focus:ring-1 \
    focus:ring-blue-400 \
    transition-all \
    duration-150\
";

/// CSS classes for the collapsed summary header.
pub(crate) const COLLAPSED_HEADER_CLASS: &str = "\
    flex \
    flex-row \
    items-center \
    gap-3 \
    cursor-pointer \
    select-none\
";

/// CSS classes for an input inside a tag-things card.
///
/// These elements use `tabindex="-1"` so they are skipped by the normal tab
/// order; the user enters edit mode by pressing Enter on the focused card.
pub(crate) const FIELD_INPUT_CLASS: &str = INPUT_CLASS;

// === TagsPage component === //

/// The **Tags** editor page.
///
/// Provides two sections:
/// 1. Tag Names: map of `TagId` to display label.
/// 2. Tag Things: map of `TagId` to set of `ThingId`s associated with the tag.
#[component]
pub fn TagsPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    // Focus-after-move state for tag name row reorder.
    let tag_name_focus_idx: Signal<Option<usize>> = use_signal(|| None);
    // Post-rename focus state for tag name rows.
    let tag_name_rename_refocus: Signal<Option<RenameRefocus>> = use_signal(|| None);
    // Post-rename focus state for tag-things cards.
    let tag_things_rename_refocus: Signal<Option<RenameRefocus>> = use_signal(|| None);

    // Drag-and-drop state for tag name rows.
    let tag_name_drag_idx: Signal<Option<usize>> = use_signal(|| None);
    let tag_name_drop_target: Signal<Option<usize>> = use_signal(|| None);

    let diagram = input_diagram.read();

    // Snapshot tag names.
    let tag_entries: Vec<(String, String)> = diagram
        .tags
        .iter()
        .map(|(tag_id, name)| (tag_id.as_str().to_owned(), name.clone()))
        .collect();

    // Snapshot tag -> things associations.
    let tag_things_entries: Vec<(String, Vec<String>)> = diagram
        .tag_things
        .iter()
        .map(|(tag_id, thing_ids)| {
            let things: Vec<String> = thing_ids.iter().map(|t| t.as_str().to_owned()).collect();
            (tag_id.as_str().to_owned(), things)
        })
        .collect();

    drop(diagram);

    let tag_count = tag_entries.len();

    rsx! {
        div {
            class: "flex flex-col gap-2",

            // === Tag Names === //
            h3 { class: SECTION_HEADING, "Tag Names" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Map of TagId to display label."
            }

            IdValueRowContainer {
                section_id: "tag_names",
                focus_index: tag_name_focus_idx,
                rename_refocus: tag_name_rename_refocus,

                for (idx, (tag_id, tag_name)) in tag_entries.iter().enumerate() {
                    {
                        let tag_id = tag_id.clone();
                        let tag_name = tag_name.clone();
                        rsx! {
                            IdValueRow {
                                key: "{tag_id}",
                                entry_id: tag_id,
                                entry_value: tag_name,
                                id_list: list_ids::TAG_IDS.to_owned(),
                                id_placeholder: "tag_id".to_owned(),
                                value_placeholder: "Display name".to_owned(),
                                index: idx,
                                entry_count: tag_count,
                                drag_index: tag_name_drag_idx,
                                drop_target: tag_name_drop_target,
                                focus_index: tag_name_focus_idx,
                                rename_refocus: tag_name_rename_refocus,
                                on_move: move |(from, to)| {
                                    TagsPageOps::tag_move(input_diagram, from, to);
                                },
                                on_rename: move |(id_old, id_new): (String, String)| {
                                    TagsPageOps::tag_rename(input_diagram, &id_old, &id_new);
                                },
                                on_update: move |(id, value): (String, String)| {
                                    TagsPageOps::tag_name_update(input_diagram, &id, &value);
                                },
                                on_remove: move |id: String| {
                                    TagsPageOps::tag_remove(input_diagram, &id);
                                },
                            }
                        }
                    }
                }
            }

            button {
                class: ADD_BTN,
                tabindex: -1,
                onclick: move |_| {
                    TagsPageOps::tag_add(input_diagram);
                },
                "+ Add tag"
            }

            // === Tag Things === //
            h3 { class: SECTION_HEADING, "Tag -> Things" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Things highlighted when each tag is focused."
            }

            for (tag_id, things) in tag_things_entries.iter() {
                {
                    let tag_id = tag_id.clone();
                    let things = things.clone();
                    rsx! {
                        TagThingsCard {
                            key: "tt_{tag_id}",
                            input_diagram,
                            tag_id,
                            things,
                            rename_refocus: tag_things_rename_refocus,
                        }
                    }
                }
            }

            button {
                class: ADD_BTN,
                tabindex: -1,
                onclick: move |_| {
                    TagsPageOps::tag_things_entry_add(input_diagram);
                },
                "+ Add tag -> things mapping"
            }
        }
    }
}
