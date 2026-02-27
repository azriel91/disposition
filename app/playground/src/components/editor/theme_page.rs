//! Theme editor page.
//!
//! Provides sub-pages for:
//! - Style Aliases (`theme_default.style_aliases`)
//! - Base Styles (`theme_default.base_styles`)
//! - Process Step Selected Styles
//!   (`theme_default.process_step_selected_styles`)
//! - Type-based Styles (`theme_types_styles`)
//! - Thing-dependencies focus styles (`theme_thing_dependencies_styles`)
//! - Tag-things focus styles (`theme_tag_things_focus`)
//! - Additional CSS (`css`)

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::{
    input_model::{
        theme::{TagIdOrDefaults, ThemeStyles},
        InputDiagram,
    },
    model_common::{entity::EntityTypeId, Id},
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse a string into an `EntityTypeId<'static>`.
fn parse_entity_type_id(s: &str) -> Option<EntityTypeId<'static>> {
    Id::new(s)
        .ok()
        .map(|id| EntityTypeId::from(id.into_static()))
}

use crate::components::editor::{
    common::{
        ADD_BTN, CARD_CLASS, INPUT_CLASS, LABEL_CLASS, REMOVE_BTN, ROW_CLASS_SIMPLE,
        SECTION_HEADING, TEXTAREA_CLASS,
    },
    datalists::list_ids,
    theme_styles_editor::{ThemeStylesEditor, ThemeStylesTarget},
};

// ===========================================================================
// Style Aliases sub-page
// ===========================================================================

/// The **Theme: Style Aliases** editor sub-page.
///
/// Edits `theme_default.style_aliases` -- a map from `StyleAlias` to
/// `CssClassPartials`. Because the `CssClassPartials` structure is complex
/// (containing `style_aliases_applied` and a flat map of `ThemeAttr` to value),
/// we present each alias entry as an editable YAML snippet for now, with the
/// alias name as a text input.
#[component]
pub fn ThemeStyleAliasesPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    let yaml = {
        let input_diagram = input_diagram.read();
        serde_saphyr::to_string(&input_diagram.theme_default.style_aliases)
            .unwrap_or_default()
            .trim()
            .to_owned()
    };

    rsx! {
        div {
            class: "flex flex-col gap-2",

            h3 { class: SECTION_HEADING, "Style Aliases" }
            p {
                class: LABEL_CLASS,
                "Style aliases group common CSS class partials under a single name. \
                 Edit as YAML."
            }

            textarea {
                class: TEXTAREA_CLASS,
                value: "{yaml}",
                oninput: move |evt| {
                    let text = evt.value();
                    if let Ok(aliases) = serde_saphyr::from_str(&text) {
                        input_diagram.write().theme_default.style_aliases = aliases;
                    }
                },
            }
        }
    }
}

// ===========================================================================
// Base Styles sub-page
// ===========================================================================

/// The **Theme: Base Styles** editor sub-page.
///
/// Edits `theme_default.base_styles` -- a map from `IdOrDefaults` to
/// `CssClassPartials`. Entries can be `node_defaults`,
/// `node_excluded_defaults`, `edge_defaults`, or specific entity IDs.
///
/// Uses the card-based [`ThemeStylesEditor`] component for rich editing.
#[component]
pub fn ThemeBaseStylesPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-2",

            h3 { class: SECTION_HEADING, "Base Styles" }
            p {
                class: LABEL_CLASS,
                "Default styles for entities when there is no user interaction. \
                 Each card configures a 'node_defaults', 'edge_defaults', or specific entity ID."
            }

            ThemeStylesEditor {
                input_diagram,
                target: ThemeStylesTarget::BaseStyles,
            }
        }
    }
}

// ===========================================================================
// Process Step Selected Styles sub-page
// ===========================================================================

/// The **Theme: Process Step Styles** editor sub-page.
///
/// Edits `theme_default.process_step_selected_styles`.
///
/// Uses the card-based [`ThemeStylesEditor`] component for rich editing.
#[component]
pub fn ThemeProcessStepStylesPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-2",

            h3 { class: SECTION_HEADING, "Process Step Selected Styles" }
            p {
                class: LABEL_CLASS,
                "Styles applied to entities when a process step is selected/focused. \
                 Each card configures a 'node_defaults', 'edge_defaults', or specific entity ID."
            }

            ThemeStylesEditor {
                input_diagram,
                target: ThemeStylesTarget::ProcessStepSelectedStyles,
            }
        }
    }
}

// ===========================================================================
// Types Styles sub-page
// ===========================================================================

