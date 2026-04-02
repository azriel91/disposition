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

pub(crate) mod dark_mode_editor;
pub(crate) mod style_aliases_section;
pub(crate) mod tag_focus_section;
pub(crate) mod types_styles_section;

use crate::components::editor::common::RenameRefocus;
use dioxus::{
    hooks::{use_context, use_signal},
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Memo, ReadableExt, Signal, WritableExt},
};
use disposition::input_model::{
    theme::{CssClassPartials, StyleAlias, TagIdOrDefaults, ThemeStyles},
    InputDiagram,
};
use disposition_input_ir_rt::{InputDiagramThemeSources, ThemeValueSource};

use crate::components::editor::theme_styles_editor::{
    css_class_partials_snapshot::CssClassPartialsSnapshot, theme_attr_entry::ThemeAttrEntry,
};

use crate::components::editor::{
    common::{
        parse_entity_type_id, parse_tag_id_or_defaults, ADD_BTN, CARD_CLASS, LABEL_CLASS,
        SECTION_HEADING, TEXTAREA_CLASS,
    },
    reorderable::ReorderableContainer,
    theme_styles_editor::{ThemeStylesEditor, ThemeStylesTarget},
};

use self::{
    dark_mode_editor::DarkModeEditor, style_aliases_section::StyleAliasesSection,
    tag_focus_section::TagFocusSection, types_styles_section::TypesStylesSection,
};

// === Style Aliases sub-page === //

/// The **Theme: Style Aliases** editor sub-page.
///
/// Edits `theme_default.style_aliases` -- a map from `StyleAlias` to
/// `CssClassPartials`. Each style alias entry gets its own card with
/// editable alias name, applied aliases, and theme attribute key-value pairs.
#[component]
pub fn ThemeStyleAliasesPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    // Drag-and-drop state for style alias cards.
    let style_alias_drag_idx: Signal<Option<usize>> = use_signal(|| None);
    let style_alias_drop_target: Signal<Option<usize>> = use_signal(|| None);
    // Focus-after-move state for style alias card reorder.
    let style_alias_focus_idx: Signal<Option<usize>> = use_signal(|| None);
    // Post-rename focus state for style alias cards.
    let style_alias_rename_refocus: Signal<Option<RenameRefocus>> = use_signal(|| None);

    let base_diagram: Memo<InputDiagram<'static>> = use_context();

    // Snapshot the entries so we can drop the borrow before event handlers.
    // Merge base + overlay style aliases so that base-only entries appear
    // as read-only in the editor.
    let entries: Vec<CssClassPartialsSnapshot> = {
        let base = base_diagram.read();
        let diagram = input_diagram.read();
        let sources = InputDiagramThemeSources::new(&base, &diagram);

        // Build the merged alias map: overlay entries first (preserving
        // their order), then base-only entries appended at the end.
        let overlay_aliases = &diagram.theme_default.style_aliases;
        let base_aliases = &base.theme_default.style_aliases;

        let mut seen_keys = Vec::new();
        let mut snapshots = Vec::new();

        // Overlay entries first.
        for (alias, css_partials) in overlay_aliases.iter() {
            let entry_key = alias.as_str().to_owned();
            seen_keys.push(alias.clone());
            let value_source = sources.style_alias_source(&entry_key);
            snapshots.push(style_alias_snapshot(&entry_key, css_partials, value_source));
        }

        // Base-only entries.
        for (alias, css_partials) in base_aliases.iter() {
            if !seen_keys.contains(alias) {
                let entry_key = alias.as_str().to_owned();
                snapshots.push(style_alias_snapshot(
                    &entry_key,
                    css_partials,
                    ThemeValueSource::BaseDiagram,
                ));
            }
        }

        snapshots
    };

    let entry_count = entries.len();

    rsx! {
        div {
            class: "flex flex-col gap-2",

            h3 { class: SECTION_HEADING, "Style Aliases" }
            p {
                class: LABEL_CLASS,
                "Style aliases group common CSS class partials under a single name. \
                 Each card below corresponds to one alias definition."
            }

            ReorderableContainer {
                data_attr: style_aliases_section::DATA_ATTR.to_owned(),
                section_id: "style_aliases".to_owned(),
                focus_index: style_alias_focus_idx,
                rename_refocus: Some(style_alias_rename_refocus),

                for (idx, entry) in entries.iter().enumerate() {
                    {
                        let alias_key = entry.entry_key.clone();
                        let style_aliases_applied = entry.style_aliases_applied.clone();
                        let theme_attrs = entry.theme_attrs.clone();
                        let value_source = entry.value_source;
                        rsx! {
                            StyleAliasesSection {
                                key: "alias_{idx}_{alias_key}",
                                input_diagram,
                                alias_key,
                                style_aliases_applied,
                                theme_attrs,
                                value_source,
                                index: idx,
                                entry_count,
                                drag_index: style_alias_drag_idx,
                                drop_target: style_alias_drop_target,
                                focus_index: style_alias_focus_idx,
                                rename_refocus: style_alias_rename_refocus,
                            }
                        }
                    }
                }
            }

            button {
                class: ADD_BTN,
                tabindex: 0,
                onclick: move |_| {
                    let mut diagram = input_diagram.write();
                    // Find a custom alias name that doesn't exist yet.
                    let mut n = 1u32;
                    let new_alias = loop {
                        let candidate = format!("custom_alias_{n}");
                        if let Ok(id) = disposition::model_common::Id::new(&candidate) {
                            let alias =
                                StyleAlias::from(id.into_static()).into_static();
                            if !diagram.theme_default.style_aliases.contains_key(&alias)
                            {
                                break alias;
                            }
                        }
                        n += 1;
                    };
                    diagram
                        .theme_default
                        .style_aliases
                        .insert(new_alias, CssClassPartials::default());
                },
                "+ Add style alias"
            }
        }
    }
}

