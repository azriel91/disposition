//! Collapsed summary row for a [`TagThingsCard`].
//!
//! Extracted from [`TagThingsCard`] to keep the parent component concise.
//!
//! [`TagThingsCard`]: super::TagThingsCard

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};

use crate::components::editor::{reorderable::DragHandle, tags_page::COLLAPSED_HEADER_CLASS};

/// Collapsed summary for a tag-things card.
///
/// Displays the drag handle, expand chevron, tag ID, and thing count.
/// Clicking the row expands the card.
#[component]
pub(crate) fn TagThingsCardSummary(
    tag_id: String,
    thing_count: usize,
    mut collapsed: Signal<bool>,
) -> Element {
    let thing_suffix = if thing_count != 1 { "s" } else { "" };

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
                "{tag_id}"
            }

            span {
                class: "text-xs text-gray-500",
                "({thing_count} thing{thing_suffix})"
            }
        }
    }
}
