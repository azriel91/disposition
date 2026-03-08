//! A single step row within the steps list of a [`ProcessCard`].
//!
//! Extracted from [`ProcessCard`] to keep the parent component concise.
//!
//! Keyboard shortcuts (on the row wrapper):
//!
//! - **Up / Down**: move focus to the previous / next sibling row.
//! - **Ctrl+Up / Ctrl+Down**: jump to the first / last row.
//! - **Alt+Up / Alt+Down**: move the step up or down in the list.
//! - **Alt+Shift+Up / Alt+Shift+Down**: insert a new step above / below the
//!   current row.
//! - **Ctrl+Shift+K**: remove the step.
//! - **Enter**: focus the first input inside the row for editing.
//! - **Escape**: focus the parent card wrapper.
//!
//! Keyboard shortcuts (on inputs inside the row):
//!
//! - **Alt+Up / Alt+Down**: move the step up or down in the list.
//! - **Alt+Shift+Up / Alt+Shift+Down**: insert a new step above / below.
//! - **Ctrl+Shift+K**: remove the step.
//! - **Tab / Shift+Tab**: cycle through focusable fields within the row.
//! - **Escape**: return focus to the row wrapper.
//!
//! The row also supports drag-and-drop reordering via a [`DragHandle`]
//! grip indicator, with drop-target border highlighting provided by
//! [`drag_border_class`].
//!
//! [`ProcessCard`]: super::ProcessCard

use dioxus::{
    prelude::{
        component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Callback, Element, Props,
    },
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::ProcessCardOps;

use crate::components::editor::{
    common::{RowComponent, REMOVE_BTN, ROW_CLASS},
    datalists::list_ids,
    processes_page::{DATA_ATTR, FIELD_INPUT_CLASS},
    reorderable::{drag_border_class, DragHandle},
};

/// Data attribute placed on each process step row wrapper.
///
/// Used by [`ReorderableContainer`] and keyboard navigation helpers
/// to locate sibling rows within the steps list.
///
/// [`ReorderableContainer`]: crate::components::editor::reorderable::ReorderableContainer
const ROW_DATA_ATTR: &str = "data-process-step-row";

/// A single step row within the steps section of a process card.
///
/// Displays a drag handle, row index, a step ID input, a step label input,
/// and a remove button for one entry in the process's step list. Supports
/// full keyboard navigation (Up/Down focus cycling, Alt reorder,
/// Alt+Shift insert, Ctrl+Shift+K remove, Enter to edit, Escape to card)
/// and drag-and-drop reordering.
#[component]
pub(crate) fn ProcessCardFieldStepsRow(
    input_diagram: Signal<InputDiagram<'static>>,
    process_id: String,
    step_id: String,
    step_label: String,
    index: usize,
    step_count: usize,
    step_focus_idx: Signal<Option<usize>>,
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
    on_move: Callback<(usize, usize)>,
    on_add: Callback<usize>,
    on_remove: Callback<usize>,
) -> Element {
    let border_class = drag_border_class(drag_index, drop_target, index);

    rsx! {
        div {
            class: "{ROW_CLASS} {border_class} rounded focus:border-blue-400 focus:bg-gray-800 focus:outline-none",
            tabindex: "0",
            draggable: "true",
            "data-process-step-row": "",
            "data-input-diagram-field": "{process_id}_step_{index}",

            // === Row-level keyboard shortcuts === //
            onkeydown: RowComponent::row_onkeydown(
                ROW_DATA_ATTR,
                DATA_ATTR,
                index,
                step_count,
                step_focus_idx,
                on_move,
                on_add,
                on_remove,
            ),

            // === Drag-and-drop === //
            ondragstart: move |_| {
                drag_index.set(Some(index));
            },
            ondragover: move |evt| {
                evt.prevent_default();
                drop_target.set(Some(index));
            },
            ondrop: {
                let process_id = process_id.clone();
                move |evt| {
                    evt.prevent_default();
                    if let Some(from) = *drag_index.read()
                        && from != index
                    {
                        ProcessCardOps::step_move(&mut input_diagram.write(), &process_id, from, index);
                    }
                    drag_index.set(None);
                    drop_target.set(None);
                }
            },
            ondragend: move |_| {
                drag_index.set(None);
                drop_target.set(None);
            },

            DragHandle {}

            span {
                class: "text-xs text-gray-500 w-6 text-right",
                "{index}."
            }

            input {
                class: FIELD_INPUT_CLASS,
                style: "max-width:14rem",
                tabindex: "-1",
                list: list_ids::PROCESS_STEP_IDS,
                placeholder: "step_id",
                value: "{step_id}",
                onchange: {
                    let process_id = process_id.clone();
                    let step_id_old = step_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        ProcessCardOps::step_rename(&mut input_diagram.write(), &process_id, &step_id_old, &evt.value());
                    }
                },
                onkeydown: RowComponent::row_field_onkeydown(
                    ROW_DATA_ATTR,
                    index,
                    step_count,
                    step_focus_idx,
                    on_move,
                    on_add,
                    on_remove,
                ),
            }

            input {
                class: FIELD_INPUT_CLASS,
                tabindex: "-1",
                placeholder: "Step label",
                value: "{step_label}",
                oninput: {
                    let process_id = process_id.clone();
                    let step_id = step_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        ProcessCardOps::step_label_update(&mut input_diagram.write(), &process_id, &step_id, &evt.value());
                    }
                },
                onkeydown: RowComponent::row_field_onkeydown(
                    ROW_DATA_ATTR,
                    index,
                    step_count,
                    step_focus_idx,
                    on_move,
                    on_add,
                    on_remove,
                ),
            }

            button {
                class: REMOVE_BTN,
                tabindex: "-1",
                "data-action": "remove",
                onclick: {
                    let process_id = process_id.clone();
                    let step_id = step_id.clone();
                    move |_| {
                        ProcessCardOps::step_remove(&mut input_diagram.write(), &process_id, &step_id);
                    }
                },
                onkeydown: RowComponent::row_field_onkeydown(
                    ROW_DATA_ATTR,
                    index,
                    step_count,
                    step_focus_idx,
                    on_move,
                    on_add,
                    on_remove,
                ),
                "x"
            }
        }
    }
}
