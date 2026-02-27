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
use disposition::input_model::InputDiagram;

use crate::components::editor::common::{LABEL_CLASS, SECTION_HEADING, TEXTAREA_CLASS};

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
/// Presented as a YAML editor with datalist hints for IDs.
#[component]
pub fn ThemeBaseStylesPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    let yaml = {
        let input_diagram = input_diagram.read();
        serde_saphyr::to_string(&input_diagram.theme_default.base_styles)
            .unwrap_or_default()
            .trim()
            .to_owned()
    };

    rsx! {
        div {
            class: "flex flex-col gap-2",

            h3 { class: SECTION_HEADING, "Base Styles" }
            p {
                class: LABEL_CLASS,
                "Default styles for entities when there is no user interaction. \
                 Keys can be 'node_defaults', 'node_excluded_defaults', 'edge_defaults', or a specific entity ID. \
                 Edit as YAML."
            }

            textarea {
                class: TEXTAREA_CLASS,
                value: "{yaml}",
                oninput: move |evt| {
                    let text = evt.value();
                    if let Ok(styles) = serde_saphyr::from_str(&text) {
                        input_diagram.write().theme_default.base_styles = styles;
                    }
                },
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
#[component]
pub fn ThemeProcessStepStylesPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    let yaml = {
        let input_diagram = input_diagram.read();
        serde_saphyr::to_string(&input_diagram.theme_default.process_step_selected_styles)
            .unwrap_or_default()
            .trim()
            .to_owned()
    };

    rsx! {
        div {
            class: "flex flex-col gap-2",

            h3 { class: SECTION_HEADING, "Process Step Selected Styles" }
            p {
                class: LABEL_CLASS,
                "Styles applied to entities when a process step is selected/focused. \
                 Edit as YAML."
            }

            textarea {
                class: TEXTAREA_CLASS,
                value: "{yaml}",
                oninput: move |evt| {
                    let text = evt.value();
                    if let Ok(styles) = serde_saphyr::from_str(&text) {
                        input_diagram.write().theme_default.process_step_selected_styles = styles;
                    }
                },
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
#[component]
pub fn ThemeTypesStylesPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    let yaml = {
        let input_diagram = input_diagram.read();
        serde_saphyr::to_string(&input_diagram.theme_types_styles)
            .unwrap_or_default()
            .trim()
            .to_owned()
    };

    rsx! {
        div {
            class: "flex flex-col gap-2",

            h3 { class: SECTION_HEADING, "Type-Based Styles" }
            p {
                class: LABEL_CLASS,
                "Styles applied to entities with a particular 'type'. \
                 Keys are entity type IDs. Edit as YAML."
            }

            textarea {
                class: TEXTAREA_CLASS,
                value: "{yaml}",
                oninput: move |evt| {
                    let text = evt.value();
                    if let Ok(styles) = serde_saphyr::from_str(&text) {
                        input_diagram.write().theme_types_styles = styles;
                    }
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
#[component]
pub fn ThemeDependenciesStylesPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    let yaml = {
        let input_diagram = input_diagram.read();
        serde_saphyr::to_string(&input_diagram.theme_thing_dependencies_styles)
            .unwrap_or_default()
            .trim()
            .to_owned()
    };

    rsx! {
        div {
            class: "flex flex-col gap-2",

            h3 { class: SECTION_HEADING, "Thing Dependencies Focus Styles" }
            p {
                class: LABEL_CLASS,
                "Styles when a thing is focused to show its dependencies. \
                 Contains 'things_included_styles' and 'things_excluded_styles'. \
                 Edit as YAML."
            }

            textarea {
                class: TEXTAREA_CLASS,
                value: "{yaml}",
                oninput: move |evt| {
                    let text = evt.value();
                    if let Ok(styles) = serde_saphyr::from_str(&text) {
                        input_diagram.write().theme_thing_dependencies_styles = styles;
                    }
                },
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
#[component]
pub fn ThemeTagsFocusPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    let yaml = {
        let input_diagram = input_diagram.read();
        serde_saphyr::to_string(&input_diagram.theme_tag_things_focus)
            .unwrap_or_default()
            .trim()
            .to_owned()
    };

    rsx! {
        div {
            class: "flex flex-col gap-2",

            h3 { class: SECTION_HEADING, "Tag Focus Styles" }
            p {
                class: LABEL_CLASS,
                "Styles when a tag is focused. \
                 'tag_defaults' applies to all tags; specific tag IDs override. \
                 Edit as YAML."
            }

            textarea {
                class: TEXTAREA_CLASS,
                value: "{yaml}",
                oninput: move |evt| {
                    let text = evt.value();
                    if let Ok(styles) = serde_saphyr::from_str(&text) {
                        input_diagram.write().theme_tag_things_focus = styles;
                    }
                },
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
