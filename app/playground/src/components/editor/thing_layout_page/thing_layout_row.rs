//! Thing layout row component.
//!
//! A single row in the thing hierarchy layout editor. Each row displays an
//! indented thing ID with controls for reordering (drag-and-drop and keyboard
//! shortcuts) and nesting (indent / outdent).

use dioxus::{
    document,
    prelude::{
        component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Key,
        ModifiersInteraction, Props,
    },
    signals::{ReadableExt, Signal, WritableExt},
};

use disposition::input_model::{thing::ThingId, InputDiagram};

use crate::components::editor::common::{DRAG_HANDLE, REMOVE_BTN};

use super::{
    drag_row_border_class::drag_row_border_class, thing_layout_ops::ThingLayoutOps,
    thing_layout_page_ops::ThingLayoutPageOps,
};

/// CSS classes for the layout row.
///
/// Compared to the generic `ROW_CLASS`, this variant adds
/// `focus:border-blue-400 focus:bg-gray-800 focus:outline-none` so that the
/// currently-focused row is visually highlighted with a blue border and a
/// slightly lighter background.
const LAYOUT_ROW_CLASS: &str = "\
    flex flex-row gap-2 items-center \
    pt-1 \
    pb-1 \
    border-t-1 \
    border-b-1 \
    rounded \
    focus:border-blue-400 \
    focus:bg-gray-800 \
    focus:outline-none \
    has-[:active]:opacity-40\
";

/// CSS classes for the indent/outdent and move buttons.
const ACTION_BTN: &str = "\
    text-gray-500 \
    hover:text-gray-200 \
    text-xs \
    cursor-pointer \
    px-0.5 \
    select-none \
    leading-none\
";

/// CSS classes for a disabled (greyed-out) action button.
const ACTION_BTN_DISABLED: &str = "\
    text-gray-700 \
    text-xs \
    px-0.5 \
    select-none \
    leading-none \
    cursor-default\
";

