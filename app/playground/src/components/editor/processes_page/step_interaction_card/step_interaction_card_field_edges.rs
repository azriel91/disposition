//! Edge group IDs list field for a [`StepInteractionCard`].
//!
//! Extracted from [`StepInteractionCard`] to keep the parent component concise.
//!
//! [`StepInteractionCard`]: super::StepInteractionCard

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::StepInteractionCardOps;

use crate::components::editor::{
    common::{FieldNav, ADD_BTN},
    processes_page::{step_interaction_card::StepInteractionCardFieldEdgesRow, DATA_ATTR},
};

/// Edge group IDs list with per-row editing and an "add" button.
///
/// Displays each edge group ID as a [`StepInteractionCardFieldEdgesRow`] and
/// provides an "+ Add edge group" button at the bottom to append a new entry.
#[component]
pub(crate) fn StepInteractionCardFieldEdges(
    input_diagram: Signal<InputDiagram<'static>>,
    process_id: String,
    step_id: String,
    edge_ids: Vec<String>,
) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-1 pl-4",

            for (idx, edge_group_id) in edge_ids.iter().enumerate() {
                {
                    let edge_group_id = edge_group_id.clone();
                    let process_id = process_id.clone();
                    let step_id = step_id.clone();
                    rsx! {
                        StepInteractionCardFieldEdgesRow {
                            key: "{process_id}_{step_id}_{idx}",
                            input_diagram,
                            process_id,
                            step_id,
                            edge_group_id,
                            index: idx,
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
