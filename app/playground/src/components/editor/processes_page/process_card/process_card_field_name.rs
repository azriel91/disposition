//! Process display name field.
//!
//! Extracted from [`ProcessCard`] to keep the parent component concise.
//!
//! [`ProcessCard`]: super::ProcessCard

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::ProcessesPageOps;

use crate::components::editor::{
    common::{FieldNav, ROW_CLASS_SIMPLE},
    processes_page::{DATA_ATTR, FIELD_INPUT_CLASS},
};

/// Process display name input.
///
/// Displays a labelled text input for the process's human-readable name.
/// Updates are applied immediately via
/// [`ProcessesPageOps::process_name_update`].
#[component]
pub(crate) fn ProcessCardFieldName(
    input_diagram: Signal<InputDiagram<'static>>,
    process_id: String,
    entry_name: String,
) -> Element {
    rsx! {
        div {
            class: ROW_CLASS_SIMPLE,

            label {
                class: "text-xs text-gray-500 w-20",
                "Name"
            }
            input {
                class: FIELD_INPUT_CLASS,
                tabindex: "-1",
                placeholder: "Display name",
                value: "{entry_name}",
                oninput: {
                    let process_id = process_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        ProcessesPageOps::process_name_update(&mut input_diagram.write(), &process_id, &evt.value());
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
            }
        }
    }
}
