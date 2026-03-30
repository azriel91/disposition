//! A single entity-type section within the types styles page.
//!
//! Shows a header with the type key (editable) and a remove button, then
//! embeds a [`ThemeStylesEditor`] targeting that specific type key.

use dioxus::{
    hooks::use_context,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Memo, ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_ir_rt::ThemeValueSource;

use crate::components::editor::{
    common::{parse_entity_type_id, CARD_CLASS, INPUT_CLASS, REMOVE_BTN, ROW_CLASS_SIMPLE},
    datalists::list_ids,
    theme_styles_editor::{ThemeStylesEditor, ThemeStylesTarget},
};

// === CSS === //

/// CSS classes for the "revert to base" button.
const REVERT_BTN: &str = "\
    text-xs \
    text-amber-400 \
    hover:text-amber-300 \
    cursor-pointer \
    select-none\
";

// === TypesStylesSection === //

/// A single entity-type section within the types styles page.
///
/// Shows a header with the type key (editable) and a remove button, then
/// embeds a [`ThemeStylesEditor`] targeting that specific type key.
#[component]
pub fn TypesStylesSection(
    input_diagram: Signal<InputDiagram<'static>>,
    type_key: String,
    value_source: ThemeValueSource,
) -> Element {
    let base_diagram: Memo<InputDiagram<'static>> = use_context();

    rsx! {
        div {
            class: CARD_CLASS,
            "data-input-diagram-field": "{type_key}_type_styles_section",

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
                    list: list_ids::ENTITY_TYPE_IDS,
                    placeholder: "type_id",
                    value: "{type_key}",
                    onchange: {
                        let old_key = type_key.clone();
                        move |evt: dioxus::events::FormEvent| {
                            let new_val = evt.value();
                            if new_val != old_key
                                && let (Some(old_id), Some(new_id)) = (
                                    parse_entity_type_id(&old_key),
                                    parse_entity_type_id(&new_val),
                                ) {
                                    let base = base_diagram.read();
                                    let mut diagram = input_diagram.write();
                                    if !diagram.theme_types_styles.contains_key(&new_id) {
                                        // If entry exists only in base, copy it into the overlay first.
                                        if !diagram.theme_types_styles.contains_key(&old_id)
                                            && let Some(base_styles) = base.theme_types_styles.get(&old_id) {
                                                diagram.theme_types_styles.insert(old_id.clone(), base_styles.clone());
                                            }
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
                    },
                }

                button {
                    class: REMOVE_BTN,
                    onclick: {
                        let key = type_key.clone();
                        move |_| {
                            if let Some(type_id) = parse_entity_type_id(&key) {
                                let mut diagram = input_diagram.write();
                                diagram.theme_types_styles.remove(&type_id);
                            }
                        }
                    },
                    "x Remove type"
                }
            }

            // === Value source indicator === //
            if value_source == ThemeValueSource::UserInput {
                div {
                    class: "flex flex-row items-center gap-2 text-xs",
                    span {
                        class: "text-amber-400",
                        "Overrides base styles"
                    }
                    button {
                        class: REVERT_BTN,
                        tabindex: "0",
                        onclick: {
                            let key = type_key.clone();
                            move |_| {
                                if let Some(type_id) = parse_entity_type_id(&key) {
                                    let mut diagram = input_diagram.write();
                                    diagram.theme_types_styles.remove(&type_id);
                                }
                            }
                        },
                        "Revert to base"
                    }
                }
            } else {
                div {
                    class: "text-xs text-gray-500 italic",
                    "From disposition's base styles"
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
