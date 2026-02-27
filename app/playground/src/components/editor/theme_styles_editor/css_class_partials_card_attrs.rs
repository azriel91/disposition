//! Theme attributes section of a
//! [`CssClassPartialsCard`](super::css_class_partials_card::CssClassPartialsCard).
//!
//! Shows the `partials` map (`ThemeAttr -> value`) for a single
//! `IdOrDefaults -> CssClassPartials` entry, with per-attribute name
//! dropdown, value input, and remove controls, plus an "add attribute"
//! button.

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};
use disposition::input_model::InputDiagram;

use crate::components::editor::{
    common::{ADD_BTN, INPUT_CLASS, LABEL_CLASS, REMOVE_BTN, ROW_CLASS_SIMPLE, SELECT_CLASS},
    theme_styles_editor::{parse_id_or_defaults, parse_theme_attr, ThemeStylesTarget, THEME_ATTRS},
};

// === CssClassPartialsCardAttrs === //

/// Renders the "Attributes" section of a single card.
///
/// Displays each `ThemeAttr -> value` pair as a row with a `<select>` for the
/// attribute name, a text `<input>` for the value, and a remove button. An
/// "+ Add attribute" button appends a new row with the first unused attribute.
#[component]
pub fn CssClassPartialsCardAttrs(
    input_diagram: Signal<InputDiagram<'static>>,
    target: ThemeStylesTarget,
    entry_key: String,
    theme_attrs: Vec<(String, String)>,
) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-1 pl-4",

            label {
                class: LABEL_CLASS,
                "Attributes"
            }

            for (attr_idx, (attr_name, attr_value)) in theme_attrs.iter().enumerate() {
                {
                    let attr_name = attr_name.clone();
                    let attr_value = attr_value.clone();
                    let key = entry_key.clone();
                    let target = target.clone();
                    rsx! {
                        CssClassPartialsCardAttrRow {
                            key: "attr_{attr_idx}_{attr_name}",
                            input_diagram,
                            target,
                            entry_key: key,
                            attr_name,
                            attr_value,
                        }
                    }
                }
            }

            div {
                class: ADD_BTN,
                onclick: {
                    let key = entry_key.clone();
                    let target = target.clone();
                    move |_| {
                        if let Some(parsed_key) = parse_id_or_defaults(&key) {
                            let mut diagram = input_diagram.write();
                            let Some(styles) = target.write_mut(&mut diagram) else {
                                return;
                            };
                            if let Some(partials) = styles.get_mut(&parsed_key) {
                                // Find first ThemeAttr not yet present.
                                let new_attr = THEME_ATTRS
                                    .iter()
                                    .find(|(_, attr)| !partials.partials.contains_key(attr))
                                    .map(|(_, attr)| *attr);
                                if let Some(attr) = new_attr {
                                    partials.partials.insert(attr, String::new());
                                }
                            }
                        }
                    }
                },
                "+ Add attribute"
            }
        }
    }
}

// === CssClassPartialsCardAttrRow === //

/// A single row within the attributes section.
///
/// Contains a `<select>` dropdown for the attribute name, a text `<input>`
/// for the attribute value, and a remove button.
#[component]
fn CssClassPartialsCardAttrRow(
    input_diagram: Signal<InputDiagram<'static>>,
    target: ThemeStylesTarget,
    entry_key: String,
    attr_name: String,
    attr_value: String,
) -> Element {
    rsx! {
        div {
            class: ROW_CLASS_SIMPLE,

            // === Attribute name dropdown === //
            select {
                class: SELECT_CLASS,
                value: "{attr_name}",
                onchange: {
                    let key = entry_key.clone();
                    let old_attr_name = attr_name.clone();
                    let current_value = attr_value.clone();
                    let target = target.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let new_attr_str = evt.value();
                        if let (Some(old_attr), Some(new_attr)) = (
                            parse_theme_attr(&old_attr_name),
                            parse_theme_attr(&new_attr_str),
                        )
                            && old_attr != new_attr
                            && let Some(parsed_key) = parse_id_or_defaults(&key)
                        {
                            let mut diagram = input_diagram.write();
                            let Some(styles) = target.write_mut(&mut diagram) else {
                                return;
                            };
                            if let Some(partials) = styles.get_mut(&parsed_key) {
                                partials.partials.shift_remove(&old_attr);
                                partials
                                    .partials
                                    .insert(new_attr, current_value.clone());
                            }
                        }
                    }
                },

                for (name, _) in THEME_ATTRS.iter() {
                    option {
                        value: "{name}",
                        selected: *name == attr_name.as_str(),
                        "{name}"
                    }
                }
            }

            // === Attribute value === //
            input {
                class: INPUT_CLASS,
                style: "max-width:8rem",
                placeholder: "value",
                value: "{attr_value}",
                onchange: {
                    let key = entry_key.clone();
                    let attr_name = attr_name.clone();
                    let target = target.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let new_val = evt.value();
                        if let Some(attr) = parse_theme_attr(&attr_name)
                            && let Some(parsed_key) = parse_id_or_defaults(&key)
                        {
                            let mut diagram = input_diagram.write();
                            let Some(styles) = target.write_mut(&mut diagram) else {
                                return;
                            };
                            if let Some(partials) = styles.get_mut(&parsed_key)
                                && let Some(v) = partials.partials.get_mut(&attr)
                            {
                                *v = new_val;
                            }
                        }
                    }
                },
            }

            // === Remove button === //
            span {
                class: REMOVE_BTN,
                onclick: {
                    let key = entry_key.clone();
                    let attr_name = attr_name.clone();
                    let target = target.clone();
                    move |_| {
                        if let Some(attr) = parse_theme_attr(&attr_name)
                            && let Some(parsed_key) = parse_id_or_defaults(&key)
                        {
                            let mut diagram = input_diagram.write();
                            let Some(styles) = target.write_mut(&mut diagram) else {
                                return;
                            };
                            if let Some(partials) = styles.get_mut(&parsed_key) {
                                partials.partials.shift_remove(&attr);
                            }
                        }
                    }
                },
                "x"
            }
        }
    }
}
