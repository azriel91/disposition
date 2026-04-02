//! Dark mode shade configuration editor.
//!
//! Provides radio buttons to select between `Disable`, `Invert`, and
//! `Shift` variants of `DarkModeShadeConfig`, with a slider for the
//! shift levels when `Shift` is selected.

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::{theme::DarkModeShadeConfig, InputDiagram};

use crate::components::editor::common::{LABEL_CLASS, SECTION_HEADING};

// === DarkModeEditor === //

/// Editor for `theme_default.dark_mode_shade_config`.
///
/// Displays radio buttons for `Disable`, `Invert`, and `Shift` variants,
/// plus a levels slider (1--10, default 5) when `Shift` is selected.
#[component]
pub fn DarkModeEditor(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    let config = input_diagram.read().theme_default.dark_mode_shade_config;

    let is_disable = matches!(config, DarkModeShadeConfig::Disable);
    let is_invert = matches!(config, DarkModeShadeConfig::Invert);
    let is_shift = matches!(config, DarkModeShadeConfig::Shift { .. });
    let shift_levels = match config {
        DarkModeShadeConfig::Shift { levels } => levels,
        _ => 5,
    };

    rsx! {
        div {
            class: "flex flex-col gap-2",

            h3 { class: SECTION_HEADING, "Dark Mode" }
            p {
                class: LABEL_CLASS,
                "Controls how shades are adjusted for dark mode. \
                 'Disable' emits no dark-mode classes, 'Invert' mirrors shades around 500, \
                 and 'Shift' offsets shades by a number of levels."
            }

            // Radio group
            div {
                class: "flex flex-col gap-1",

                // Disable
                div {
                    class: "flex flex-col gap-0.5",
                    label {
                        class: "flex items-center gap-2 text-sm text-gray-300 cursor-pointer",
                        "data-input-diagram-field": "theme_dark_mode_disable",
                        input {
                            r#type: "radio",
                            name: "dark_mode_shade_config",
                            value: "disable",
                            checked: is_disable,
                            "data-input-diagram-field": "theme_dark_mode_disable",
                            onchange: move |_| {
                                input_diagram.write().theme_default.dark_mode_shade_config =
                                    DarkModeShadeConfig::Disable;
                            },
                        }
                        "Disable"
                    }
                    p {
                        class: "text-xs text-gray-500 pl-6",
                        "No dark-mode classes are emitted. The diagram uses the same shades in both light and dark mode."
                    }
                }

                // Invert
                div {
                    class: "flex flex-col gap-0.5",
                    label {
                        class: "flex items-center gap-2 text-sm text-gray-300 cursor-pointer",
                        "data-input-diagram-field": "theme_dark_mode_invert",
                        input {
                            r#type: "radio",
                            name: "dark_mode_shade_config",
                            value: "invert",
                            checked: is_invert,
                            "data-input-diagram-field": "theme_dark_mode_invert",
                            onchange: move |_| {
                                input_diagram.write().theme_default.dark_mode_shade_config =
                                    DarkModeShadeConfig::Invert;
                            },
                        }
                        "Invert"
                    }
                    p {
                        class: "text-xs text-gray-500 pl-6",
                        "Shades are mirrored around 500. For example, shade 100 becomes 900 and 200 becomes 800."
                    }
                }

                // Shift
                div {
                    class: "flex flex-col gap-0.5",
                    label {
                        class: "flex items-center gap-2 text-sm text-gray-300 cursor-pointer",
                        "data-input-diagram-field": "theme_dark_mode_shift",
                        input {
                            r#type: "radio",
                            name: "dark_mode_shade_config",
                            value: "shift",
                            checked: is_shift,
                            "data-input-diagram-field": "theme_dark_mode_shift",
                            onchange: move |_| {
                                input_diagram.write().theme_default.dark_mode_shade_config =
                                    DarkModeShadeConfig::Shift { levels: shift_levels };
                            },
                        }
                        "Shift"
                    }
                    p {
                        class: "text-xs text-gray-500 pl-6",
                        "Shades are offset by a number of levels. For example, with levels 4, shade 100 becomes 500."
                    }
                }
            }

            // Shift levels slider (only shown when Shift is selected)
            if is_shift {
                div {
                    class: "flex items-center gap-2 mt-1 pl-6",

                    label {
                        class: "text-xs text-gray-400",
                        r#for: "dark_mode_shift_levels",
                        "Levels: {shift_levels}"
                    }

                    input {
                        r#type: "range",
                        id: "dark_mode_shift_levels",
                        name: "dark_mode_shift_levels",
                        min: "1",
                        max: "10",
                        value: "{shift_levels}",
                        class: "w-32 accent-blue-500",
                        "data-input-diagram-field": "theme_dark_mode_shift_levels",
                        onchange: move |evt: dioxus::events::FormEvent| {
                            if let Ok(levels) = evt.value().parse::<u8>() {
                                input_diagram.write().theme_default.dark_mode_shade_config =
                                    DarkModeShadeConfig::Shift { levels };
                            }
                        },
                    }
                }
            }
        }
    }
}
