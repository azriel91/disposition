//! Process ID field with remove button.
//!
//! Extracted from [`ProcessCard`] to keep the parent component concise.
//!
//! [`ProcessCard`]: super::ProcessCard

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::ProcessesPageOps;

use crate::components::editor::{
    common::{FieldNav, RenameRefocus, RenameRefocusTarget, REMOVE_BTN, ROW_CLASS_SIMPLE},
    datalists::list_ids,
    processes_page::{DATA_ATTR, FIELD_INPUT_CLASS},
};

/// Process ID input and remove button.
///
/// Displays the process ID input (with datalist) and a remove button.
/// Handles ID rename with post-rename refocus.
#[component]
pub(crate) fn ProcessCardFieldId(
    input_diagram: Signal<InputDiagram<'static>>,
    process_id: String,
    rename_target: Signal<RenameRefocusTarget>,
    mut rename_refocus: Signal<Option<RenameRefocus>>,
) -> Element {
    rsx! {
        div {
            class: ROW_CLASS_SIMPLE,

            label {
                class: "text-xs text-gray-500 w-20",
                "Process ID"
            }
            input {
                class: FIELD_INPUT_CLASS,
                style: "max-width:16rem",
                tabindex: "-1",
                list: list_ids::PROCESS_IDS,
                placeholder: "process_id",
                value: "{process_id}",
                onchange: {
                    let process_id_old = process_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let id_new = evt.value();
                        let target = *rename_target.read();
                        ProcessesPageOps::process_rename(
                            &mut input_diagram.write(),
                            &process_id_old,
                            &id_new,
                        );
                        rename_refocus.set(Some(RenameRefocus {
                            new_id: id_new,
                            target,
                        }));
                    }
                },
                onkeydown: FieldNav::id_onkeydown(DATA_ATTR, rename_target),
            }

            button {
                class: REMOVE_BTN,
                tabindex: "-1",
                "data-action": "remove",
                onclick: {
                    let process_id = process_id.clone();
                    move |_| {
                        ProcessesPageOps::process_remove(&mut input_diagram.write(), &process_id);
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "\u{2715} Remove"
            }
        }
    }
}
