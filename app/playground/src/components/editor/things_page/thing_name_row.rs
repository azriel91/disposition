//! Thing name row component.
//!
//! A single editable row for a thing name (`ThingId` -> display label).
//! Supports keyboard shortcuts:
//!
//! - **Up / Down** (on row): move focus to the previous / next row. When on the
//!   first or last visible row, moves focus to the adjacent `CollapseBar` (if
//!   any).
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

use dioxus::{
    document,
    prelude::{
        component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Key,
        ModifiersInteraction, Props,
    },
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;

use crate::components::editor::{
    common::{ID_INPUT_CLASS, INPUT_CLASS, REMOVE_BTN, ROW_CLASS},
    datalists::list_ids,
};

use super::{
    drag_handle::DragHandle, drag_row_border_class::drag_row_border_class,
    things_page_ops::ThingsPageOps,
};

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

/// JavaScript snippet: from the first row, walk backwards through the
/// container's preceding siblings to find a focusable element (e.g. a
/// `CollapseBar` button).
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

/// JavaScript snippet: from the last visible row, walk forwards through the
/// container's following siblings to find a focusable element (e.g. a
/// `CollapseBar` button). First checks for a next sibling row (handles
/// collapsed sections where `entry_count` exceeds the number of rendered
/// rows).
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

/// JavaScript snippet: focus the first input inside the currently focused
/// element.
const JS_FOCUS_FIRST_INPUT: &str = "\
    document.activeElement\
        ?.querySelector('input')\
        ?.focus()";

/// A single editable row for a thing name (`ThingId` -> display label).
#[component]
pub fn ThingNameRow(
    input_diagram: Signal<InputDiagram<'static>>,
    thing_id: String,
    thing_name: String,
    index: usize,
    entry_count: usize,
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
    mut focus_index: Signal<Option<usize>>,
) -> Element {
    let border_class = drag_row_border_class(drag_index, drop_target, index);

    let can_move_up = index > 0;
    let can_move_down = index + 1 < entry_count;

    let is_first = index == 0;

    rsx! {
        div {
            class: "{ROW_CLASS} {border_class} rounded focus:border-blue-400 focus:bg-gray-800 focus:outline-none",
            tabindex: "0",
            draggable: "true",

            // === Keyboard shortcuts (row-level) === //
            // Arrow keys from child `<input>`s and the remove button do not
            // reach here because each child's `onkeydown` calls
            // `stop_propagation` for them.
            onkeydown: move |evt| {
                let alt = evt.modifiers().alt();

                match evt.key() {
                    Key::ArrowUp if alt => {
                        evt.prevent_default();
                        if can_move_up {
                            ThingsPageOps::thing_move(input_diagram, index, index - 1);
                            focus_index.set(Some(index - 1));
                        }
                    }
                    Key::ArrowDown if alt => {
                        evt.prevent_default();
                        if can_move_down {
                            ThingsPageOps::thing_move(input_diagram, index, index + 1);
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
                        // Use a DOM-based check: if there is no next
                        // sibling row, walk to the container's next
                        // focusable sibling (e.g. CollapseBar). This
                        // handles collapsed sections where `entry_count`
                        // exceeds the number of rendered rows.
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
                        ThingsPageOps::thing_move(input_diagram, from, index);
                    }
                drag_index.set(None);
                drop_target.set(None);
            },
            ondragend: move |_| {
                drag_index.set(None);
                drop_target.set(None);
            },

            DragHandle {}

            // ThingId input
            input {
                class: ID_INPUT_CLASS,
                style: "max-width:14rem",
                tabindex: "-1",
                list: list_ids::THING_IDS,
                placeholder: "thing_id",
                value: "{thing_id}",
                pattern: "^[a-zA-Z_][a-zA-Z0-9_]*$",
                onchange: {
                    let thing_id_old = thing_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let thing_id_new = evt.value();
                        ThingsPageOps::thing_rename(input_diagram, &thing_id_old, &thing_id_new);
                    }
                },
                onkeydown: move |evt| {
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
                },
            }

            // Display name input
            input {
                class: INPUT_CLASS,
                tabindex: "-1",
                placeholder: "Display name",
                value: "{thing_name}",
                oninput: {
                    let thing_id = thing_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let name = evt.value();
                        ThingsPageOps::thing_name_update(input_diagram, &thing_id, &name);
                    }
                },
                onkeydown: move |evt| {
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
                        Key::ArrowUp | Key::ArrowDown => {
                            evt.stop_propagation();
                        }
                        _ => {}
                    }
                },
            }

            // Remove button
            span {
                class: REMOVE_BTN,
                tabindex: "-1",
                "data-action": "remove",
                onclick: {
                    let thing_id = thing_id.clone();
                    move |_| {
                        ThingsPageOps::thing_remove(input_diagram, &thing_id);
                    }
                },
                onkeydown: move |evt| {
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
                        // Stop arrow keys from bubbling to the row handler.
                        Key::ArrowUp | Key::ArrowDown => {
                            evt.stop_propagation();
                        }
                        _ => {}
                    }
                },
                "✕"
            }
        }
    }
}
