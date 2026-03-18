//! Collapsed summary row for a [`CssClassPartialsCard`].
//!
//! Extracted from [`CssClassPartialsCard`] to keep the parent component
//! concise.
//!
//! [`CssClassPartialsCard`]: super::css_class_partials_card::CssClassPartialsCard

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_ir_rt::ThemeValueSource;

use crate::components::editor::{common::REMOVE_BTN, reorderable::DragHandle};

use super::{
    css_class_partials_card::COLLAPSED_HEADER_CLASS, parse_id_or_defaults, ThemeStylesTarget,
};

/// Collapsed summary for a CSS class partials card.
///
/// Displays the drag handle, expand chevron, entry key, counts of
/// aliases and attributes, and a remove button. Clicking the row
/// (except the remove button) expands the card.
#[component]
pub fn CssClassPartialsCardSummary(
    input_diagram: Signal<InputDiagram<'static>>,
    target: ThemeStylesTarget,
    entry_key: String,
    alias_count: usize,
    attr_count: usize,
    mut collapsed: Signal<bool>,
    value_source: ThemeValueSource,
) -> Element {
    let alias_suffix = if alias_count != 1 { "es" } else { "" };
    let attr_suffix = if attr_count != 1 { "s" } else { "" };

    rsx! {
        div {
            class: COLLAPSED_HEADER_CLASS,
            onclick: move |_| collapsed.set(false),

            DragHandle {}

            // Expand chevron
            span {
                class: "text-gray-500 text-xs",
                ">"
            }

            span {
                class: "text-sm font-mono text-blue-400",
                "{entry_key}"
            }

            span {
                class: "text-xs text-gray-500",
                "({alias_count} alias{alias_suffix}, {attr_count} attr{attr_suffix})"
            }

            // Value source indicator
            if value_source == ThemeValueSource::BaseDiagram {
                span {
                    class: "text-xs text-gray-500 italic",
                    "(base)"
                }
            } else {
                span {
                    class: "text-xs text-amber-400",
                    "(override)"
                }
            }

            // === Remove button === //
            button {
                class: REMOVE_BTN,
                tabindex: "0",
                "data-action": "remove",
                onclick: {
                    let entry_key = entry_key.clone();
                    let target = target.clone();
                    move |evt: dioxus::events::MouseEvent| {
                        evt.stop_propagation();
                        if let Some(parsed) = parse_id_or_defaults(&entry_key) {
                            let mut diagram = input_diagram.write();
                            if let Some(styles) = target.write_mut(&mut diagram) {
                                styles.remove(&parsed);
                            }
                        }
                    }
                },
                "\u{2715}"
            }
        }
    }
}
