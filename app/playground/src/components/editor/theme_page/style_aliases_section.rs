//! A single style-alias section within the style aliases page.
//!
//! Shows a header with the alias name (editable) and a remove button, then
//! embeds sub-sections for editing the `style_aliases_applied` list and the
//! `partials` (`ThemeAttr -> value`) map within the `CssClassPartials` value.
//!
//! Supports keyboard shortcuts:
//!
//! - **ArrowUp / ArrowDown**: navigate between sibling cards.
//! - **ArrowRight**: expand the card (when collapsed).
//! - **ArrowLeft**: collapse the card (when expanded).
//! - **Space**: toggle expand/collapse.
//! - **Enter** (on card): focus the first input inside the card for editing.
//! - **Escape** (on card): focus the parent section / tab.
//! - **Tab / Shift+Tab** (inside a field): cycle through focusable fields
//!   within the card. Wraps from last to first / first to last.
//! - **Esc** (inside a field): return focus to the card wrapper.
//! - **Space** (inside a field): stop propagation.

use dioxus::{
    hooks::use_signal,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::{
    input_model::{
        theme::{StyleAlias, ThemeAttr},
        InputDiagram,
    },
    model_common::Id,
};

use crate::components::editor::{
    common::{ADD_BTN, INPUT_CLASS, LABEL_CLASS, REMOVE_BTN, ROW_CLASS_SIMPLE, SELECT_CLASS},
    datalists::list_ids,
    keyboard_nav::{self, CardKeyAction},
    theme_styles_editor::{
        parse_theme_attr, theme_attr_entry::ThemeAttrEntry, theme_attr_name, THEME_ATTRS,
    },
};

// === Helpers === //

/// Try to construct a `StyleAlias<'static>` from a string, returning `None`
/// if the string is not a valid identifier.
///
/// Valid values: `"shade_light"`, `"padding_normal"`, `"my_custom_alias"`.
fn parse_style_alias(s: &str) -> Option<StyleAlias<'static>> {
    Id::new(s)
        .ok()
        .map(|id| StyleAlias::from(id.into_static()).into_static())
}

// === Data attribute for the card wrapper === //

/// The `data-*` attribute placed on each `StyleAliasesSection` wrapper.
///
/// Used by [`keyboard_nav`] helpers to locate the nearest ancestor card.
const DATA_ATTR: &str = "data-style-alias-card";

// === CSS === //

/// CSS classes for the focusable style alias card wrapper.
///
/// Provides focus ring and transitions for keyboard navigation.
const STYLE_ALIAS_CARD_CLASS: &str = "\
    rounded-lg \
    border \
    border-gray-700 \
    bg-gray-900 \
    p-3 \
    mb-2 \
    flex \
    flex-col \
    gap-2 \
    focus:outline-none \
    focus:ring-1 \
    focus:ring-blue-400 \
    transition-all \
    duration-150\
";

// === StyleAliasesSection === //

