//! A single step row within the steps list of a [`ProcessCard`].
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
    common::{FieldNav, REMOVE_BTN, ROW_CLASS_SIMPLE},
    datalists::list_ids,
    processes_page::{process_card_ops::ProcessCardOps, DATA_ATTR, FIELD_INPUT_CLASS},
};

/// A single step row within the steps section of a process card.
///
/// Displays a step ID input, a step label input, and a remove button for
/// one entry in the process's step list.
#[component]
pub(crate) fn ProcessCardFieldStepsRow(
    input_diagram: Signal<InputDiagram<'static>>,
    process_id: String,
    step_id: String,
    step_label: String,
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
                    move |evt: dioxus::events::FormEvent| {
                        ProcessCardOps::step_rename(input_diagram, &process_id, &step_id_old, &evt.value());
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
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
                        ProcessCardOps::step_label_update(input_diagram, &process_id, &step_id, &evt.value());
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
                        ProcessCardOps::step_remove(input_diagram, &process_id, &step_id);
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "\u{2715}"
            }
        }
    }
}
