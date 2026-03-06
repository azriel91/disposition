//! Steps list field for a [`ProcessCard`].
//!
//! Extracted from [`ProcessCard`] to keep the parent component concise.
//!
//! [`ProcessCard`]: super::ProcessCard

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::Signal,
};
use disposition::input_model::InputDiagram;

use crate::components::editor::{
    common::{FieldNav, ADD_BTN},
    processes_page::{
        process_card::ProcessCardFieldStepsRow, process_card_ops::ProcessCardOps, DATA_ATTR,
    },
};

/// Steps list with per-row editing and an "add" button.
///
/// Displays each step as a [`ProcessCardFieldStepsRow`] (step ID + label +
/// remove) and provides an "+ Add step" button at the bottom to append a
/// new entry.
#[component]
pub(crate) fn ProcessCardFieldSteps(
    input_diagram: Signal<InputDiagram<'static>>,
    process_id: String,
    steps: Vec<(String, String)>,
) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-1 pl-4",

            h4 {
                class: "text-xs font-semibold text-gray-400 mt-1",
                "Steps"
            }

            for (step_id, step_label) in steps.iter() {
                {
                    let step_id = step_id.clone();
                    let step_label = step_label.clone();
                    let process_id = process_id.clone();
                    rsx! {
                        ProcessCardFieldStepsRow {
                            key: "{process_id}_{step_id}",
                            input_diagram,
                            process_id,
                            step_id,
                            step_label,
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
                        ProcessCardOps::step_add(input_diagram, &process_id);
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "+ Add step"
            }
        }
    }
}
