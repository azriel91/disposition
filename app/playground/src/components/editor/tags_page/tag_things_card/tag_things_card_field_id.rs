//! Tag ID field with remove button.
//!
//! Extracted from [`TagThingsCard`] to keep the parent component concise.
//!
//! [`TagThingsCard`]: super::TagThingsCard

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::TagsPageOps;

use crate::components::editor::{
    common::{FieldNav, RenameRefocus, RenameRefocusTarget, REMOVE_BTN, ROW_CLASS_SIMPLE},
    datalists::list_ids,
    tags_page::{DATA_ATTR, FIELD_INPUT_CLASS},
};

/// Tag ID input and remove button.
///
/// Displays the tag ID input (with datalist) and a remove button.
/// Handles ID rename with post-rename refocus.
#[component]
pub(crate) fn TagThingsCardFieldId(
    input_diagram: Signal<InputDiagram<'static>>,
    tag_id: String,
    things: Vec<String>,
    rename_target: Signal<RenameRefocusTarget>,
    mut rename_refocus: Signal<Option<RenameRefocus>>,
) -> Element {
    rsx! {
        div {
            class: ROW_CLASS_SIMPLE,

            label {
                class: "text-xs text-gray-500 w-12",
                "Tag"
            }

            input {
                class: FIELD_INPUT_CLASS,
                style: "max-width:14rem",
                tabindex: "-1",
                list: list_ids::TAG_IDS,
                placeholder: "tag_id",
                value: "{tag_id}",
                onchange: {
                    let tag_id_old = tag_id.clone();
                    let current_things = things.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let id_new = evt.value();
                        let target = *rename_target.read();
                        TagsPageOps::tag_things_entry_rename(
                            &mut input_diagram.write(),
                            &tag_id_old,
                            &id_new,
                            &current_things,
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
                    let tag_id = tag_id.clone();
                    move |_| {
                        TagsPageOps::tag_things_entry_remove(&mut input_diagram.write(), &tag_id);
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "x Remove"
            }
        }
    }
}
