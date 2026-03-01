//! Shared editable row component for ID-value maps.
//!
//! Provides [`IdValueRow`] -- a reusable row with:
//! - A drag handle for reordering.
//! - An ID input (with optional datalist and pattern).
//! - A value input.
//! - A remove button.
//!
//! Keyboard shortcuts:
//!
//! - **Up / Down** (on row): move focus to the previous / next row.
//! - **Alt+Up / Alt+Down**: move the entry up or down in the list.
//! - **Enter** (on row): focus the first input inside the row for editing.
//! - **Tab** (inside an input or remove button): move focus to the next
//!   interactive element within the same row (inputs then remove button), or
//!   back to the parent row when there are no more elements.
//! - **Esc** (inside an input or remove button): return focus to the parent
//!   row.
//!
//! Arrow keys are **not** intercepted when an `<input>` has focus, so the
//! cursor can still be moved within the text field.
//!
//! After an ID rename the row element is destroyed and recreated under the new
//! key. The row signals its stable parent container via `rename_refocus` so
//! that the container can re-focus the correct sub-element (ID input on Enter,
//! next field on Tab, parent row on Shift+Tab or Esc) once the new DOM node
//! exists.

mod drag_handle;
mod drag_row_border_class;

pub use self::{drag_handle::DragHandle, drag_row_border_class::drag_row_border_class};

use dioxus::{
    document,
    hooks::use_signal,
    prelude::{
        component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Callback, Element, Key,
        ModifiersInteraction, Props,
    },
    signals::{ReadableExt, Signal, WritableExt},
};

use crate::components::editor::common::{
    RenameRefocus, RenameRefocusTarget, ID_INPUT_CLASS, INPUT_CLASS, REMOVE_BTN, ROW_CLASS,
};

// === JS focus helpers === //

/// Moves focus to the next focusable element (input or remove button) within
/// the same row. If there is no next element, focuses the parent row `div`
/// instead.
pub const JS_TAB_NEXT: &str = "\
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

/// Moves focus to the previous focusable element (input or remove button)
/// within the same row. If there is no previous element, focuses the parent
/// row `div` instead.
pub const JS_TAB_PREV: &str = "\
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

/// Focus the parent row.
pub const JS_FOCUS_PARENT_ROW: &str = "\
    document.activeElement\
        ?.closest('[tabindex=\"0\"]')\
        ?.focus()";

/// Focus the active editor sub-tab so the user can navigate away via arrow
/// keys. Falls back to blurring the active element.
pub const JS_FOCUS_PARENT_SECTION: &str = "\
    (() => {\
        let tab = document.querySelector('[role=\"tab\"][aria-selected=\"true\"]');\
        if (tab) { tab.focus(); return; }\
        document.activeElement?.blur();\
    })()";

/// Move focus to the previous sibling row.
pub const JS_FOCUS_PREV_ROW: &str = "\
    document.activeElement\
        ?.previousElementSibling\
        ?.focus()";

/// From the current row, walk forwards through the container's following
/// siblings to find a focusable element. First checks for a next sibling
/// row, then walks to the container's next siblings.
pub const JS_FOCUS_AFTER_CONTAINER: &str = "\
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

/// Focus the first input inside the currently focused element.
pub const JS_FOCUS_FIRST_INPUT: &str = "\
    document.activeElement\
        ?.querySelector('input')\
        ?.focus()";

// === Shared field keydown handler === //

/// Shared `onkeydown` handler for inputs and remove buttons inside a row.
///
/// - **Esc**: return focus to the parent row.
/// - **Tab / Shift+Tab**: cycle through focusable fields within the row.
/// - **Enter**: stop propagation so the row-level handler does not fire (only
///   relevant for the remove button -- the browser still fires the native
///   click).
/// - **ArrowUp / ArrowDown**: stop propagation so the row-level handler does
///   not fire (allows cursor movement in text inputs).
pub fn field_keydown(evt: dioxus::events::KeyboardEvent) {
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
        Key::Enter => {
            evt.stop_propagation();
        }
        Key::ArrowUp | Key::ArrowDown => {
            evt.stop_propagation();
        }
        _ => {}
    }
}

// === IdValueRow component === //