/// A single style-alias section within the style aliases page.
///
/// Shows a header with the alias name (editable) and a remove button, then
/// embeds sub-sections for editing the `style_aliases_applied` list and the
/// `partials` (`ThemeAttr -> value`) map.
///
/// The card is focusable and supports full keyboard navigation via the
/// shared [`keyboard_nav`] helpers.
#[component]
pub fn StyleAliasesSection(
    input_diagram: Signal<InputDiagram<'static>>,
    alias_key: String,
    style_aliases_applied: Vec<String>,
    theme_attrs: Vec<ThemeAttrEntry>,
) -> Element {
    let mut collapsed = use_signal(|| true);

    let alias_count = style_aliases_applied.len();
    let attr_count = theme_attrs.len();
    let alias_suffix = if alias_count != 1 { "es" } else { "" };
    let attr_suffix = if attr_count != 1 { "s" } else { "" };

    rsx! {
        div {
            class: STYLE_ALIAS_CARD_CLASS,
            tabindex: "0",
            "data-style-alias-card": "true",

            // === Card-level keyboard shortcuts === //
            onkeydown: move |evt| {
                let action = keyboard_nav::card_keydown(evt, DATA_ATTR);
                match action {
                    CardKeyAction::Collapse => collapsed.set(true),
                    CardKeyAction::Expand => collapsed.set(false),
                    CardKeyAction::Toggle => {
                        let is_collapsed = *collapsed.read();
                        collapsed.set(!is_collapsed);
                    }
                    CardKeyAction::EnterEdit => collapsed.set(false),
                    CardKeyAction::None => {}
                }
            },

            if *collapsed.read() {
                // === Collapsed summary === //
                div {
                    class: "\
                        flex \
                        flex-row \
                        items-center \
                        gap-3 \
                        cursor-pointer \
                        select-none\
                    ",
                    onclick: move |_| collapsed.set(false),

                    // Expand chevron
                    span {
                        class: "text-gray-500 text-xs",
                        ">"
                    }

                    span {
                        class: "text-sm font-mono text-blue-400",
                        "{alias_key}"
                    }

                    span {
                        class: "text-xs text-gray-500",
                        "({alias_count} alias{alias_suffix}, {attr_count} attr{attr_suffix})"
                    }
                }
            } else {
                // === Expanded content === //

                // Collapse toggle
                div {
                    class: "flex flex-row items-center gap-1 cursor-pointer select-none mb-1",
                    onclick: move |_| collapsed.set(true),

                    span {
                        class: "text-gray-500 text-xs rotate-90 inline-block",
                        ">"
                    }
                    span {
                        class: "text-xs text-gray-500",
                        "Collapse"
                    }
                }

                // === Header: alias name + remove === //
                StyleAliasesSectionHeader {
                    input_diagram,
                    alias_key: alias_key.clone(),
                }

                // === Style aliases applied === //
                StyleAliasesSectionAliases {
                    input_diagram,
                    alias_key: alias_key.clone(),
                    style_aliases_applied,
                }

                // === Theme attributes (partials map) === //
                StyleAliasesSectionAttrs {
                    input_diagram,
                    alias_key,
                    theme_attrs,
                }
            }
        }
    }
}

// === StyleAliasesSectionHeader === //

/// Header row for a single style alias card.
///
/// Contains an editable `<input>` for the alias name with a datalist for
/// suggestions, and a remove button to delete the entry.
#[component]
fn StyleAliasesSectionHeader(
    input_diagram: Signal<InputDiagram<'static>>,
    alias_key: String,
) -> Element {
    rsx! {
        div {
            class: ROW_CLASS_SIMPLE,

            label {
                class: "text-xs text-gray-500 w-20 shrink-0",
                "Alias Name"
            }

            input {
                class: INPUT_CLASS,
                style: "max-width:14rem",
                tabindex: "-1",
                list: list_ids::STYLE_ALIASES,
                placeholder: "style_alias",
                value: "{alias_key}",
                onchange: {
                    let old_key = alias_key.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let new_val = evt.value();
                        if new_val != old_key
                            && let (Some(old_alias), Some(new_alias)) = (
                                parse_style_alias(&old_key),
                                parse_style_alias(&new_val),
                            ) {
                                let mut diagram = input_diagram.write();
                                if !diagram
                                    .theme_default
                                    .style_aliases
                                    .contains_key(&new_alias)
                                    && let Some(idx) = diagram
                                        .theme_default
                                        .style_aliases
                                        .get_index_of(&old_alias)
                                    {
                                        diagram
                                            .theme_default
                                            .style_aliases
                                            .replace_index(idx, new_alias)
                                            .expect(
                                                "Expected new key to be unique; \
                                                 checked for availability above",
                                            );
                                    }
                            }
                    }
                },
                onkeydown: move |evt| {
                    keyboard_nav::field_keydown(evt, DATA_ATTR);
                },
            }

            button {
                class: REMOVE_BTN,
                tabindex: "-1",
                "data-action": "remove",
                onclick: {
                    let key = alias_key.clone();
                    move |_| {
                        if let Some(alias) = parse_style_alias(&key) {
                            let mut diagram = input_diagram.write();
                            diagram.theme_default.style_aliases.shift_remove(&alias);
                        }
                    }
                },
                onkeydown: move |evt| {
                    keyboard_nav::field_keydown(evt, DATA_ATTR);
                },
                "x Remove alias"
            }
        }
    }
}

// === StyleAliasesSectionAliases === //

