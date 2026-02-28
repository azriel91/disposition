//! Style aliases section of a
//! [`CssClassPartialsCard`](super::css_class_partials_card::CssClassPartialsCard).
//!
//! Shows the list of `style_aliases_applied` for a single
//! `IdOrDefaults -> CssClassPartials` entry, with per-alias edit and remove
//! controls plus an "add alias" button.

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};
use disposition::{
    input_model::{theme::StyleAlias, InputDiagram},
    model_common::Id,
};

use crate::components::editor::{
    common::{ADD_BTN, INPUT_CLASS, LABEL_CLASS, REMOVE_BTN, ROW_CLASS_SIMPLE},
    datalists::list_ids,
    theme_styles_editor::{
        css_class_partials_card::css_card_field_keydown, parse_id_or_defaults, ThemeStylesTarget,
    },
};

// === CssClassPartialsCardAliases === //

/// Renders the "Style aliases applied" section of a single card.
///
/// Displays each alias as an editable `<input>` with a remove button, and an
/// "+ Add alias" button at the bottom.
#[component]
pub fn CssClassPartialsCardAliases(
    input_diagram: Signal<InputDiagram<'static>>,
    target: ThemeStylesTarget,
    entry_key: String,
    style_aliases: Vec<String>,
) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-1 pl-4",

            label {
                class: LABEL_CLASS,
                "Style aliases applied"
            }

            for (alias_idx, alias_name) in style_aliases.iter().enumerate() {
                {
                    let alias_name = alias_name.clone();
                    let key = entry_key.clone();
                    let target = target.clone();
                    rsx! {
                        CssClassPartialsCardAliasRow {
                            key: "alias_{alias_idx}_{alias_name}",
                            input_diagram,
                            target,
                            entry_key: key,
                            alias_index: alias_idx,
                            alias_name,
                        }
                    }
                }
            }

            button {
                class: ADD_BTN,
                tabindex: -1,
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
                                // Default to `shade_light` as a sensible starting alias.
                                partials
                                    .style_aliases_applied
                                    .push(StyleAlias::ShadeLight);
                            }
                        }
                    }
                },
                onkeydown: move |evt| {
                    css_card_field_keydown(evt);
                },
                "+ Add alias"
            }
        }
    }
}

// === CssClassPartialsCardAliasRow === //

/// A single row within the style aliases section.
///
/// Contains an editable `<input>` for the alias name and a remove button.
#[component]
fn CssClassPartialsCardAliasRow(
    input_diagram: Signal<InputDiagram<'static>>,
    target: ThemeStylesTarget,
    entry_key: String,
    alias_index: usize,
    alias_name: String,
) -> Element {
    rsx! {
        div {
            class: ROW_CLASS_SIMPLE,

            input {
                class: INPUT_CLASS,
                style: "max-width:12rem",
                tabindex: "-1",
                list: list_ids::STYLE_ALIASES,
                placeholder: "style_alias",
                value: "{alias_name}",
                onchange: {
                    let key = entry_key.clone();
                    let target = target.clone();
                    let alias_idx = alias_index;
                    move |evt: dioxus::events::FormEvent| {
                        let new_val = evt.value();
                        if let Some(parsed_key) = parse_id_or_defaults(&key) {
                            // Parse the alias through serde round-trip:
                            // StyleAlias::from(Id) handles builtin matching.
                            if let Ok(new_alias_id) = Id::new(&new_val) {
                                let new_alias =
                                    StyleAlias::from(new_alias_id.into_static()).into_static();
                                let mut diagram = input_diagram.write();
                                let Some(styles) = target.write_mut(&mut diagram) else {
                                    return;
                                };
                                if let Some(partials) = styles.get_mut(&parsed_key)
                                    && alias_idx < partials.style_aliases_applied.len()
                                {
                                    partials.style_aliases_applied[alias_idx] = new_alias;
                                }
                            }
                        }
                    }
                },
                onkeydown: move |evt| {
                    css_card_field_keydown(evt);
                },
            }

            span {
                class: REMOVE_BTN,
                tabindex: "-1",
                "data-action": "remove",
                onclick: {
                    let key = entry_key.clone();
                    let target = target.clone();
                    let alias_idx = alias_index;
                    move |_| {
                        if let Some(parsed_key) = parse_id_or_defaults(&key) {
                            let mut diagram = input_diagram.write();
                            let Some(styles) = target.write_mut(&mut diagram) else {
                                return;
                            };
                            if let Some(partials) = styles.get_mut(&parsed_key)
                                && alias_idx < partials.style_aliases_applied.len()
                            {
                                partials.style_aliases_applied.remove(alias_idx);
                            }
                        }
                    }
                },
                onkeydown: move |evt| {
                    css_card_field_keydown(evt);
                },
                "x"
            }
        }
    }
}