/// A single row in the thing layout hierarchy editor.
///
/// # Props
///
/// * `input_diagram`: the shared diagram signal.
/// * `thing_id`: the `ThingId` for this entry, e.g. `ThingId` for `"t_aws"`.
/// * `depth`: nesting depth (`0` = top-level).
/// * `flat_index`: the index of this entry in the flattened list.
/// * `flat_len`: total number of entries in the flattened list.
/// * `is_first_sibling`: whether this is the first sibling at its depth.
/// * `is_last_sibling`: whether this is the last sibling at its depth.
/// * `drag_index`: signal tracking which row is being dragged.
/// * `drop_target`: signal tracking which row is the current drop target.
/// * `focus_index`: signal that, when set to `Some(idx)`, causes the row at
///   that flat index to receive focus after the next DOM update.
#[component]
pub fn ThingLayoutRow(
    input_diagram: Signal<InputDiagram<'static>>,
    thing_id: ThingId<'static>,
    depth: usize,
    flat_index: usize,
    flat_len: usize,
    is_first_sibling: bool,
    is_last_sibling: bool,
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
    mut focus_index: Signal<Option<usize>>,
) -> Element {
    let border_class = drag_row_border_class(drag_index, drop_target, flat_index);
    let indent_px = depth * 24;

    // Determine which actions are available.
    let can_move_up = flat_index > 0;
    let can_move_down = flat_index + 1 < flat_len;
    // Indent requires a previous sibling to become a child of.
    let can_indent = !is_first_sibling;
    let can_outdent = depth > 0;

    let up_btn_class = if can_move_up {
        ACTION_BTN
    } else {
        ACTION_BTN_DISABLED
    };
    let down_btn_class = if can_move_down {
        ACTION_BTN
    } else {
        ACTION_BTN_DISABLED
    };
    let indent_btn_class = if can_indent {
        ACTION_BTN
    } else {
        ACTION_BTN_DISABLED
    };
    let outdent_btn_class = if can_outdent {
        ACTION_BTN
    } else {
        ACTION_BTN_DISABLED
    };

    let thing_id_display = thing_id.to_string();

    rsx! {
        div {
            class: "{LAYOUT_ROW_CLASS} {border_class}",
            tabindex: "0",
            draggable: "true",
            style: "padding-left: {indent_px}px;",

            // === Keyboard shortcuts === //
            onkeydown: move |evt| {
                let alt = evt.modifiers().alt();
                let shift = evt.modifiers().shift();

                match evt.key() {
                    Key::ArrowUp if alt => {
                        evt.prevent_default();
                        if let Some(new_idx) = ThingLayoutOps::entry_move_up(input_diagram, flat_index) {
                            focus_index.set(Some(new_idx));
                        }
                    }
                    Key::ArrowDown if alt => {
                        evt.prevent_default();
                        if let Some(new_idx) = ThingLayoutOps::entry_move_down(input_diagram, flat_index) {
                            focus_index.set(Some(new_idx));
                        }
                    }
                    Key::ArrowUp => {
                        evt.prevent_default();
                        // Focus the previous sibling row.
                        document::eval(
                            "document.activeElement\
                                ?.previousElementSibling\
                                ?.focus()",
                        );
                    }
                    Key::ArrowDown => {
                        evt.prevent_default();
                        // Focus the next sibling row.
                        document::eval(
                            "document.activeElement\
                                ?.nextElementSibling\
                                ?.focus()",
                        );
                    }
                    Key::Tab if shift => {
                        evt.prevent_default();
                        if let Some(new_idx) = ThingLayoutOps::entry_outdent(input_diagram, flat_index) {
                            focus_index.set(Some(new_idx));
                        }
                    }
                    Key::Tab => {
                        evt.prevent_default();
                        if let Some(new_idx) = ThingLayoutOps::entry_indent(input_diagram, flat_index) {
                            focus_index.set(Some(new_idx));
                        }
                    }
                    _ => {}
                }
            },

            // === Drag-and-drop === //
            ondragstart: move |_| {
                drag_index.set(Some(flat_index));
            },
            ondragover: move |evt| {
                evt.prevent_default();
                drop_target.set(Some(flat_index));
            },
            ondrop: move |evt| {
                evt.prevent_default();
                if let Some(from) = *drag_index.read()
                    && from != flat_index
                {
                    ThingLayoutOps::entry_drag_move(input_diagram, from, flat_index);
                }
                drag_index.set(None);
                drop_target.set(None);
            },
            ondragend: move |_| {
                drag_index.set(None);
                drop_target.set(None);
            },

            // === Drag handle === //
            span {
                class: DRAG_HANDLE,
                title: "Drag to reorder",
                "⠿"
            }

            // === Move buttons === //
            span {
                class: "{up_btn_class}",
                title: "Move up (Alt+Up)",
                onclick: move |_| {
                    if can_move_up
                        && let Some(new_idx) = ThingLayoutOps::entry_move_up(input_diagram, flat_index) {
                            focus_index.set(Some(new_idx));
                        }
                },
                "▲"
            }
            span {
                class: "{down_btn_class}",
                title: "Move down (Alt+Down)",
                onclick: move |_| {
                    if can_move_down
                        && let Some(new_idx) = ThingLayoutOps::entry_move_down(input_diagram, flat_index) {
                            focus_index.set(Some(new_idx));
                        }
                },
                "▼"
            }

            // === Indent / Outdent buttons === //
            span {
                class: "{outdent_btn_class}",
                title: "Outdent (Shift+Tab)",
                onclick: move |_| {
                    if can_outdent
                        && let Some(new_idx) = ThingLayoutOps::entry_outdent(input_diagram, flat_index) {
                            focus_index.set(Some(new_idx));
                        }
                },
                "⇤"
            }
            span {
                class: "{indent_btn_class}",
                title: "Indent (Tab)",
                onclick: move |_| {
                    if can_indent
                        && let Some(new_idx) = ThingLayoutOps::entry_indent(input_diagram, flat_index) {
                            focus_index.set(Some(new_idx));
                        }
                },
                "⇥"
            }

            // === Thing ID label === //
            span {
                class: "\
                    flex-1 \
                    text-sm \
                    font-mono \
                    text-gray-200 \
                    select-none \
                    truncate\
                ",
                title: "{thing_id_display}",
                "{thing_id_display}"
            }

            // === Remove button === //
            span {
                class: REMOVE_BTN,
                title: "Remove from hierarchy",
                onclick: {
                    let thing_id_str = thing_id_display.clone();
                    move |_| {
                        ThingLayoutPageOps::entry_remove(input_diagram, &thing_id_str);
                    }
                },
                "✕"
            }
        }
    }
}
