//! Steps list field with reorderable rows and an add button.
//!
//! Extracted from [`ProcessCard`] to keep the parent component concise.
//!
//! [`ProcessCard`]: super::ProcessCard

use dioxus::{
    hooks::use_signal,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::ProcessCardOps;

use crate::components::editor::{
    common::{FieldNav, ADD_BTN},
    processes_page::{process_card::ProcessCardFieldStepsRow, DATA_ATTR},
    reorderable::ReorderableContainer,
};

/// Steps list with per-row editing, Alt+Up/Down reordering, and an "add"
/// button.
///
/// Displays each step as a [`ProcessCardFieldStepsRow`] (step ID + label +
/// remove) inside a [`ReorderableContainer`] and provides an "+ Add step"
/// button at the bottom to append a new entry.
#[component]
pub(crate) fn ProcessCardFieldSteps(
    input_diagram: Signal<InputDiagram<'static>>,
    process_id: String,
    steps: Vec<(String, String)>,
) -> Element {
    let step_focus_idx: Signal<Option<usize>> = use_signal(|| None);
    let step_drag_idx: Signal<Option<usize>> = use_signal(|| None);
    let step_drop_target: Signal<Option<usize>> = use_signal(|| None);
    let step_count = steps.len();

    rsx! {
        div {
            class: "flex flex-col gap-1 pl-4",

            h4 {
                class: "text-xs font-semibold text-gray-400 mt-1",
                "Steps"
            }

            ReorderableContainer {
                data_attr: "data-process-step-row".to_owned(),
                section_id: format!("process_steps_{process_id}"),
                focus_index: step_focus_idx,
                focus_inner_selector: Some("input".to_owned()),

                for (idx, (step_id, step_label)) in steps.iter().enumerate() {
                    {
                        let step_id = step_id.clone();
                        let step_label = step_label.clone();
                        let process_id = process_id.clone();
                        let process_id_move = process_id.clone();
                        let process_id_add = process_id.clone();
                        let process_id_remove = process_id.clone();
                        let step_id_remove = step_id.clone();
                        rsx! {
                            ProcessCardFieldStepsRow {
                                key: "{process_id}_{idx}",
                                input_diagram,
                                process_id,
                                step_id,
                                step_label,
                                index: idx,
                                step_count,
                                step_focus_idx,
                                drag_index: step_drag_idx,
                                drop_target: step_drop_target,
                                on_move: move |(from, to): (usize, usize)| {
                                    ProcessCardOps::step_move(
                                        &mut input_diagram.write(),
                                        &process_id_move,
                                        from,
                                        to,
                                    );
                                },
                                on_add: move |insert_at: usize| {
                                    ProcessCardOps::step_add(
                                        &mut input_diagram.write(),
                                        &process_id_add,
                                    );
                                    let last = step_count;
                                    ProcessCardOps::step_move(
                                        &mut input_diagram.write(),
                                        &process_id_add,
                                        last,
                                        insert_at,
                                    );
                                },
                                on_remove: move |_row_index: usize| {
                                    ProcessCardOps::step_remove(
                                        &mut input_diagram.write(),
                                        &process_id_remove,
                                        &step_id_remove,
                                    );
                                },
                            }
                        }
                    }
                }
            }

            button {
                class: ADD_BTN,
                tabindex: -1,
                onclick: {
                    let process_id = process_id.clone();
                    move |_| {
                        ProcessCardOps::step_add(&mut input_diagram.write(), &process_id);
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "+ Add step"
            }
        }
    }
}
