//! A single entity-type section within the types styles page.
//!
//! Shows a header with the type key (editable) and a remove button, then
//! embeds a [`ThemeStylesEditor`] targeting that specific type key.

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};
use disposition::input_model::InputDiagram;

use crate::components::editor::{
    common::{parse_entity_type_id, CARD_CLASS, INPUT_CLASS, REMOVE_BTN, ROW_CLASS_SIMPLE},
    datalists::list_ids,
    theme_styles_editor::{ThemeStylesEditor, ThemeStylesTarget},
};

// === TypesStylesSection === //

/// A single entity-type section within the types styles page.
///
/// Shows a header with the type key (editable) and a remove button, then
/// embeds a [`ThemeStylesEditor`] targeting that specific type key.
#[component]
pub fn TypesStylesSection(
    input_diagram: Signal<InputDiagram<'static>>,
    type_key: String,
) -> Element {
    rsx! {
        div {
            class: CARD_CLASS,

            // === Header: type key + remove === //
            div {
                class: ROW_CLASS_SIMPLE,

                label {
                    class: "text-xs text-gray-500 w-20 shrink-0",
                    "Entity Type"
                }

                input {
                    class: INPUT_CLASS,
                    style: "max-width:14rem",
                    list: list_ids::ENTITY_IDS,
                    placeholder: "type_id",
                    value: "{type_key}",
                    onchange: {
                        let old_key = type_key.clone();
                        move |evt: dioxus::events::FormEvent| {
                            let new_val = evt.value();
                            if new_val != old_key {
                                if let (Some(old_id), Some(new_id)) = (
                                    parse_entity_type_id(&old_key),
                                    parse_entity_type_id(&new_val),
                                ) {
                                    let mut diagram = input_diagram.write();
                                    if !diagram.theme_types_styles.contains_key(&new_id) {
                                        if let Some(idx) =
                                            diagram.theme_types_styles.get_index_of(&old_id)
                                        {
                                            diagram
                                                .theme_types_styles
                                                .replace_index(idx, new_id)
                                                .expect(
                                                    "Expected new key to be unique; \
                                                     checked for availability above",
                                                );
                                        }
                                    }
                                }
                            }
                        }
                    },
                }

                span {
                    class: REMOVE_BTN,
                    onclick: {
                        let key = type_key.clone();
                        move |_| {
                            if let Some(type_id) = parse_entity_type_id(&key) {
                                let mut diagram = input_diagram.write();
                                diagram.theme_types_styles.shift_remove(&type_id);
                            }
                        }
                    },
                    "x Remove type"
                }
            }

            // === Inner ThemeStyles editor === //
            ThemeStylesEditor {
                input_diagram,
                target: ThemeStylesTarget::TypesStyles {
                    entity_type_key: type_key.clone(),
                },
            }
        }
    }
}