/// Builds a [`CssClassPartialsSnapshot`] for a single style alias entry.
fn style_alias_snapshot(
    entry_key: &str,
    css_partials: &CssClassPartials<'static>,
    value_source: ThemeValueSource,
) -> CssClassPartialsSnapshot {
    let style_aliases_applied: Vec<String> = css_partials
        .style_aliases_applied
        .iter()
        .map(|a: &StyleAlias<'static>| a.as_str().to_owned())
        .collect();
    let theme_attrs: Vec<ThemeAttrEntry> = css_partials
        .partials
        .iter()
        .map(|(attr, val)| ThemeAttrEntry {
            theme_attr: *attr,
            attr_value: val.clone(),
        })
        .collect();
    CssClassPartialsSnapshot {
        entry_key: entry_key.to_owned(),
        style_aliases_applied,
        theme_attrs,
        value_source,
    }
}

// === Base Styles sub-page === //

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

            DarkModeEditor { input_diagram }
        }
    }
}

// === Process Step Selected Styles sub-page === //

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

// === Types Styles sub-page === //

/// The **Theme: Types Styles** editor sub-page.
///
/// Edits `theme_types_styles` -- a map from `EntityTypeId` to `ThemeStyles`.
///
/// Each entity type gets its own collapsible section containing a
/// [`ThemeStylesEditor`] that edits the inner `ThemeStyles` map.
#[component]
pub fn ThemeTypesStylesPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    let base_diagram: Memo<InputDiagram<'static>> = use_context();

    // Snapshot the outer keys, merging base + overlay so that base-only
    // entity types appear in the editor as read-only sections.
    let type_keys: Vec<String> = {
        let base = base_diagram.read();
        let diagram = input_diagram.read();

        // Overlay keys first (preserving order), then base-only keys.
        let mut keys = Vec::new();
        for k in diagram.theme_types_styles.keys() {
            keys.push(k.as_str().to_owned());
        }
        for k in base.theme_types_styles.keys() {
            let key_str = k.as_str().to_owned();
            if !keys.contains(&key_str) {
                keys.push(key_str);
            }
        }
        keys
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
                    let value_source = {
                        let base = base_diagram.read();
                        let diagram = input_diagram.read();
                        let sources = InputDiagramThemeSources::new(&base, &diagram);
                        sources.types_styles_key_source(&type_key)
                    };
                    rsx! {
                        TypesStylesSection {
                            key: "type_{type_key}",
                            input_diagram,
                            type_key,
                            value_source,
                        }
                    }
                }
            }

            button {
                class: ADD_BTN,
                tabindex: 0,
                onclick: move |_| {
                    let mut diagram = input_diagram.write();
                    // Find a type key that doesn't exist yet.
                    let mut n = 1u32;
                    let new_key = loop {
                        let candidate = format!("type_custom_{n}");
                        if let Some(type_id) = parse_entity_type_id(&candidate)
                            && !diagram.theme_types_styles.contains_key(&type_id) {
                                break type_id;
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

// === Thing Dependencies Styles sub-page === //

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

            // === things_included_styles === //
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

            // === things_excluded_styles === //
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

// === Tag Things Focus Styles sub-page === //

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
    let base_diagram: Memo<InputDiagram<'static>> = use_context();

    // Snapshot the outer keys, merging base + overlay so that base-only
    // tag entries appear in the editor as read-only sections.
    let tag_keys: Vec<String> = {
        let base = base_diagram.read();
        let diagram = input_diagram.read();

        // Overlay keys first (preserving order), then base-only keys.
        let mut keys = Vec::new();
        for k in diagram.theme_tag_things_focus.keys() {
            keys.push(k.as_str().to_owned());
        }
        for k in base.theme_tag_things_focus.keys() {
            let key_str = k.as_str().to_owned();
            if !keys.contains(&key_str) {
                keys.push(key_str);
            }
        }
        keys
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
                    let value_source = {
                        let base = base_diagram.read();
                        let diagram = input_diagram.read();
                        let sources = InputDiagramThemeSources::new(&base, &diagram);
                        sources.tag_focus_key_source(&tag_key)
                    };
                    rsx! {
                        TagFocusSection {
                            key: "tag_{tag_key}",
                            input_diagram,
                            tag_key,
                            value_source,
                        }
                    }
                }
            }

            button {
                class: ADD_BTN,
                tabindex: 0,
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
                            if let Some(tag_key) = parse_tag_id_or_defaults(&candidate)
                                && !diagram.theme_tag_things_focus.contains_key(&tag_key) {
                                    diagram
                                        .theme_tag_things_focus
                                        .insert(tag_key, ThemeStyles::default());
                                    break;
                                }
                            n += 1;
                        }
                    }
                },
                "+ Add tag entry"
            }

            // === Additional CSS === //
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
                        onchange: move |evt| {
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