/// Renders the "Style aliases applied" sub-section of a single style alias
/// card.
///
/// Displays each applied alias as an editable `<input>` with a remove button,
/// and an "+ Add alias" button at the bottom.
#[component]
fn StyleAliasesSectionAliases(
    input_diagram: Signal<InputDiagram<'static>>,
    alias_key: String,
    style_aliases_applied: Vec<String>,
) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-1 pl-4",

            label {
                class: LABEL_CLASS,
                "Style aliases applied"
            }

            for (alias_idx, alias_name) in style_aliases_applied.iter().enumerate() {
                {
                    let alias_name = alias_name.clone();
                    let key = alias_key.clone();
                    rsx! {
                        StyleAliasesSectionAliasRow {
                            key: "applied_{alias_idx}_{alias_name}",
                            input_diagram,
                            alias_key: key,
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
                    let key = alias_key.clone();
                    move |_| {
                        if let Some(parsed_key) = parse_style_alias(&key) {
                            let mut diagram = input_diagram.write();
                            if let Some(partials) =
                                diagram.theme_default.style_aliases.get_mut(&parsed_key)
                            {
                                // Default to `shade_light` as a sensible starting alias.
                                partials
                                    .style_aliases_applied
                                    .push(StyleAlias::ShadeLight);
                            }
                        }
                    }
                },
                onkeydown: move |evt| {
                    keyboard_nav::field_keydown(evt, DATA_ATTR);
                },
                "+ Add alias"
            }
        }
    }
}

// === StyleAliasesSectionAliasRow === //

/// A single row within the style aliases applied sub-section.
///
/// Contains an editable `<input>` for the applied alias name and a remove
/// button.
#[component]
fn StyleAliasesSectionAliasRow(
    input_diagram: Signal<InputDiagram<'static>>,
    alias_key: String,
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
                    let key = alias_key.clone();
                    let alias_idx = alias_index;
                    move |evt: dioxus::events::FormEvent| {
                        let new_val = evt.value();
                        if let Some(parsed_key) = parse_style_alias(&key)
                            && let Ok(new_alias_id) = Id::new(&new_val) {
                                let new_alias =
                                    StyleAlias::from(new_alias_id.into_static()).into_static();
                                let mut diagram = input_diagram.write();
                                if let Some(partials) =
                                    diagram.theme_default.style_aliases.get_mut(&parsed_key)
                                    && alias_idx < partials.style_aliases_applied.len() {
                                        partials.style_aliases_applied[alias_idx] = new_alias;
                                    }
                            }
                    }
                },
                onkeydown: move |evt| {
                    keyboard_nav::field_keydown(evt, DATA_ATTR);
                },
            }

            button {
                class: REMOVE_BTN,
                tabindex: "-1",
                "data-action": "remove",
                onclick: {
                    let key = alias_key.clone();
                    let alias_idx = alias_index;
                    move |_| {
                        if let Some(parsed_key) = parse_style_alias(&key) {
                            let mut diagram = input_diagram.write();
                            if let Some(partials) =
                                diagram.theme_default.style_aliases.get_mut(&parsed_key)
                                && alias_idx < partials.style_aliases_applied.len() {
                                    partials.style_aliases_applied.remove(alias_idx);
                                }
                        }
                    }
                },
                onkeydown: move |evt| {
                    keyboard_nav::field_keydown(evt, DATA_ATTR);
                },
                "x"
            }
        }
    }
}

// === StyleAliasesSectionAttrs === //

