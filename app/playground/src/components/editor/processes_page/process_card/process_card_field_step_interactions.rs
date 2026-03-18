//! Step thing-interactions list field for a [`ProcessCard`].
//!
//! Extracted from [`ProcessCard`] to keep the parent component concise.
//!
//! [`ProcessCard`]: super::ProcessCard

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::ProcessCardOps;

use crate::components::editor::{
    common::{FieldNav, ADD_BTN},
    processes_page::{step_interaction_card::StepInteractionCard, DATA_ATTR},
};

/// Step thing-interactions list with per-step cards and an "add" button.
///
/// Displays each step-interaction mapping as a [`StepInteractionCard`] and
/// provides an "+ Add step interaction mapping" button at the bottom to
/// append a new entry.
#[component]
pub(crate) fn ProcessCardFieldStepInteractions(
    input_diagram: Signal<InputDiagram<'static>>,
    process_id: String,
    step_interactions: Vec<(String, Vec<String>)>,
) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-1 pl-4",

            h4 {
                class: "text-xs font-semibold text-gray-400 mt-1",
                "Step -> Thing Interactions"
            }

            for (step_id, edge_ids) in step_interactions.iter() {
                {
                    let step_id = step_id.clone();
                    let edge_ids = edge_ids.clone();
                    let process_id = process_id.clone();
                    rsx! {
                        StepInteractionCard {
                            key: "{process_id}_sti_{step_id}",
                            input_diagram,
                            process_id,
                            step_id,
                            edge_ids,
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
                        ProcessCardOps::step_interaction_add(&mut input_diagram.write(), &process_id);
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "+ Add step interaction mapping"
            }
        }
    }
}
