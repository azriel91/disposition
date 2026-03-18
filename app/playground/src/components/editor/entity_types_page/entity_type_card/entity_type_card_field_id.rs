//! Entity ID field with remove button.
//!
//! Extracted from [`EntityTypeCard`] to keep the parent component concise.
//!
//! [`EntityTypeCard`]: super::EntityTypeCard

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::EntityTypesPageOps;

use crate::components::editor::{
    common::{FieldNav, RenameRefocus, RenameRefocusTarget, REMOVE_BTN, ROW_CLASS_SIMPLE},
    datalists::list_ids,
    entity_types_page::{DATA_ATTR, FIELD_INPUT_CLASS},
};

/// Entity ID input and remove button.
///
/// Displays the entity ID input (with datalist) and a remove button.
/// Handles ID rename with post-rename refocus.
#[component]
pub(crate) fn EntityTypeCardFieldId(
    input_diagram: Signal<InputDiagram<'static>>,
    entity_id: String,
    types: Vec<String>,
    rename_target: Signal<RenameRefocusTarget>,
    mut rename_refocus: Signal<Option<RenameRefocus>>,
) -> Element {
    rsx! {
        div {
            class: ROW_CLASS_SIMPLE,

            label {
                class: "text-xs text-gray-500 w-12",
                "Entity"
            }

            input {
                class: FIELD_INPUT_CLASS,
                style: "max-width:14rem",
                tabindex: "-1",
                list: list_ids::ENTITY_IDS,
                placeholder: "entity_id",
                value: "{entity_id}",
                onchange: {
                    let entity_id_old = entity_id.clone();
                    let current_types = types.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let id_new = evt.value();
                        let target = *rename_target.read();
                        EntityTypesPageOps::entry_rename(
                            &mut input_diagram.write(),
                            &entity_id_old,
                            &id_new,
                            &current_types,
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
                    let entity_id = entity_id.clone();
                    move |_| {
                        EntityTypesPageOps::entry_remove(&mut input_diagram.write(), &entity_id);
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "x Remove"
            }
        }
    }
}