/// Renders the "Attributes" sub-section of a single style alias card.
///
/// Displays each `ThemeAttr -> value` pair as a row with a `<select>` for the
/// attribute name, a text `<input>` for the value, and a remove button. An
/// "+ Add attribute" button appends a new row with the first unused attribute.
#[component]
fn StyleAliasesSectionAttrs(
    input_diagram: Signal<InputDiagram<'static>>,
    alias_key: String,
    theme_attrs: Vec<ThemeAttrEntry>,
) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-1 pl-4",

            label {
                class: LABEL_CLASS,
                "Attributes"
            }

            for (attr_idx, theme_attr_entry) in theme_attrs.iter().enumerate() {
                {
                    let theme_attr = theme_attr_entry.theme_attr;
                    let attr_value = theme_attr_entry.attr_value.clone();
                    let key = alias_key.clone();
                    let attr_name = theme_attr_name(&theme_attr);
                    rsx! {
                        StyleAliasesSectionAttrRow {
                            key: "attr_{attr_idx}_{attr_name}",
                            input_diagram,
                            alias_key: key,
                            theme_attr,
                            attr_value,
                        }
                    }
                }
            }

            button {
                class: ADD_BTN,
                tabindex: -1,
                onclick: {
                    let key = alias_key.clone();
                    move |_| {
                        if let Some(parsed_key) = parse_style_alias(&key) {
                            let mut diagram = input_diagram.write();
                            if let Some(partials) =
                                diagram.theme_default.style_aliases.get_mut(&parsed_key)
                            {
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
                onkeydown: move |evt| {
                    keyboard_nav::field_keydown(evt, DATA_ATTR);
                },
                "+ Add attribute"
            }
        }
    }
}

// === StyleAliasesSectionAttrRow === //

/// A single row within the attributes sub-section.
///
/// Contains a `<select>` dropdown for the attribute name, a text `<input>`
/// for the attribute value, and a remove button.
#[component]
fn StyleAliasesSectionAttrRow(
    input_diagram: Signal<InputDiagram<'static>>,
    alias_key: String,
    theme_attr: ThemeAttr,
    attr_value: String,
) -> Element {
    let attr_name = theme_attr_name(&theme_attr);

    rsx! {
        div {
            class: ROW_CLASS_SIMPLE,

            // === Attribute name dropdown === //
            select {
                class: SELECT_CLASS,
                tabindex: "-1",
                value: "{attr_name}",
                onchange: {
                    let key = alias_key.clone();
                    let old_attr = theme_attr;
                    let current_value = attr_value.clone();
                    move |evt: dioxus::events::FormEvent| {
                        let new_attr_str = evt.value();
                        if let Some(new_attr) = parse_theme_attr(&new_attr_str)
                            && old_attr != new_attr
                            && let Some(parsed_key) = parse_style_alias(&key) {
                                let mut diagram = input_diagram.write();
                                if let Some(partials) =
                                    diagram.theme_default.style_aliases.get_mut(&parsed_key)
                                {
                                    partials.partials.shift_remove(&old_attr);
                                    partials
                                        .partials
                                        .insert(new_attr, current_value.clone());
                                }
                            }
                    }
                },
                onkeydown: move |evt| {
                    keyboard_nav::field_keydown(evt, DATA_ATTR);
                },

                for (name, _) in THEME_ATTRS.iter() {
                    option {
                        value: "{name}",
                        selected: *name == attr_name,
                        "{name}"
                    }
                }
            }

            // === Attribute value === //
            input {
                class: INPUT_CLASS,
                style: "max-width:8rem",
                tabindex: "-1",
                placeholder: "value",
                value: "{attr_value}",
                onchange: {
                    let key = alias_key.clone();
                    let attr = theme_attr;
                    move |evt: dioxus::events::FormEvent| {
                        let new_val = evt.value();
                        if let Some(parsed_key) = parse_style_alias(&key) {
                            let mut diagram = input_diagram.write();
                            if let Some(partials) =
                                diagram.theme_default.style_aliases.get_mut(&parsed_key)
                                && let Some(v) = partials.partials.get_mut(&attr) {
                                    *v = new_val;
                                }
                        }
                    }
                },
                onkeydown: move |evt| {
                    keyboard_nav::field_keydown(evt, DATA_ATTR);
                },
            }

            // === Remove button === //
            button {
                class: REMOVE_BTN,
                tabindex: "-1",
                "data-action": "remove",
                onclick: {
                    let key = alias_key.clone();
                    let attr = theme_attr;
                    move |_| {
                        if let Some(parsed_key) = parse_style_alias(&key) {
                            let mut diagram = input_diagram.write();
                            if let Some(partials) =
                                diagram.theme_default.style_aliases.get_mut(&parsed_key)
                            {
                                partials.partials.shift_remove(&attr);
                            }
                        }
                    }
                },
                onkeydown: move |evt| {
                    keyboard_nav::field_keydown(evt, DATA_ATTR);
                },
                "x"
            }
        }
    }
}
