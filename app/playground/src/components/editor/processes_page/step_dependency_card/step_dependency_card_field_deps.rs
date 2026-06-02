//! Dependency step IDs list field for a [`StepDependencyCard`].
//!
//! Extracted from [`StepDependencyCard`] to keep the parent component concise.
//!
//! [`StepDependencyCard`]: super::StepDependencyCard

use dioxus::{
    hooks::use_signal,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::StepDependencyCardOps;

use crate::components::editor::{
    common::{FieldNav, ADD_BTN},
    processes_page::{step_dependency_card::StepDependencyCardFieldDepsRow, DATA_ATTR},
    reorderable::ReorderableContainer,
};

/// Dependency step IDs list with per-row editing, reordering, and an "add"
/// button.
///
/// Displays each dependency step ID as a [`StepDependencyCardFieldDepsRow`]
/// inside a [`ReorderableContainer`] and provides an "+ Add dependency" button
/// at the bottom to append a new entry.
#[component]
pub(crate) fn StepDependencyCardFieldDeps(
    input_diagram: Signal<InputDiagram<'static>>,
    process_id: String,
    step_id: String,
    dep_ids: Vec<String>,
) -> Element {
    let dep_count = dep_ids.len();
    let dep_focus_idx: Signal<Option<usize>> = use_signal(|| None);
    let dep_drag_idx: Signal<Option<usize>> = use_signal(|| None);
    let dep_drop_target: Signal<Option<usize>> = use_signal(|| None);

    rsx! {
        div {
            class: "flex flex-col gap-1 pl-4",

            ReorderableContainer {
                data_attr: "data-step-dep-row".to_owned(),
                section_id: format!("step_deps_{process_id}_{step_id}"),
                focus_index: dep_focus_idx,
                focus_inner_selector: Some("input".to_owned()),

                for (idx, dep_id) in dep_ids.iter().enumerate() {
                    {
                        let dep_id = dep_id.clone();
                        let process_id = process_id.clone();
                        let step_id = step_id.clone();
                        let process_id_move = process_id.clone();
                        let step_id_move = step_id.clone();
                        let process_id_add = process_id.clone();
                        let step_id_add = step_id.clone();
                        let process_id_remove = process_id.clone();
                        let step_id_remove = step_id.clone();
                        rsx! {
                            StepDependencyCardFieldDepsRow {
                                key: "{process_id}_{step_id}_{idx}",
                                input_diagram,
                                process_id,
                                step_id,
                                dep_id,
                                index: idx,
                                dep_count,
                                dep_focus_idx,
                                drag_index: dep_drag_idx,
                                drop_target: dep_drop_target,
                                on_move: move |(from, to): (usize, usize)| {
                                    StepDependencyCardOps::step_dependency_dep_move(
                                        &mut input_diagram.write(),
                                        &process_id_move,
                                        &step_id_move,
                                        from,
                                        to,
                                    );
                                },
                                on_add: move |insert_at: usize| {
                                    StepDependencyCardOps::step_dependency_dep_add(
                                        &mut input_diagram.write(),
                                        &process_id_add,
                                        &step_id_add,
                                    );
                                    let last = dep_count;
                                    StepDependencyCardOps::step_dependency_dep_move(
                                        &mut input_diagram.write(),
                                        &process_id_add,
                                        &step_id_add,
                                        last,
                                        insert_at,
                                    );
                                },
                                on_remove: move |row_index: usize| {
                                    StepDependencyCardOps::step_dependency_dep_remove(
                                        &mut input_diagram.write(),
                                        &process_id_remove,
                                        &step_id_remove,
                                        row_index,
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
                    let step_id = step_id.clone();
                    move |_| {
                        StepDependencyCardOps::step_dependency_dep_add(
                            &mut input_diagram.write(),
                            &process_id,
                            &step_id,
                        );
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "+ Add dependency"
            }
        }
    }
}
