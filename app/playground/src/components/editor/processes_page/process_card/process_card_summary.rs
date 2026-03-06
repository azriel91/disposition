//! Collapsed summary row for a [`ProcessCard`].
//!
//! Extracted from [`ProcessCard`] to keep the parent component concise.
//!
//! [`ProcessCard`]: super::ProcessCard

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};

use crate::components::editor::{processes_page::COLLAPSED_HEADER_CLASS, reorderable::DragHandle};

/// Collapsed summary for a process card.
///
/// Displays the drag handle, expand chevron, process ID, optional display
/// name, and step count. Clicking the row expands the card.
#[component]
pub(crate) fn ProcessCardSummary(
    process_id: String,
    display_name: String,
    step_count: usize,
    mut collapsed: Signal<bool>,
) -> Element {
    let step_suffix = if step_count != 1 { "s" } else { "" };
    let has_name = !display_name.is_empty();

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
                "{process_id}"
            }

            if has_name {
                span {
                    class: "text-sm text-gray-300",
                    "-- {display_name}"
                }
            }

            span {
                class: "text-xs text-gray-500",
                "({step_count} step{step_suffix})"
            }
        }
    }
}
