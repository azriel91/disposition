//! Header row for a
//! [`CssClassPartialsCard`](super::css_class_partials_card::CssClassPartialsCard).
//!
//! Shows the entry key as either a built-in `<select>` or a custom text
//! `<input>`, a checkbox to toggle between them, and a remove button.

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::{theme::IdOrDefaults, InputDiagram};

use crate::components::editor::{
    common::{INPUT_CLASS, REMOVE_BTN, ROW_CLASS_SIMPLE, SELECT_CLASS},
    datalists::list_ids,
    theme_styles_editor::{parse_id_or_defaults, ThemeStylesTarget, ID_OR_DEFAULTS_BUILTINS},
};

// === CssClassPartialsCardHeader === //

/// Header row for a single card within the theme styles editor.
///
/// Contains:
/// 1. A "Key" label.
/// 2. Either a `<select>` (for built-in defaults) or a text `<input>` (for
///    custom entity IDs).
/// 3. A checkbox to toggle between built-in and custom mode.
/// 4. A remove button to delete the entry.
#[component]
pub fn CssClassPartialsCardHeader(
    input_diagram: Signal<InputDiagram<'static>>,
    target: ThemeStylesTarget,
    entry_key: String,
) -> Element {
    let is_builtin = matches!(
        entry_key.as_str(),
        "node_defaults" | "node_excluded_defaults" | "edge_defaults"
    );

    rsx! {
        div {
            class: ROW_CLASS_SIMPLE,

            label {
                class: "text-xs text-gray-500 w-10 shrink-0",
                "Key"
            }

            if is_builtin {
                {
                    let entry_key = entry_key.clone();
                    let target = target.clone();
                    rsx! {
                        CssClassPartialsCardHeaderBuiltinSelect {
                            input_diagram,
                            target,
                            entry_key,
                        }
                    }
                }
            } else {
                {
                    let entry_key = entry_key.clone();
                    let target = target.clone();
                    rsx! {
                        CssClassPartialsCardHeaderCustomInput {
                            input_diagram,
                            target,
                            entry_key,
                        }
                    }
                }
            }

            // === Toggle built-in / custom === //
            {
                let entry_key = entry_key.clone();
                let target = target.clone();
                rsx! {
                    CssClassPartialsCardHeaderToggle {
                        input_diagram,
                        target,
                        entry_key,
                        is_builtin,
                    }
                }
            }

            // === Remove button === //
            span {
                class: REMOVE_BTN,
                onclick: {
                    let key = entry_key.clone();
                    let target = target.clone();
                    move |_| {
                        if let Some(parsed) = parse_id_or_defaults(&key) {
                            let mut diagram = input_diagram.write();
                            let Some(styles) = target.write_mut(&mut diagram) else {
                                return;
                            };
                            styles.shift_remove(&parsed);
                        }
                    }
                },
                "x Remove"
            }
        }
    }
}

// === CssClassPartialsCardHeaderBuiltinSelect === //

/// Built-in `<select>` dropdown for choosing a well-known key such as
/// `node_defaults`, `node_excluded_defaults`, or `edge_defaults`.
#[component]
fn CssClassPartialsCardHeaderBuiltinSelect(
    input_diagram: Signal<InputDiagram<'static>>,
    target: ThemeStylesTarget,
    entry_key: String,
) -> Element {
    rsx! {
        select {
            class: SELECT_CLASS,
            value: "{entry_key}",
            onchange: {
                let old_key = entry_key.clone();
                let target = target.clone();
                move |evt: dioxus::events::FormEvent| {
                    let new_val = evt.value();
                    if let (Some(old), Some(new)) = (
                        parse_id_or_defaults(&old_key),
                        parse_id_or_defaults(&new_val),
                    )
                        && old != new
                    {
                        let mut diagram = input_diagram.write();
                        let Some(styles) = target.write_mut(&mut diagram) else {
                            return;
                        };
                        if let Some(idx) = styles.get_index_of(&old) {
                            styles
                                .replace_index(idx, new)
                                .expect("Expected new key to be unique after equality check");
                        }
                    }
                }
            },

            for (val, label) in ID_OR_DEFAULTS_BUILTINS.iter() {
                option {
                    value: "{val}",
                    selected: *val == entry_key.as_str(),
                    "{label}"
                }
            }
        }
    }
}

// === CssClassPartialsCardHeaderCustomInput === //

