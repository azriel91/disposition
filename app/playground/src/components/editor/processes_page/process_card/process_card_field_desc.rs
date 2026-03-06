//! Process description field.
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
    common::{FieldNav, ROW_CLASS_SIMPLE, TEXTAREA_CLASS},
    processes_page::{processes_page_ops::ProcessesPageOps, DATA_ATTR},
};

/// Process description textarea.
///
/// Displays a labelled textarea for the process's markdown description.
/// Updates are applied immediately via
/// [`ProcessesPageOps::process_desc_update`].
#[component]
pub(crate) fn ProcessCardFieldDesc(
    input_diagram: Signal<InputDiagram<'static>>,
    process_id: String,
    entry_desc: String,
) -> Element {
    rsx! {
        div {
            class: ROW_CLASS_SIMPLE,

            label {
                class: "text-xs text-gray-500 w-20",
                "Description"
            }
            textarea {
                class: TEXTAREA_CLASS,
                tabindex: "-1",
                placeholder: "Process description (markdown)",
                value: "{entry_desc}",
                oninput: {
                    let process_id = process_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        ProcessesPageOps::process_desc_update(input_diagram, &process_id, &evt.value());
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
            }
        }
    }
}