/// A reusable editable row for ID-value maps.
///
/// The row renders a drag handle, an ID input, a value input, and a remove
/// button with unified keyboard and drag-and-drop behaviour. Callers
/// supply callbacks for the four mutation operations that differ between
/// pages.
///
/// # Callbacks
///
/// * `on_move(from, to)`: reorder the entry from index `from` to index `to`.
/// * `on_rename(id_old, id_new)`: change the entry key.
/// * `on_update(id, value)`: change the entry value.
/// * `on_remove(id)`: delete the entry.
///
/// # Props
///
/// * `entry_id`: the current ID string, e.g. `"thing_0"`.
/// * `entry_value`: the current value string.
/// * `id_list`: datalist id for the ID input (e.g. `list_ids::THING_IDS`).
/// * `id_placeholder`: placeholder text for the ID input, e.g. `"thing_id"`.
/// * `value_placeholder`: placeholder text for the value input, e.g. `"Display
///   name"`.
/// * `index`: position of this entry in its list.
/// * `entry_count`: total number of entries.
/// * `drag_index` / `drop_target`: shared drag-and-drop signals.
/// * `focus_index`: shared focus-after-move signal.
/// * `rename_refocus`: when an ID rename completes, this signal is set so that
///   the stable parent container can re-focus the correct field inside the
///   newly created row element.
#[component]
pub fn IdValueRow(
    entry_id: String,
    entry_value: String,
    id_list: String,
    id_placeholder: String,
    value_placeholder: String,
    index: usize,
    entry_count: usize,
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
    mut focus_index: Signal<Option<usize>>,
    mut rename_refocus: Signal<Option<RenameRefocus>>,
    on_move: Callback<(usize, usize)>,
    on_rename: Callback<(String, String)>,
    on_update: Callback<(String, String)>,
    on_remove: Callback<String>,
) -> Element {
    let border_class = drag_row_border_class(drag_index, drop_target, index);

    let can_move_up = index > 0;
    let can_move_down = index + 1 < entry_count;

    let is_first = index == 0;

    // Tracks which refocus target the next ID rename should use.
    // - `IdInput`: Enter or blur triggered the rename.
    // - `NextField`: forward Tab triggered the rename.
    // - `FocusParent`: Shift+Tab or Esc triggered the rename.
    let mut rename_target = use_signal(|| RenameRefocusTarget::IdInput);

    rsx! {
        div {
            class: "{ROW_CLASS} {border_class} rounded focus:border-blue-400 focus:bg-gray-800 focus:outline-none",
            tabindex: "0",
            draggable: "true",
            "data-entry-id": "{entry_id}",

            // === Keyboard shortcuts (row-level) === //
            onkeydown: move |evt| {
                let alt = evt.modifiers().alt();

                match evt.key() {
                    Key::ArrowUp if alt => {
                        evt.prevent_default();
                        evt.stop_propagation();
                        if can_move_up {
                            on_move.call((index, index - 1));
                            focus_index.set(Some(index - 1));
                        }
                    }
                    Key::ArrowDown if alt => {
                        evt.prevent_default();
                        evt.stop_propagation();
                        if can_move_down {
                            on_move.call((index, index + 1));
                            focus_index.set(Some(index + 1));
                        }
                    }
                    Key::ArrowUp => {
                        evt.prevent_default();
                        evt.stop_propagation();
                        if !is_first {
                            document::eval(JS_FOCUS_PREV_ROW);
                        }
                    }
                    Key::ArrowDown => {
                        evt.prevent_default();
                        evt.stop_propagation();
                        document::eval(JS_FOCUS_AFTER_CONTAINER);
                    }
                    Key::Escape => {
                        evt.prevent_default();
                        evt.stop_propagation();
                        document::eval(JS_FOCUS_PARENT_SECTION);
                    }
                    Key::Enter => {
                        evt.prevent_default();
                        evt.stop_propagation();
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
                    && from != index
                {
                    on_move.call((from, index));
                }
                drag_index.set(None);
                drop_target.set(None);
            },
            ondragend: move |_| {
                drag_index.set(None);
                drop_target.set(None);
            },

            DragHandle {}

            // === ID input === //
            input {
                class: ID_INPUT_CLASS,
                style: "max-width:14rem",
                tabindex: "-1",
                list: "{id_list}",
                placeholder: "{id_placeholder}",
                value: "{entry_id}",
                pattern: "^[a-zA-Z_][a-zA-Z0-9_]*$",
                onchange: {
                    let id_old = entry_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let id_new = evt.value();
                        let target = *rename_target.read();
                        on_rename.call((id_old.clone(), id_new.clone()));
                        rename_refocus.set(Some(RenameRefocus {
                            new_id: id_new,
                            target,
                        }));
                    }
                },
                onkeydown: move |evt| {
                    // Record which refocus target the upcoming onchange should
                    // use before field_keydown handles the event (which
                    // prevents default and may move focus).
                    match evt.key() {
                        Key::Tab if evt.modifiers().shift() => {
                            rename_target.set(RenameRefocusTarget::FocusParent);
                        }
                        Key::Tab => {
                            rename_target.set(RenameRefocusTarget::NextField);
                        }
                        Key::Escape => {
                            rename_target.set(RenameRefocusTarget::FocusParent);
                        }
                        Key::Enter => {
                            rename_target.set(RenameRefocusTarget::IdInput);
                        }
                        _ => {}
                    }
                    field_keydown(evt);
                },
            }

            // === Value input === //
            input {
                class: INPUT_CLASS,
                tabindex: "-1",
                placeholder: "{value_placeholder}",
                value: "{entry_value}",
                oninput: {
                    let entry_id = entry_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let new_value = evt.value();
                        on_update.call((entry_id.clone(), new_value));
                    }
                },
                onkeydown: move |evt| {
                    field_keydown(evt);
                },
            }

            // === Remove button === //
            button {
                class: REMOVE_BTN,
                tabindex: "-1",
                "data-action": "remove",
                onclick: {
                    let entry_id = entry_id.clone();
                    move |_| {
                        on_remove.call(entry_id.clone());
                    }
                },
                onkeydown: move |evt| {
                    field_keydown(evt);
                },
                "\u{2715}"
            }
        }
    }
}
