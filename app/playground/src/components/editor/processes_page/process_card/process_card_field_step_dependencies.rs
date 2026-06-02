//! Step dependencies list field for a [`ProcessCard`].
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
    processes_page::{step_dependency_card::StepDependencyCard, DATA_ATTR},
};

/// Step dependencies list with per-step cards and an "add" button.
///
/// Displays each step's dependency set as a [`StepDependencyCard`] and provides
/// an "+ Add step dependency mapping" button at the bottom to append a new
/// entry.
#[component]
pub(crate) fn ProcessCardFieldStepDependencies(
    input_diagram: Signal<InputDiagram<'static>>,
    process_id: String,
    step_dependencies: Vec<(String, Vec<String>)>,
) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-1 pl-4",

            h4 {
                class: "text-xs font-semibold text-gray-400 mt-1",
                "Step -> Step Dependencies"
            }

            for (step_id, dep_ids) in step_dependencies.iter() {
                {
                    let step_id = step_id.clone();
                    let dep_ids = dep_ids.clone();
                    let process_id = process_id.clone();
                    rsx! {
                        StepDependencyCard {
                            key: "{process_id}_sd_{step_id}",
                            input_diagram,
                            process_id,
                            step_id,
                            dep_ids,
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
                        ProcessCardOps::step_dependency_add(&mut input_diagram.write(), &process_id);
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "+ Add step dependency mapping"
            }
        }
    }
}
