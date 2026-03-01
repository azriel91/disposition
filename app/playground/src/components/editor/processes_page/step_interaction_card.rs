//! Card component for a single step's thing-interaction list.
//!
//! Displays the step ID, a remove button, and a list of edge group IDs
//! that can be individually edited, removed, or added to.

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::Signal,
};
use disposition::input_model::InputDiagram;

use crate::components::editor::{
    common::{ADD_BTN, INNER_CARD_CLASS, REMOVE_BTN, ROW_CLASS_SIMPLE},
    datalists::list_ids,
};

use super::{
    process_card_field_keydown, step_interaction_card_ops::StepInteractionCardOps,
    FIELD_INPUT_CLASS,
};

/// A card for one step's thing-interaction list.
#[component]
pub(crate) fn StepInteractionCard(
    input_diagram: Signal<InputDiagram<'static>>,
    process_id: String,
    step_id: String,
    edge_ids: Vec<String>,
) -> Element {
    rsx! {
        div {
            class: INNER_CARD_CLASS,

            // Step ID + remove
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
                    onkeydown: move |evt| {
                        process_card_field_keydown(evt);
                    },
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
                    onkeydown: move |evt| {
                        process_card_field_keydown(evt);
                    },
                    "\u{2715}"
                }
            }

            // Edge group IDs
            div {
                class: "flex flex-col gap-1 pl-4",

                for (idx, edge_group_id) in edge_ids.iter().enumerate() {
                    {
                        let edge_group_id = edge_group_id.clone();
                        let process_id = process_id.clone();
                        let step_id = step_id.clone();
                        rsx! {
                            div {
                                key: "{process_id}_{step_id}_{idx}",
                                class: ROW_CLASS_SIMPLE,

                                span {
                                    class: "text-xs text-gray-500 w-6 text-right",
                                    "{idx}."
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
                                                input_diagram,
                                                &process_id,
                                                &step_id,
                                                idx,
                                                &evt.value(),
                                            );
                                        }
                                    },
                                    onkeydown: move |evt| {
                                        process_card_field_keydown(evt);
                                    },
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
                                                input_diagram,
                                                &process_id,
                                                &step_id,
                                                idx,
                                            );
                                        }
                                    },
                                    onkeydown: move |evt| {
                                        process_card_field_keydown(evt);
                                    },
                                    "\u{2715}"
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
                            StepInteractionCardOps::step_interaction_edge_add(input_diagram, &process_id, &step_id);
                        }
                    },
                    onkeydown: move |evt| {
                        process_card_field_keydown(evt);
                    },
                    "+ Add edge group"
                }
            }
        }
    }
}