/// The **Theme: Types Styles** editor sub-page.
///
/// Edits `theme_types_styles` -- a map from `EntityTypeId` to `ThemeStyles`.
///
/// Each entity type gets its own collapsible section containing a
/// [`ThemeStylesEditor`] that edits the inner `ThemeStyles` map.
#[component]
pub fn ThemeTypesStylesPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    // Snapshot the outer keys so we can drop the borrow before event handlers.
    let type_keys: Vec<String> = {
        let diagram = input_diagram.read();
        diagram
            .theme_types_styles
            .keys()
            .map(|k| k.as_str().to_owned())
            .collect()
    };

    rsx! {
        div {
            class: "flex flex-col gap-2",

            h3 { class: SECTION_HEADING, "Type-Based Styles" }
            p {
                class: LABEL_CLASS,
                "Styles applied to entities with a particular 'type'. \
                 Each section below corresponds to an entity type ID. \
                 Within each section you can configure node_defaults, edge_defaults, \
                 or specific entity styles."
            }

            for type_key in type_keys.iter() {
                {
                    let type_key = type_key.clone();
                    rsx! {
                        TypesStylesSection {
                            key: "type_{type_key}",
                            input_diagram,
                            type_key,
                        }
                    }
                }
            }

            div {
                class: ADD_BTN,
                onclick: move |_| {
                    let mut diagram = input_diagram.write();
                    // Find a type key that doesn't exist yet.
                    let mut n = 1u32;
                    let new_key = loop {
                        let candidate = format!("type_custom_{n}");
                        if let Some(type_id) = parse_entity_type_id(&candidate) {
                            if !diagram.theme_types_styles.contains_key(&type_id) {
                                break type_id;
                            }
                        }
                        n += 1;
                    };
                    diagram
                        .theme_types_styles
                        .insert(new_key, ThemeStyles::default());
                },
                "+ Add entity type"
            }
        }
    }
}

/// A single entity-type section within [`ThemeTypesStylesPage`].
///
/// Shows a header with the type key (editable) and a remove button, then
/// embeds a [`ThemeStylesEditor`] targeting that specific type key.
#[component]
fn TypesStylesSection(input_diagram: Signal<InputDiagram<'static>>, type_key: String) -> Element {
    rsx! {
        div {
            class: CARD_CLASS,

            // ── Header: type key + remove ────────────────────────────
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
                    "✕ Remove type"
                }
            }

            // ── Inner ThemeStyles editor ─────────────────────────────
            ThemeStylesEditor {
                input_diagram,
                target: ThemeStylesTarget::TypesStyles {
                    entity_type_key: type_key.clone(),
                },
            }
        }
    }
}

// ===========================================================================
// Thing Dependencies Styles sub-page
// ===========================================================================

/// The **Theme: Dependencies Styles** editor sub-page.
///
/// Edits `theme_thing_dependencies_styles` which has two sub-fields:
/// - `things_included_styles`
/// - `things_excluded_styles`
///
/// Each field is a [`ThemeStyles`] map and gets its own [`ThemeStylesEditor`].
#[component]
pub fn ThemeDependenciesStylesPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-4",

            h3 { class: SECTION_HEADING, "Thing Dependencies Focus Styles" }
            p {
                class: LABEL_CLASS,
                "Styles when a thing is focused to show its dependencies. \
                 Configure separate styles for included and excluded things."
            }

            // ── things_included_styles ───────────────────────────────
            div {
                class: CARD_CLASS,

                h4 {
                    class: "text-sm font-semibold text-gray-400",
                    "Included Things Styles"
                }
                p {
                    class: LABEL_CLASS,
                    "Styles applied to things that are part of the focused dependency chain."
                }

                ThemeStylesEditor {
                    input_diagram,
                    target: ThemeStylesTarget::DependenciesIncluded,
                }
            }

            // ── things_excluded_styles ───────────────────────────────
            div {
                class: CARD_CLASS,

                h4 {
                    class: "text-sm font-semibold text-gray-400",
                    "Excluded Things Styles"
                }
                p {
                    class: LABEL_CLASS,
                    "Styles applied to things that are NOT part of the focused dependency chain."
                }

                ThemeStylesEditor {
                    input_diagram,
                    target: ThemeStylesTarget::DependenciesExcluded,
                }
            }
        }
    }
}

// ===========================================================================
// Tag Things Focus Styles sub-page
// ===========================================================================

