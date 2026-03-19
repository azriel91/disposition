//! Undo / redo toolbar buttons for the playground editor.
//!
//! Renders a pair of small buttons (undo / redo) that operate on the
//! [`UndoHistory`] context signal. The buttons are disabled when no
//! undo / redo steps are available.
//!
//! Keyboard shortcuts (Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y) are handled by
//! a separate global `onkeydown` handler installed on the editor root
//! element (see [`DispositionEditor`](super::super::DispositionEditor)).

use dioxus::{
    document,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;

use crate::undo_history::{history_redo, history_undo, UndoHistory};

use super::focus_restore::{JS_FOCUS_RESTORE, JS_FOCUS_SAVE};

/// CSS classes for the undo/redo buttons when enabled.
const BTN_ENABLED: &str = "\
    rounded \
    w-8 h-8 \
    flex \
    items-center \
    justify-center \
    font-semibold \
    cursor-pointer \
    select-none \
    bg-gray-700 \
    hover:bg-gray-600 \
    text-gray-200 \
    border \
    border-gray-600 \
    focus:outline-none \
    focus:border-blue-400\
";

/// CSS classes for the undo/redo buttons when disabled.
const BTN_DISABLED: &str = "\
    rounded \
    w-8 h-8 \
    flex \
    items-center \
    justify-center \
    font-semibold \
    select-none \
    bg-gray-800 \
    text-gray-600 \
    border \
    border-gray-700 \
    cursor-default\
";

/// Toolbar with undo and redo buttons.
///
/// Reads the [`UndoHistory`] signal to determine button enabled state and
/// applies undo/redo by writing back to the `input_diagram` signal.
///
/// # Props
///
/// * `input_diagram`: the writable signal for the current diagram.
/// * `undo_history`: the signal holding the [`UndoHistory`].
#[component]
pub fn UndoRedoToolbar(
    input_diagram: Signal<InputDiagram<'static>>,
    undo_history: Signal<UndoHistory>,
) -> Element {
    let history = undo_history.read();
    let can_undo = history.can_undo();
    let can_redo = history.can_redo();
    let undo_depth = history.undo_depth();
    let redo_depth = history.redo_depth();
    drop(history);

    let undo_title = if can_undo {
        format!(
            "Undo ({undo_depth} step{s} available) -- Ctrl+Z",
            s = if undo_depth == 1 { "" } else { "s" }
        )
    } else {
        String::from("Nothing to undo")
    };

    let redo_title = if can_redo {
        format!(
            "Redo ({redo_depth} step{s} available) -- Ctrl+Shift+Z / Ctrl+Y",
            s = if redo_depth == 1 { "" } else { "s" }
        )
    } else {
        String::from("Nothing to redo")
    };

    rsx! {
        div {
            class: "flex flex-row gap-1 items-center",
            role: "toolbar",
            "aria-label": "Undo / Redo",

            button {
                class: if can_undo { BTN_ENABLED } else { BTN_DISABLED },
                disabled: !can_undo,
                title: "{undo_title}",
                "aria-label": "Undo",
                onclick: move |_| {
                    if let Some(diagram) = history_undo(undo_history) {
                        document::eval(JS_FOCUS_SAVE);
                        input_diagram.set(diagram);
                        document::eval(JS_FOCUS_RESTORE);
                    }
                },
                // Left-pointing arrow
                svg {
                    width: "24",
                    height: "24",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    path {
                        d: "M6.5 11a6 6 0 1 1 2.5 5.0"
                    },
                    path {
                        d: "M5.5 7 v5 h5",
                        transform: "rotate(-20 6 11)",
                    }
                }
            }

            button {
                class: if can_redo { BTN_ENABLED } else { BTN_DISABLED },
                disabled: !can_redo,
                title: "{redo_title}",
                "aria-label": "Redo",
                onclick: move |_| {
                    if let Some(diagram) = history_redo(undo_history) {
                        document::eval(JS_FOCUS_SAVE);
                        input_diagram.set(diagram);
                        document::eval(JS_FOCUS_RESTORE);
                    }
                },
                // right-pointing arrow
                svg {
                    width: "24",
                    height: "24",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    path {
                        d: "M17.5 11a6 6 0 1 0 -2.5 5.0"
                    },
                    path {
                        d: "M18.5 7 v5 h-5",
                        transform: "rotate(20 18 11)",
                    }
                }
            }
        }
    }
}
