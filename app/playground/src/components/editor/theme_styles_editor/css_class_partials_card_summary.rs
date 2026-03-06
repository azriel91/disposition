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

use crate::components::editor::reorderable::DragHandle;

use super::css_class_partials_card::COLLAPSED_HEADER_CLASS;

/// Collapsed summary for a CSS class partials card.
///
/// Displays the drag handle, expand chevron, entry key, and counts of
/// aliases and attributes. Clicking the row expands the card.
#[component]
pub fn CssClassPartialsCardSummary(
    entry_key: String,
    alias_count: usize,
    attr_count: usize,
    mut collapsed: Signal<bool>,
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
        }
    }
}
