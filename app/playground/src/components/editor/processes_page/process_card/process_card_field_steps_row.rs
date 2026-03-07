//! A single step row within the steps list of a [`ProcessCard`].
//!
//! Extracted from [`ProcessCard`] to keep the parent component concise.
//!
//! Keyboard shortcuts (on the inputs):
//!
//! - **Alt+Up / Alt+Down**: move the step up or down in the list.
//! - All other keys fall through to the standard field navigation
//!   (`field_keydown` with the card-level data attribute).
//!
//! The row also supports drag-and-drop reordering via a [`DragHandle`]
//! grip indicator, with drop-target border highlighting provided by
//! [`drag_border_class`].
//!
//! [`ProcessCard`]: super::ProcessCard

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::ProcessCardOps;

use crate::components::editor::{
    common::{CardComponent, FieldNav, REMOVE_BTN, ROW_CLASS},
    datalists::list_ids,
    processes_page::{DATA_ATTR, FIELD_INPUT_CLASS},
    reorderable::{drag_border_class, DragHandle},
};

/// A single step row within the steps section of a process card.
///
/// Displays a drag handle, row index, a step ID input, a step label input,
/// and a remove button for one entry in the process's step list. Supports
/// Alt+Up/Down keyboard reordering and drag-and-drop reordering.
#[component]
pub(crate) fn ProcessCardFieldStepsRow(
    input_diagram: Signal<InputDiagram<'static>>,
    process_id: String,
    step_id: String,
    step_label: String,
    index: usize,
    step_count: usize,
    mut step_focus_idx: Signal<Option<usize>>,
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
) -> Element {
    let can_move_up = index > 0;
    let can_move_down = index + 1 < step_count;
    let border_class = drag_border_class(drag_index, drop_target, index);

    rsx! {
        div {
            class: "{ROW_CLASS} {border_class}",
            draggable: "true",
            "data-process-step-row": "",

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
                onkeydown: {
                    let process_id = process_id.clone();
                    let process_id_down = process_id.clone();
                    CardComponent::field_onkeydown(
                        DATA_ATTR,
                        can_move_up,
                        can_move_down,
                        move || {
                            ProcessCardOps::step_move(
                                &mut input_diagram.write(),
                                &process_id,
                                index,
                                index - 1,
                            );
                            step_focus_idx.set(Some(index - 1));
                        },
                        move || {
                            ProcessCardOps::step_move(
                                &mut input_diagram.write(),
                                &process_id_down,
                                index,
                                index + 1,
                            );
                            step_focus_idx.set(Some(index + 1));
                        },
                    )
                },
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
                onkeydown: {
                    let process_id = process_id.clone();
                    let process_id_down = process_id.clone();
                    CardComponent::field_onkeydown(
                        DATA_ATTR,
                        can_move_up,
                        can_move_down,
                        move || {
                            ProcessCardOps::step_move(
                                &mut input_diagram.write(),
                                &process_id,
                                index,
                                index - 1,
                            );
                            step_focus_idx.set(Some(index - 1));
                        },
                        move || {
                            ProcessCardOps::step_move(
                                &mut input_diagram.write(),
                                &process_id_down,
                                index,
                                index + 1,
                            );
                            step_focus_idx.set(Some(index + 1));
                        },
                    )
                },
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
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "x"
            }
        }
    }
}
