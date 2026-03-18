//! Edge group IDs list field for a [`StepInteractionCard`].
//!
//! Extracted from [`StepInteractionCard`] to keep the parent component concise.
//!
//! [`StepInteractionCard`]: super::StepInteractionCard

use dioxus::{
    hooks::use_signal,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::StepInteractionCardOps;

use crate::components::editor::{
    common::{FieldNav, ADD_BTN},
    processes_page::{step_interaction_card::StepInteractionCardFieldEdgesRow, DATA_ATTR},
    reorderable::ReorderableContainer,
};

/// Edge group IDs list with per-row editing, reordering, and an "add" button.
///
/// Displays each edge group ID as a [`StepInteractionCardFieldEdgesRow`]
/// inside a [`ReorderableContainer`] and provides an "+ Add edge group"
/// button at the bottom to append a new entry.
#[component]
pub(crate) fn StepInteractionCardFieldEdges(
    input_diagram: Signal<InputDiagram<'static>>,
    process_id: String,
    step_id: String,
    edge_ids: Vec<String>,
) -> Element {
    let edge_count = edge_ids.len();
    let edge_focus_idx: Signal<Option<usize>> = use_signal(|| None);
    let edge_drag_idx: Signal<Option<usize>> = use_signal(|| None);
    let edge_drop_target: Signal<Option<usize>> = use_signal(|| None);

    rsx! {
        div {
            class: "flex flex-col gap-1 pl-4",

            ReorderableContainer {
                data_attr: "data-step-edge-row".to_owned(),
                section_id: format!("step_edges_{process_id}_{step_id}"),
                focus_index: edge_focus_idx,
                focus_inner_selector: Some("input".to_owned()),

                for (idx, edge_group_id) in edge_ids.iter().enumerate() {
                    {
                        let edge_group_id = edge_group_id.clone();
                        let process_id = process_id.clone();
                        let step_id = step_id.clone();
                        let process_id_move = process_id.clone();
                        let step_id_move = step_id.clone();
                        let process_id_add = process_id.clone();
                        let step_id_add = step_id.clone();
                        let process_id_remove = process_id.clone();
                        let step_id_remove = step_id.clone();
                        rsx! {
                            StepInteractionCardFieldEdgesRow {
                                key: "{process_id}_{step_id}_{idx}",
                                input_diagram,
                                process_id,
                                step_id,
                                edge_group_id,
                                index: idx,
                                edge_count,
                                edge_focus_idx,
                                drag_index: edge_drag_idx,
                                drop_target: edge_drop_target,
                                on_move: move |(from, to): (usize, usize)| {
                                    StepInteractionCardOps::step_interaction_edge_move(
                                        &mut input_diagram.write(),
                                        &process_id_move,
                                        &step_id_move,
                                        from,
                                        to,
                                    );
                                },
                                on_add: move |insert_at: usize| {
                                    StepInteractionCardOps::step_interaction_edge_add(
                                        &mut input_diagram.write(),
                                        &process_id_add,
                                        &step_id_add,
                                    );
                                    let last = edge_count;
                                    StepInteractionCardOps::step_interaction_edge_move(
                                        &mut input_diagram.write(),
                                        &process_id_add,
                                        &step_id_add,
                                        last,
                                        insert_at,
                                    );
                                },
                                on_remove: move |row_index: usize| {
                                    StepInteractionCardOps::step_interaction_edge_remove(
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
                        StepInteractionCardOps::step_interaction_edge_add(
                            &mut input_diagram.write(),
                            &process_id,
                            &step_id,
                        );
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "+ Add edge group"
            }
        }
    }
}