/// The **Theme: Tags Focus** editor sub-page.
///
/// Edits `theme_tag_things_focus` -- a map from `TagIdOrDefaults` to
/// `ThemeStyles`. `tag_defaults` applies to all tags; specific tag IDs
/// override.
///
/// Each tag key gets its own collapsible section containing a
/// [`ThemeStylesEditor`].
#[component]
pub fn ThemeTagsFocusPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    // Snapshot the outer keys.
    let tag_keys: Vec<String> = {
        let diagram = input_diagram.read();
        diagram
            .theme_tag_things_focus
            .keys()
            .map(|k| k.as_str().to_owned())
            .collect()
    };

    rsx! {
        div {
            class: "flex flex-col gap-2",

            h3 { class: SECTION_HEADING, "Tag Focus Styles" }
            p {
                class: LABEL_CLASS,
                "Styles when a tag is focused. \
                 'tag_defaults' applies to all tags; specific tag IDs override."
            }

            for tag_key in tag_keys.iter() {
                {
                    let tag_key = tag_key.clone();
                    rsx! {
                        TagFocusSection {
                            key: "tag_{tag_key}",
                            input_diagram,
                            tag_key,
                        }
                    }
                }
            }

            div {
                class: ADD_BTN,
                onclick: move |_| {
                    let mut diagram = input_diagram.write();
                    // Add tag_defaults first if not present, otherwise a custom tag.
                    let tag_defaults_key = TagIdOrDefaults::TagDefaults;
                    if !diagram.theme_tag_things_focus.contains_key(&tag_defaults_key) {
                        diagram
                            .theme_tag_things_focus
                            .insert(tag_defaults_key, ThemeStyles::default());
                    } else {
                        let mut n = 1u32;
                        loop {
                            let candidate = format!("tag_custom_{n}");
                            if let Ok(id) = Id::new(&candidate) {
                                use disposition::input_model::tag::TagId;
                                let tag_key =
                                    TagIdOrDefaults::Custom(TagId::from(id.into_static()));
                                if !diagram.theme_tag_things_focus.contains_key(&tag_key) {
                                    diagram
                                        .theme_tag_things_focus
                                        .insert(tag_key, ThemeStyles::default());
                                    break;
                                }
                            }
                            n += 1;
                        }
                    }
                },
                "+ Add tag entry"
            }

            // ── Additional CSS ───────────────────────────────────────
            h3 { class: SECTION_HEADING, "Additional CSS" }
            p {
                class: LABEL_CLASS,
                "Extra CSS to include in the SVG's inline <style> section."
            }

            {
                let css_yaml = {
                    let input_diagram = input_diagram.read();
                    serde_saphyr::to_string(&input_diagram.css)
                        .unwrap_or_default()
                        .trim()
                        .to_owned()
                };

                rsx! {
                    textarea {
                        class: TEXTAREA_CLASS,
                        value: "{css_yaml}",
                        oninput: move |evt| {
                            let text = evt.value();
                            if let Ok(css) = serde_saphyr::from_str(&text) {
                                input_diagram.write().css = css;
                            }
                        },
                    }
                }
            }
        }
    }
}

/// A single tag-focus section within [`ThemeTagsFocusPage`].
///
/// Shows a header with the tag key (select for `tag_defaults`, text input for
/// custom tags) and a remove button, then embeds a [`ThemeStylesEditor`]
/// targeting that specific tag key.
#[component]
fn TagFocusSection(input_diagram: Signal<InputDiagram<'static>>, tag_key: String) -> Element {
    let is_defaults = tag_key == "tag_defaults";

    rsx! {
        div {
            class: CARD_CLASS,

            // ── Header: tag key + remove ─────────────────────────────
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
                                if new_val != old_key {
                                    if let (Some(old_tag), Some(new_tag)) = (
                                        parse_tag_key(&old_key),
                                        parse_tag_key(&new_val),
                                    ) {
                                        let mut diagram = input_diagram.write();
                                        if !diagram.theme_tag_things_focus.contains_key(&new_tag)
                                        {
                                            if let Some(idx) = diagram
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
                                }
                            }
                        },
                    }
                }

                span {
                    class: REMOVE_BTN,
                    onclick: {
                        let key = tag_key.clone();
                        move |_| {
                            if let Some(parsed) = parse_tag_key(&key) {
                                let mut diagram = input_diagram.write();
                                diagram.theme_tag_things_focus.shift_remove(&parsed);
                            }
                        }
                    },
                    "✕ Remove tag"
                }
            }

            // ── Inner ThemeStyles editor ─────────────────────────────
            ThemeStylesEditor {
                input_diagram,
                target: ThemeStylesTarget::TagFocus {
                    tag_key: tag_key.clone(),
                },
            }
        }
    }
}

/// Parse a string into a `TagIdOrDefaults`.
fn parse_tag_key(s: &str) -> Option<TagIdOrDefaults<'static>> {
    use disposition::input_model::tag::TagId;
    match s {
        "tag_defaults" => Some(TagIdOrDefaults::TagDefaults),
        other => Id::new(other)
            .ok()
            .map(|id| TagIdOrDefaults::Custom(TagId::from(id.into_static()))),
    }
}
