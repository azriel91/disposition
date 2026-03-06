//! Edge group ID field with remove button.
//!
//! Extracted from [`EdgeGroupCard`] to keep the parent component concise.
//!
//! [`EdgeGroupCard`]: super::EdgeGroupCard

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;

use crate::components::editor::{
    common::{FieldNav, RenameRefocus, RenameRefocusTarget, REMOVE_BTN, ROW_CLASS_SIMPLE},
    datalists::list_ids,
    thing_dependencies_page::{
        edge_group_card_ops::EdgeGroupCardOps, MapTarget, DATA_ATTR, FIELD_INPUT_CLASS,
    },
};

/// Edge group ID input and remove button.
///
/// Displays the edge group ID input (with datalist) and a remove button.
/// Handles ID rename with post-rename refocus.
#[component]
pub(crate) fn EdgeGroupCardFieldId(
    input_diagram: Signal<InputDiagram<'static>>,
    target: MapTarget,
    edge_group_id: String,
    rename_target: Signal<RenameRefocusTarget>,
    mut rename_refocus: Signal<Option<RenameRefocus>>,
) -> Element {
    rsx! {
        div {
            class: ROW_CLASS_SIMPLE,

            input {
                class: FIELD_INPUT_CLASS,
                style: "max-width:16rem",
                tabindex: "-1",
                list: list_ids::EDGE_GROUP_IDS,
                placeholder: "edge_group_id",
                value: "{edge_group_id}",
                onchange: {
                    let edge_group_id_old = edge_group_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let id_new = evt.value();
                        let target = *rename_target.read();
                        EdgeGroupCardOps::edge_group_rename(
                            input_diagram,
                            &edge_group_id_old,
                            &id_new,
                        );
                        rename_refocus.set(Some(RenameRefocus {
                            new_id: id_new,
                            target,
                        }));
                    }
                },
                onkeydown: FieldNav::id_onkeydown(DATA_ATTR, rename_target)
            }

            button {
                class: REMOVE_BTN,
                tabindex: "-1",
                "data-action": "remove",
                onclick: {
                    let edge_group_id = edge_group_id.clone();
                    move |_| {
                        EdgeGroupCardOps::edge_group_remove(input_diagram, target, &edge_group_id);
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "x Remove"
            }
        }
    }
}