/// Text `<input>` for entering a custom entity ID key.
#[component]
fn CssClassPartialsCardHeaderCustomInput(
    input_diagram: Signal<InputDiagram<'static>>,
    target: ThemeStylesTarget,
    entry_key: String,
) -> Element {
    rsx! {
        input {
            class: INPUT_CLASS,
            style: "max-width:14rem",
            list: list_ids::ENTITY_IDS,
            placeholder: "entity_id",
            value: "{entry_key}",
            onchange: {
                let old_key = entry_key.clone();
                let target = target.clone();
                move |evt: dioxus::events::FormEvent| {
                    let new_val = evt.value();
                    if let (Some(old), Some(new)) = (
                        parse_id_or_defaults(&old_key),
                        parse_id_or_defaults(&new_val),
                    )
                        && old != new
                    {
                        let mut diagram = input_diagram.write();
                        let Some(styles) = target.write_mut(&mut diagram) else {
                            return;
                        };
                        if let Some(idx) = styles.get_index_of(&old) {
                            styles
                                .replace_index(idx, new)
                                .expect("Expected new key to be unique after equality check");
                        }
                    }
                }
            },
        }
    }
}

// === CssClassPartialsCardHeaderToggle === //

/// Checkbox that toggles an entry between built-in `<select>` mode and custom
/// `<input>` mode.
///
/// When switching to custom mode, generates a placeholder key like
/// `"custom_1"`. When switching to built-in mode, picks the first available
/// well-known default key.
#[component]
fn CssClassPartialsCardHeaderToggle(
    input_diagram: Signal<InputDiagram<'static>>,
    target: ThemeStylesTarget,
    entry_key: String,
    is_builtin: bool,
) -> Element {
    rsx! {
        label {
            class: "text-xs text-gray-500 ml-1 flex items-center gap-1 select-none cursor-pointer",
            title: "Toggle between built-in defaults and a custom entity ID",
            input {
                r#type: "checkbox",
                class: "accent-blue-500",
                checked: !is_builtin,
                onchange: {
                    let old_key = entry_key.clone();
                    let target = target.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let wants_custom = evt.value() == "true";
                        let new_key = if wants_custom {
                            toggle_to_custom_key(input_diagram, &target)
                        } else {
                            toggle_to_builtin_key(input_diagram, &target)
                        };
                        if let Some(new) = new_key
                            && let Some(old) = parse_id_or_defaults(&old_key)
                        {
                            let mut diagram = input_diagram.write();
                            let Some(styles) = target.write_mut(&mut diagram) else {
                                return;
                            };
                            if let Some(idx) = styles.get_index_of(&old) {
                                styles
                                    .replace_index(idx, new)
                                    .expect(
                                        "Expected new key to be unique; \
                                         checked for availability above",
                                    );
                            }
                        }
                    }
                },
            }
            "ID"
        }
    }
}

// === Helper functions === //

/// Find the first available placeholder custom key (e.g. `"custom_1"`,
/// `"custom_2"`, etc.) that does not collide with existing entries.
fn toggle_to_custom_key(
    input_diagram: Signal<InputDiagram<'static>>,
    target: &ThemeStylesTarget,
) -> Option<IdOrDefaults<'static>> {
    let mut n = 1u32;
    loop {
        let candidate = format!("custom_{n}");
        if let Some(id) = parse_id_or_defaults(&candidate) {
            let diagram = input_diagram.read();
            let styles = target.read(&diagram);
            if let Some(styles) = styles {
                if !styles.contains_key(&id) {
                    drop(diagram);
                    break Some(id);
                }
            } else {
                drop(diagram);
                break Some(id);
            }
            drop(diagram);
        }
        n += 1;
    }
}

/// Find the first available built-in default key (`NodeDefaults`,
/// `NodeExcludedDefaults`, `EdgeDefaults`) that does not collide with
/// existing entries.
fn toggle_to_builtin_key(
    input_diagram: Signal<InputDiagram<'static>>,
    target: &ThemeStylesTarget,
) -> Option<IdOrDefaults<'static>> {
    let diagram = input_diagram.read();
    let styles = target.read(&diagram);
    let key = styles.and_then(|styles| {
        [
            IdOrDefaults::NodeDefaults,
            IdOrDefaults::NodeExcludedDefaults,
            IdOrDefaults::EdgeDefaults,
        ]
        .into_iter()
        .find(|k| !styles.contains_key(k))
    });
    drop(diagram);
    key
}
