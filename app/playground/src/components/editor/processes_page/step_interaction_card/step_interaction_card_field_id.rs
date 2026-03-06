//! Step ID field with remove button.
//!
//! Extracted from [`StepInteractionCard`] to keep the parent component concise.
//!
//! [`StepInteractionCard`]: super::StepInteractionCard

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::Signal,
};
use disposition::input_model::InputDiagram;

use crate::components::editor::{
    common::{FieldNav, REMOVE_BTN, ROW_CLASS_SIMPLE},
    datalists::list_ids,
    processes_page::{
        step_interaction_card_ops::StepInteractionCardOps, DATA_ATTR, FIELD_INPUT_CLASS,
    },
};

/// Step ID input and remove button for a step-interaction card.
///
/// Displays the step ID input (with datalist) and a remove button.
/// Handles step rename and removal.
#[component]
pub(crate) fn StepInteractionCardFieldId(
    input_diagram: Signal<InputDiagram<'static>>,
    process_id: String,
    step_id: String,
    edge_ids: Vec<String>,
) -> Element {
    rsx! {
        div {
            class: ROW_CLASS_SIMPLE,

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
                    let edge_ids = edge_ids.clone();
                    move |evt: dioxus::events::FormEvent| {
                        StepInteractionCardOps::step_interaction_rename(
                            input_diagram,
                            &process_id,
                            &step_id_old,
                            &evt.value(),
                            &edge_ids,
                        );
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
            }

            button {
                class: REMOVE_BTN,
                tabindex: "-1",
                "data-action": "remove",
                onclick: {
                    let process_id = process_id.clone();
                    let step_id = step_id.clone();
                    move |_| {
                        StepInteractionCardOps::step_interaction_remove(input_diagram, &process_id, &step_id);
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "\u{2715}"
            }
        }
    }
}
