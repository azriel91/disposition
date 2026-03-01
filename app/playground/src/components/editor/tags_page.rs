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
//! - **Tab / Shift+Tab** (inside an input or remove button): cycle through
//!   interactive elements within the same row, returning to the row when
//!   exhausted.
//! - **Esc** (inside an input or remove button): return focus to the parent
//!   row.
//!
//! `TagThingsCard` supports keyboard shortcuts:
//!
//! - **ArrowRight**: expand the card (when collapsed).
//! - **ArrowLeft**: collapse the card (when expanded).
//! - **Enter**: expand + focus the first input inside the card.
//! - **Tab / Shift+Tab** (inside a field): cycle through focusable fields
//!   within the card.
//! - **Esc** (inside a field): return focus to the card wrapper.

use dioxus::{
    document,
    hooks::use_signal,
    prelude::{
        component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Key,
        ModifiersInteraction, Props,
    },
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::{
    input_model::{theme::TagIdOrDefaults, thing::ThingId, InputDiagram},
    model_common::Set,
};

use crate::components::editor::{
    common::{
        id_rename_in_input_diagram, parse_tag_id, parse_thing_id, ADD_BTN, DRAG_HANDLE,
        ID_INPUT_CLASS, INPUT_CLASS, REMOVE_BTN, ROW_CLASS, ROW_CLASS_SIMPLE, SECTION_HEADING,
    },
    datalists::list_ids,
};

// === TagNameRow JS helpers === //

/// JavaScript snippet evaluated when the user presses **Tab** inside a row
/// child.
///
/// Moves focus to the next focusable element (input or remove button) within
/// the same row. If there is no next element, focuses the parent row `div`
/// instead.
const JS_TAB_NEXT: &str = "\
    (() => {\
        let el = document.activeElement;\
        if (!el) return;\
        let row = el.closest('[tabindex=\"0\"]');\
        if (!row) return;\
        let items = Array.from(row.querySelectorAll('input, [data-action=\"remove\"]'));\
        let idx = items.indexOf(el);\
        if (idx >= 0 && idx + 1 < items.length) {\
            items[idx + 1].focus();\
        } else {\
            row.focus();\
        }\
    })()";

/// JavaScript snippet evaluated when the user presses **Shift+Tab** inside a
/// row child.
///
/// Moves focus to the previous focusable element (input or remove button)
/// within the same row. If there is no previous element, focuses the parent
/// row `div` instead.
const JS_TAB_PREV: &str = "\
    (() => {\
        let el = document.activeElement;\
        if (!el) return;\
        let row = el.closest('[tabindex=\"0\"]');\
        if (!row) return;\
        let items = Array.from(row.querySelectorAll('input, [data-action=\"remove\"]'));\
        let idx = items.indexOf(el);\
        if (idx > 0) {\
            items[idx - 1].focus();\
        } else {\
            row.focus();\
        }\
    })()";

/// JavaScript snippet: focus the parent row.
const JS_FOCUS_PARENT_ROW: &str = "\
    document.activeElement\
        ?.closest('[tabindex=\"0\"]')\
        ?.focus()";

/// JavaScript snippet: move focus to the previous sibling row.
const JS_FOCUS_PREV_ROW: &str = "\
    document.activeElement\
        ?.previousElementSibling\
        ?.focus()";

/// JavaScript snippet: from the last visible row, walk forwards through the
/// container's following siblings to find a focusable element.
const JS_FOCUS_AFTER_CONTAINER: &str = "\
    (() => {\
        let row = document.activeElement;\
        if (!row) return;\
        let nextRow = row.nextElementSibling;\
        if (nextRow) { nextRow.focus(); return; }\
        let container = row.parentElement;\
        if (!container) return;\
        let next = container.nextElementSibling;\
        while (next) {\
            if (next.tabIndex >= 0 || next.tagName === 'BUTTON' || next.tagName === 'A') {\
                next.focus();\
                return;\
            }\
            next = next.nextElementSibling;\
        }\
    })()";

/// JavaScript snippet: from the first row, walk backwards through the
/// container's preceding siblings to find a focusable element.
const JS_FOCUS_BEFORE_CONTAINER: &str = "\
    (() => {\
        let row = document.activeElement;\
        if (!row) return;\
        let container = row.parentElement;\
        if (!container) return;\
        let prev = container.previousElementSibling;\
        while (prev) {\
            if (prev.tabIndex >= 0 || prev.tagName === 'BUTTON' || prev.tagName === 'A') {\
                prev.focus();\
                return;\
            }\
            prev = prev.previousElementSibling;\
        }\
    })()";

/// JavaScript snippet: focus the first input inside the currently focused
/// element.
const JS_FOCUS_FIRST_INPUT: &str = "\
    document.activeElement\
        ?.querySelector('input')\
        ?.focus()";

// === TagThingsCard JS helpers === //

/// JavaScript snippet: focus the parent `[data-tag-things-card]` ancestor.
const JS_FOCUS_PARENT_CARD: &str = "\
    document.activeElement\
        ?.closest('[data-tag-things-card]')\
        ?.focus()";

/// JavaScript snippet: Tab to the next focusable element (input or
/// `[data-action="remove"]`) within the same `[data-tag-things-card]`.
const JS_CARD_TAB_NEXT: &str = "\
    (() => {\
        let el = document.activeElement;\
        if (!el) return;\
        let card = el.closest('[data-tag-things-card]');\
        if (!card) return;\
        let items = Array.from(card.querySelectorAll(\
            'input, button, [data-action=\"remove\"]'\
        ));\
        let idx = items.indexOf(el);\
        if (idx >= 0 && idx + 1 < items.length) {\
            items[idx + 1].focus();\
        } else {\
            card.focus();\
        }\
    })()";

/// JavaScript snippet: Shift+Tab to the previous focusable element within the
/// same `[data-tag-things-card]`.
const JS_CARD_TAB_PREV: &str = "\
    (() => {\
        let el = document.activeElement;\
        if (!el) return;\
        let card = el.closest('[data-tag-things-card]');\
        if (!card) return;\
        let items = Array.from(card.querySelectorAll(\
            'input, button, [data-action=\"remove\"]'\
        ));\
        let idx = items.indexOf(el);\
        if (idx > 0) {\
            items[idx - 1].focus();\
        } else {\
            card.focus();\
        }\
    })()";

// === TagThingsCard CSS === //

/// CSS classes for the focusable tag-things card wrapper.
///
/// Extends the standard card styling with focus ring and transitions.
const TAG_THINGS_CARD_CLASS: &str = "\
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
const COLLAPSED_HEADER_CLASS: &str = "\
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
const FIELD_INPUT_CLASS: &str = INPUT_CLASS;

/// The **Tags** editor page.
///
/// Provides two sections:
/// 1. Tag Names: map of `TagId` to display label.
/// 2. Tag Things: map of `TagId` to set of `ThingId`s associated with the tag.
#[component]
pub fn TagsPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    // Focus-after-move state for TagNameRow reorder.
    let mut tag_name_focus_idx: Signal<Option<usize>> = use_signal(|| None);

    // Drag-and-drop state for TagNameRow.
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

    // Post-render focus effect for TagNameRow reorder.
    dioxus::hooks::use_effect(move || {
        if let Some(idx) = tag_name_focus_idx() {
            tag_name_focus_idx.set(None);
            document::eval(&format!(
                "setTimeout(() => {{\
                    let container = document.querySelector(\
                        '[data-section-id=\"tag_names\"]'\
                    );\
                    if (container) {{\
                        let row = container.children[{idx}];\
                        if (row) row.focus();\
                    }}\
                }}, 0)"
            ));
        }
    });

    rsx! {
        div {
            class: "flex flex-col gap-2",

            // === Tag Names === //
            h3 { class: SECTION_HEADING, "Tag Names" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Map of TagId to display label."
            }

            div {
                class: "flex flex-col group/key-value-rows",
                "data-section-id": "tag_names",

                for (idx, (tag_id, tag_name)) in tag_entries.iter().enumerate() {
                    {
                        let tag_id = tag_id.clone();
                        let tag_name = tag_name.clone();
                        rsx! {
                            TagNameRow {
                                key: "{tag_id}",
                                input_diagram,
                                tag_id,
                                tag_name,
                                index: idx,
                                entry_count: tag_count,
                                drag_index: tag_name_drag_idx,
                                drop_target: tag_name_drop_target,
                                focus_index: tag_name_focus_idx,
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

// === TagsPage mutation helpers === //

/// Mutation operations for the Tags editor page.
///
/// Grouped here so that related functions are discoverable when sorted by
/// name, per the project's `noun_verb` naming convention.
struct TagsPageOps;

impl TagsPageOps {
    // === Tag name helpers === //

    /// Adds a new tag with a unique placeholder TagId.
    fn tag_add(mut input_diagram: Signal<InputDiagram<'static>>) {
        let mut n = input_diagram.read().tags.len();
        loop {
            let candidate = format!("tag_{n}");
            if let Some(tag_id) = parse_tag_id(&candidate)
                && !input_diagram.read().tags.contains_key(&tag_id)
            {
                input_diagram.write().tags.insert(tag_id, String::new());
                break;
            }
            n += 1;
        }
    }

    /// Removes a tag from the `tags` map.
    fn tag_remove(mut input_diagram: Signal<InputDiagram<'static>>, tag_id_str: &str) {
        if let Some(tag_id) = parse_tag_id(tag_id_str) {
            input_diagram.write().tags.shift_remove(&tag_id);
        }
    }

    /// Renames a tag across all maps in the [`InputDiagram`].
    fn tag_rename(
        mut input_diagram: Signal<InputDiagram<'static>>,
        tag_id_old_str: &str,
        tag_id_new_str: &str,
    ) {
        if tag_id_old_str == tag_id_new_str {
            return;
        }
        let tag_id_old = match parse_tag_id(tag_id_old_str) {
            Some(tag_id) => tag_id,
            None => return,
        };
        let tag_id_new = match parse_tag_id(tag_id_new_str) {
            Some(tag_id) => tag_id,
            None => return,
        };

        let mut input_diagram_ref = input_diagram.write();

        // tags: rename TagId key.
        if let Some(index) = input_diagram_ref.tags.get_index_of(&tag_id_old) {
            let _result = input_diagram_ref
                .tags
                .replace_index(index, tag_id_new.clone());
        }

        // tag_things: rename TagId key.
        if let Some(index) = input_diagram_ref.tag_things.get_index_of(&tag_id_old) {
            let _result = input_diagram_ref
                .tag_things
                .replace_index(index, tag_id_new.clone());
        }

        // theme_tag_things_focus: rename TagIdOrDefaults::Custom key.
        let tag_key_old = TagIdOrDefaults::Custom(tag_id_old.clone());
        if let Some(index) = input_diagram_ref
            .theme_tag_things_focus
            .get_index_of(&tag_key_old)
        {
            let tag_key_new = TagIdOrDefaults::Custom(tag_id_new.clone());
            let _result = input_diagram_ref
                .theme_tag_things_focus
                .replace_index(index, tag_key_new);
        }

        // Shared rename across entity_descs, entity_tooltips, entity_types,
        // and all theme style maps.
        let id_old = tag_id_old.into_inner();
        let id_new = tag_id_new.into_inner();
        id_rename_in_input_diagram(&mut input_diagram_ref, &id_old, &id_new);
    }

    /// Updates the display name for an existing tag.
    fn tag_name_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
        tag_id_str: &str,
        name: &str,
    ) {
        if let Some(tag_id) = parse_tag_id(tag_id_str)
            && let Some(entry) = input_diagram.write().tags.get_mut(&tag_id)
        {
            *entry = name.to_owned();
        }
    }

    /// Moves a tag entry from one index to another in the `tags` map.
    fn tag_move(mut input_diagram: Signal<InputDiagram<'static>>, from: usize, to: usize) {
        input_diagram.write().tags.move_index(from, to);
    }

    // === Tag things helpers === //

    /// Adds a new tag->things entry, picking an unmapped tag or generating a
    /// placeholder.
    fn tag_things_entry_add(mut input_diagram: Signal<InputDiagram<'static>>) {
        let input_diagram_read = input_diagram.read();
        let tag_id = input_diagram_read
            .tags
            .keys()
            .find(|tag_id| !input_diagram_read.tag_things.contains_key(*tag_id))
            .cloned();

        match tag_id {
            Some(tag_id) => {
                drop(input_diagram_read);
                input_diagram.write().tag_things.insert(tag_id, Set::new());
            }
            None => {
                let mut n = input_diagram_read.tag_things.len();
                loop {
                    let candidate = format!("tag_{n}");
                    if let Some(tag_id) = parse_tag_id(&candidate)
                        && !input_diagram_read.tag_things.contains_key(&tag_id)
                    {
                        drop(input_diagram_read);
                        input_diagram.write().tag_things.insert(tag_id, Set::new());
                        break;
                    }
                    n += 1;
                }
            }
        }
    }

    /// Removes a tag->things entry.
    fn tag_things_entry_remove(mut input_diagram: Signal<InputDiagram<'static>>, tag_id_str: &str) {
        if let Some(tag_id) = parse_tag_id(tag_id_str) {
            input_diagram.write().tag_things.shift_remove(&tag_id);
        }
    }

    /// Renames the key of a tag->things entry.
    fn tag_things_entry_rename(
        mut input_diagram: Signal<InputDiagram<'static>>,
        tag_id_old_str: &str,
        tag_id_new_str: &str,
        current_things: &[String],
    ) {
        if tag_id_old_str == tag_id_new_str {
            return;
        }
        let tag_id_old = match parse_tag_id(tag_id_old_str) {
            Some(tag_id) => tag_id,
            None => return,
        };
        let tag_id_new = match parse_tag_id(tag_id_new_str) {
            Some(tag_id) => tag_id,
            None => return,
        };
        let things: Set<ThingId<'static>> = current_things
            .iter()
            .filter_map(|s| parse_thing_id(s))
            .collect();
        let mut input_diagram = input_diagram.write();
        input_diagram.tag_things.insert(tag_id_new, things);
        input_diagram.tag_things.swap_remove(&tag_id_old);
    }

    /// Updates a single thing within a tag's thing set at the given index.
    fn tag_things_thing_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
        tag_id_str: &str,
        idx: usize,
        thing_id_new_str: &str,
    ) {
        let tag_id = match parse_tag_id(tag_id_str) {
            Some(tag_id) => tag_id,
            None => return,
        };
        let thing_id_new = match parse_thing_id(thing_id_new_str) {
            Some(thing_id) => thing_id,
            None => return,
        };

        let mut input_diagram = input_diagram.write();
        if let Some(things) = input_diagram.tag_things.get_mut(&tag_id) {
            // `Set` (IndexSet) does not support indexed mutation directly.
            // Rebuild the set with the replacement at the given position.
            let mut things_new = Set::with_capacity(things.len());
            for (i, existing) in things.iter().enumerate() {
                if i == idx {
                    things_new.insert(thing_id_new.clone());
                } else {
                    things_new.insert(existing.clone());
                }
            }
            *things = things_new;
        }
    }

    /// Removes a thing from a tag's thing set by index.
    fn tag_things_thing_remove(
        mut input_diagram: Signal<InputDiagram<'static>>,
        tag_id_str: &str,
        idx: usize,
    ) {
        let tag_id = match parse_tag_id(tag_id_str) {
            Some(tag_id) => tag_id,
            None => return,
        };

        let mut input_diagram = input_diagram.write();
        if let Some(things) = input_diagram.tag_things.get_mut(&tag_id)
            && idx < things.len()
        {
            things.shift_remove_index(idx);
        }
    }

    /// Adds a thing to a tag's thing set.
    fn tag_things_thing_add(mut input_diagram: Signal<InputDiagram<'static>>, tag_id_str: &str) {
        let tag_id = match parse_tag_id(tag_id_str) {
            Some(tag_id) => tag_id,
            None => return,
        };

        // Pick the first thing ID as a placeholder.
        let placeholder = {
            let input_diagram = input_diagram.read();
            input_diagram
                .things
                .keys()
                .next()
                .map(|thing_id| thing_id.as_str().to_owned())
                .unwrap_or_else(|| "thing_0".to_owned())
        };
        let thing_id_new = match parse_thing_id(&placeholder) {
            Some(thing_id) => thing_id,
            None => return,
        };

        let mut input_diagram = input_diagram.write();
        if let Some(things) = input_diagram.tag_things.get_mut(&tag_id) {
            things.insert(thing_id_new);
        }
    }
}

// === Helper functions === //

/// Returns Tailwind border-color classes for the drop-target indicator on
/// tag name rows.
fn tag_drag_row_border_class(
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
    index: usize,
) -> &'static str {
    let drag_src = *drag_index.read();
    let is_target = drop_target.read().is_some_and(|i| i == index);

    if is_target
        && let Some(from) = drag_src
        && from != index
    {
        if from < index {
            return "border-t-transparent border-b-blue-400";
        } else {
            return "border-t-blue-400 border-b-transparent";
        }
    }
    "border-t-transparent border-b-transparent"
}

// === Helper components === //

/// A single editable row for a tag name (TagId -> display label).
///
/// Supports drag-and-drop reorder, Alt+Up/Alt+Down keyboard reorder, Enter
/// to edit, Tab/Shift+Tab to cycle within the row, and Esc to return focus
/// to the row.
#[component]
fn TagNameRow(
    input_diagram: Signal<InputDiagram<'static>>,
    tag_id: String,
    tag_name: String,
    index: usize,
    entry_count: usize,
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
    mut focus_index: Signal<Option<usize>>,
) -> Element {
    let border_class = tag_drag_row_border_class(drag_index, drop_target, index);

    let can_move_up = index > 0;
    let can_move_down = index + 1 < entry_count;

    let is_first = index == 0;

    rsx! {
        div {
            class: "{ROW_CLASS} {border_class} rounded focus:border-blue-400 focus:bg-gray-800 focus:outline-none",
            tabindex: "0",
            draggable: "true",

            // === Keyboard shortcuts (row-level) === //
            onkeydown: move |evt| {
                let alt = evt.modifiers().alt();

                match evt.key() {
                    Key::ArrowUp if alt => {
                        evt.prevent_default();
                        if can_move_up {
                            TagsPageOps::tag_move(input_diagram, index, index - 1);
                            focus_index.set(Some(index - 1));
                        }
                    }
                    Key::ArrowDown if alt => {
                        evt.prevent_default();
                        if can_move_down {
                            TagsPageOps::tag_move(input_diagram, index, index + 1);
                            focus_index.set(Some(index + 1));
                        }
                    }
                    Key::ArrowUp => {
                        evt.prevent_default();
                        if is_first {
                            document::eval(JS_FOCUS_BEFORE_CONTAINER);
                        } else {
                            document::eval(JS_FOCUS_PREV_ROW);
                        }
                    }
                    Key::ArrowDown => {
                        evt.prevent_default();
                        document::eval(JS_FOCUS_AFTER_CONTAINER);
                    }
                    Key::Enter => {
                        evt.prevent_default();
                        document::eval(JS_FOCUS_FIRST_INPUT);
                    }
                    _ => {}
                }
            },

            // === Drag-and-drop === //
            ondragstart: move |_| {
                drag_index.set(Some(index));
            },
            ondragover: move |evt| {
                evt.prevent_default();
                drop_target.set(Some(index));
            },
            ondrop: move |evt| {
                evt.prevent_default();
                if let Some(from) = *drag_index.read()
                    && from != index {
                        TagsPageOps::tag_move(input_diagram, from, index);
                    }
                drag_index.set(None);
                drop_target.set(None);
            },
            ondragend: move |_| {
                drag_index.set(None);
                drop_target.set(None);
            },

            // Drag handle
            span {
                class: DRAG_HANDLE,
                title: "Drag to reorder",
                "\u{2817}"
            }

            // TagId input
            input {
                class: ID_INPUT_CLASS,
                style: "max-width:14rem",
                tabindex: "-1",
                list: list_ids::TAG_IDS,
                placeholder: "tag_id",
                pattern: "^[a-zA-Z_][a-zA-Z0-9_]*$",
                value: "{tag_id}",
                onchange: {
                    let tag_id_old = tag_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let tag_id_new = evt.value();
                        TagsPageOps::tag_rename(input_diagram, &tag_id_old, &tag_id_new);
                    }
                },
                onkeydown: move |evt| {
                    tag_name_row_field_keydown(evt);
                },
            }

            // Display name input
            input {
                class: INPUT_CLASS,
                tabindex: "-1",
                placeholder: "Display name",
                value: "{tag_name}",
                oninput: {
                    let tag_id = tag_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let name = evt.value();
                        TagsPageOps::tag_name_update(input_diagram, &tag_id, &name);
                    }
                },
                onkeydown: move |evt| {
                    tag_name_row_field_keydown(evt);
                },
            }

            // Remove button
            button {
                class: REMOVE_BTN,
                tabindex: "-1",
                "data-action": "remove",
                onclick: {
                    let tag_id = tag_id.clone();
                    move |_| {
                        TagsPageOps::tag_remove(input_diagram, &tag_id);
                    }
                },
                onkeydown: move |evt| {
                    tag_name_row_field_keydown(evt);
                },
                "\u{2715}"
            }
        }
    }
}

/// Shared `onkeydown` handler for inputs and remove buttons inside a
/// `TagNameRow`.
///
/// - **Esc**: return focus to the parent row.
/// - **Tab / Shift+Tab**: cycle through focusable fields within the row.
/// - **ArrowUp / ArrowDown**: stop propagation so the row-level handler does
///   not fire (allows cursor movement in text inputs).
fn tag_name_row_field_keydown(evt: dioxus::events::KeyboardEvent) {
    let shift = evt.modifiers().shift();
    match evt.key() {
        Key::Escape => {
            evt.prevent_default();
            evt.stop_propagation();
            document::eval(JS_FOCUS_PARENT_ROW);
        }
        Key::Tab => {
            evt.prevent_default();
            evt.stop_propagation();
            if shift {
                document::eval(JS_TAB_PREV);
            } else {
                document::eval(JS_TAB_NEXT);
            }
        }
        // Stop arrow keys from bubbling to the row handler
        // so that the cursor can move inside the input.
        Key::ArrowUp | Key::ArrowDown => {
            evt.stop_propagation();
        }
        _ => {}
    }
}

/// A card for one tag's associated things.
///
/// Supports keyboard shortcuts:
///
/// - **ArrowRight**: expand the card.
/// - **ArrowLeft**: collapse the card.
/// - **Enter**: expand + focus the first input inside the card.
/// - **Tab / Shift+Tab** (inside a field): cycle through focusable fields.
/// - **Esc** (inside a field): return focus to the card wrapper.
///
/// When collapsed, shows the tag ID and number of things.
#[component]
fn TagThingsCard(
    input_diagram: Signal<InputDiagram<'static>>,
    tag_id: String,
    things: Vec<String>,
) -> Element {
    let mut collapsed = use_signal(|| true);

    let thing_count = things.len();
    let thing_suffix = if thing_count != 1 { "s" } else { "" };

    rsx! {
        div {
            class: TAG_THINGS_CARD_CLASS,
            tabindex: "0",
            "data-tag-things-card": "true",

            // === Card-level keyboard shortcuts === //
            onkeydown: move |evt| {
                match evt.key() {
                    Key::ArrowRight => {
                        evt.prevent_default();
                        collapsed.set(false);
                    }
                    Key::ArrowLeft => {
                        evt.prevent_default();
                        collapsed.set(true);
                    }
                    Key::Enter => {
                        evt.prevent_default();
                        collapsed.set(false);
                        document::eval(
                            "setTimeout(() => {\
                                document.activeElement\
                                    ?.querySelector('input')\
                                    ?.focus();\
                            }, 0)"
                        );
                    }
                    _ => {}
                }
            },

            if *collapsed.read() {
                // === Collapsed summary === //
                div {
                    class: COLLAPSED_HEADER_CLASS,
                    onclick: move |_| collapsed.set(false),

                    // Expand chevron
                    span {
                        class: "text-gray-500 text-xs",
                        ">"
                    }

                    span {
                        class: "text-sm font-mono text-blue-400",
                        "{tag_id}"
                    }

                    span {
                        class: "text-xs text-gray-500",
                        "({thing_count} thing{thing_suffix})"
                    }
                }
            } else {
                // === Expanded content === //

                // Collapse toggle
                div {
                    class: "flex flex-row items-center gap-1 cursor-pointer select-none mb-1",
                    onclick: move |_| collapsed.set(true),

                    span {
                        class: "text-gray-500 text-xs rotate-90 inline-block",
                        ">"
                    }
                    span {
                        class: "text-xs text-gray-500",
                        "Collapse"
                    }
                }

                // === Header: TagId + Remove === //
                div {
                    class: ROW_CLASS_SIMPLE,

                    label {
                        class: "text-xs text-gray-500 w-12",
                        "Tag"
                    }

                    input {
                        class: FIELD_INPUT_CLASS,
                        style: "max-width:14rem",
                        tabindex: "-1",
                        list: list_ids::TAG_IDS,
                        placeholder: "tag_id",
                        value: "{tag_id}",
                        onchange: {
                            let tag_id_old = tag_id.clone();
                            let current_things = things.clone();
                            move |evt: dioxus::events::FormEvent| {
                                TagsPageOps::tag_things_entry_rename(
                                    input_diagram,
                                    &tag_id_old,
                                    &evt.value(),
                                    &current_things,
                                );
                            }
                        },
                        onkeydown: move |evt| {
                            tag_things_card_field_keydown(evt);
                        },
                    }

                    button {
                        class: REMOVE_BTN,
                        tabindex: "-1",
                        "data-action": "remove",
                        onclick: {
                            let tag_id = tag_id.clone();
                            move |_| {
                                TagsPageOps::tag_things_entry_remove(input_diagram, &tag_id);
                            }
                        },
                        onkeydown: move |evt| {
                            tag_things_card_field_keydown(evt);
                        },
                        "x Remove"
                    }
                }

                // === Thing list === //
                div {
                    class: "flex flex-col gap-1 pl-4",

                    for (idx, thing_id) in things.iter().enumerate() {
                        {
                            let thing_id = thing_id.clone();
                            let tag_id = tag_id.clone();
                            rsx! {
                                div {
                                    key: "{tag_id}_{idx}",
                                    class: ROW_CLASS_SIMPLE,

                                    span {
                                        class: "text-xs text-gray-500 w-6 text-right",
                                        "{idx}."
                                    }

                                    input {
                                        class: FIELD_INPUT_CLASS,
                                        style: "max-width:14rem",
                                        tabindex: "-1",
                                        list: list_ids::THING_IDS,
                                        placeholder: "thing_id",
                                        value: "{thing_id}",
                                        onchange: {
                                            let tag_id = tag_id.clone();
                                            move |evt: dioxus::events::FormEvent| {
                                                TagsPageOps::tag_things_thing_update(
                                                    input_diagram,
                                                    &tag_id,
                                                    idx,
                                                    &evt.value(),
                                                );
                                            }
                                        },
                                        onkeydown: move |evt| {
                                            tag_things_card_field_keydown(evt);
                                        },
                                    }

                                    button {
                                        class: REMOVE_BTN,
                                        tabindex: "-1",
                                        "data-action": "remove",
                                        onclick: {
                                            let tag_id = tag_id.clone();
                                            move |_| {
                                                TagsPageOps::tag_things_thing_remove(
                                                    input_diagram,
                                                    &tag_id,
                                                    idx,
                                                );
                                            }
                                        },
                                        onkeydown: move |evt| {
                                            tag_things_card_field_keydown(evt);
                                        },
                                        "\u{2715}"
                                    }
                                }
                            }
                        }
                    }

                    button {
                        class: ADD_BTN,
                        tabindex: -1,
                        onclick: {
                            let tag_id = tag_id.clone();
                            move |_| {
                                TagsPageOps::tag_things_thing_add(input_diagram, &tag_id);
                            }
                        },
                        onkeydown: move |evt| {
                            tag_things_card_field_keydown(evt);
                        },
                        "+ Add thing"
                    }
                }
            }
        }
    }
}

/// Shared `onkeydown` handler for inputs and remove buttons inside a
/// `TagThingsCard`.
///
/// - **Esc**: return focus to the parent `TagThingsCard`.
/// - **Tab / Shift+Tab**: cycle through focusable fields within the card.
/// - **ArrowUp / ArrowDown / ArrowLeft / ArrowRight**: stop propagation so the
///   card-level handler does not fire (allows cursor movement in text inputs).
fn tag_things_card_field_keydown(evt: dioxus::events::KeyboardEvent) {
    let shift = evt.modifiers().shift();
    match evt.key() {
        Key::Escape => {
            evt.prevent_default();
            evt.stop_propagation();
            document::eval(JS_FOCUS_PARENT_CARD);
        }
        Key::Tab => {
            evt.prevent_default();
            evt.stop_propagation();
            if shift {
                document::eval(JS_CARD_TAB_PREV);
            } else {
                document::eval(JS_CARD_TAB_NEXT);
            }
        }
        Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight => {
            evt.stop_propagation();
        }
        _ => {}
    }
}
