//! A single tag-focus section within the tags focus page.
//!
//! Shows a header with the tag key (select for `tag_defaults`, text input for
//! custom tags) and a remove button, then embeds a [`ThemeStylesEditor`]
//! targeting that specific tag key.

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};
use disposition::input_model::InputDiagram;

use crate::components::editor::{
    common::{parse_tag_id_or_defaults, CARD_CLASS, INPUT_CLASS, REMOVE_BTN, ROW_CLASS_SIMPLE},
    datalists::list_ids,
    theme_styles_editor::{ThemeStylesEditor, ThemeStylesTarget},
};

// === TagFocusSection === //

/// A single tag-focus section within the tags focus page.
///
/// Shows a header with the tag key -- either a static `"tag_defaults"` label
/// or an editable text `<input>` for custom tag IDs -- and a remove button,
/// then embeds a [`ThemeStylesEditor`] targeting that specific tag key.
#[component]
pub fn TagFocusSection(input_diagram: Signal<InputDiagram<'static>>, tag_key: String) -> Element {
    let is_defaults = tag_key == "tag_defaults";

    rsx! {
        div {
            class: CARD_CLASS,

            // === Header: tag key + remove === //
            div {
                class: ROW_CLASS_SIMPLE,

                label {
                    class: "text-xs text-gray-500 w-14 shrink-0",
                    "Tag"
                }

                if is_defaults {
                    span {
                        class: "text-sm font-mono text-gray-300 px-2 py-1",
                        "tag_defaults"
                    }
                } else {
                    input {
                        class: INPUT_CLASS,
                        style: "max-width:14rem",
                        list: list_ids::TAG_IDS,
                        placeholder: "tag_id",
                        value: "{tag_key}",
                        onchange: {
                            let old_key = tag_key.clone();
                            move |evt: dioxus::events::FormEvent| {
                                let new_val = evt.value();
                                if new_val != old_key
                                    && let (Some(old_tag), Some(new_tag)) = (
                                        parse_tag_id_or_defaults(&old_key),
                                        parse_tag_id_or_defaults(&new_val),
                                    ) {
                                        let mut diagram = input_diagram.write();
                                        if !diagram
                                            .theme_tag_things_focus
                                            .contains_key(&new_tag)
                                            && let Some(idx) = diagram
                                                .theme_tag_things_focus
                                                .get_index_of(&old_tag)
                                            {
                                                diagram
                                                    .theme_tag_things_focus
                                                    .replace_index(idx, new_tag)
                                                    .expect(
                                                        "Expected new key to be unique; \
                                                         checked for availability above",
                                                    );
                                            }
                                    }
                            }
                        },
                    }
                }

                button {
                    class: REMOVE_BTN,
                    onclick: {
                        let key = tag_key.clone();
                        move |_| {
                            if let Some(parsed) = parse_tag_id_or_defaults(&key) {
                                let mut diagram = input_diagram.write();
                                diagram.theme_tag_things_focus.shift_remove(&parsed);
                            }
                        }
                    },
                    "x Remove tag"
                }
            }

            // === Inner ThemeStyles editor === //
            ThemeStylesEditor {
                input_diagram,
                target: ThemeStylesTarget::TagFocus {
                    tag_key: tag_key.clone(),
                },
            }
        }
    }
}
