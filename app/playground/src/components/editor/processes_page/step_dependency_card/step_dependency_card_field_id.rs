//! Step ID field with remove button.
//!
//! Extracted from [`StepDependencyCard`] to keep the parent component concise.
//!
//! [`StepDependencyCard`]: super::StepDependencyCard

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::StepDependencyCardOps;

use crate::components::editor::{
    common::{FieldNav, REMOVE_BTN, ROW_CLASS_SIMPLE},
    datalists::list_ids,
    processes_page::{DATA_ATTR, FIELD_INPUT_CLASS},
};

/// Step ID input and remove button for a step-dependency card.
///
/// Displays the step ID input (with a datalist scoped to the process's steps)
/// and a remove button. Handles step rename and removal.
#[component]
pub(crate) fn StepDependencyCardFieldId(
    input_diagram: Signal<InputDiagram<'static>>,
    process_id: String,
    step_id: String,
    dep_ids: Vec<String>,
) -> Element {
    let step_ids_list = list_ids::process_step_ids_for(&process_id);

    rsx! {
        div {
            class: ROW_CLASS_SIMPLE,

            input {
                class: FIELD_INPUT_CLASS,
                style: "max-width:14rem",
                tabindex: "-1",
                list: "{step_ids_list}",
                placeholder: "step_id",
                value: "{step_id}",
                onchange: {
                    let process_id = process_id.clone();
                    let step_id_old = step_id.clone();
                    let dep_ids = dep_ids.clone();
                    move |evt: dioxus::events::FormEvent| {
                        StepDependencyCardOps::step_dependency_rename(
                            &mut input_diagram.write(),
                            &process_id,
                            &step_id_old,
                            &evt.value(),
                            &dep_ids,
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
                        StepDependencyCardOps::step_dependency_remove(&mut input_diagram.write(), &process_id, &step_id);
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "\u{2715}"
            }
        }
    }
}
