//! A single edge group row within the edges list of a [`StepInteractionCard`].
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
    common::{FieldNav, REMOVE_BTN, ROW_CLASS_SIMPLE},
    datalists::list_ids,
    processes_page::{DATA_ATTR, FIELD_INPUT_CLASS},
};

/// A single edge group row within the edges section of a step-interaction card.
///
/// Displays an index label, an edge group ID input (with datalist), and a
/// remove button for one entry in the step's edge group list.
#[component]
pub(crate) fn StepInteractionCardFieldEdgesRow(
    input_diagram: Signal<InputDiagram<'static>>,
    process_id: String,
    step_id: String,
    edge_group_id: String,
    index: usize,
) -> Element {
    rsx! {
        div {
            class: ROW_CLASS_SIMPLE,

            span {
                class: "text-xs text-gray-500 w-6 text-right",
                "{index}."
            }

            input {
                class: FIELD_INPUT_CLASS,
                style: "max-width:14rem",
                tabindex: "-1",
                list: list_ids::EDGE_GROUP_IDS,
                placeholder: "edge_group_id",
                value: "{edge_group_id}",
                onchange: {
                    let process_id = process_id.clone();
                    let step_id = step_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        StepInteractionCardOps::step_interaction_edge_update(
                            &mut input_diagram.write(),
                            &process_id,
                            &step_id,
                            index,
                            &evt.value(),
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
                        StepInteractionCardOps::step_interaction_edge_remove(
                            &mut input_diagram.write(),
                            &process_id,
                            &step_id,
                            index,
                        );
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "\u{2715}"
            }
        }
    }
}
