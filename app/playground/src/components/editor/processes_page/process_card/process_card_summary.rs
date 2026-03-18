//! Collapsed summary row for a [`ProcessCard`].
//!
//! Extracted from [`ProcessCard`] to keep the parent component concise.
//!
//! [`ProcessCard`]: super::ProcessCard

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::ProcessesPageOps;

use crate::components::editor::{
    common::REMOVE_BTN, processes_page::COLLAPSED_HEADER_CLASS, reorderable::DragHandle,
};

/// Collapsed summary for a process card.
///
/// Displays the drag handle, expand chevron, process ID, optional display
/// name, step count, and a remove button. Clicking the row (except the
/// remove button) expands the card.
#[component]
pub(crate) fn ProcessCardSummary(
    input_diagram: Signal<InputDiagram<'static>>,
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

            // === Remove button === //
            button {
                class: REMOVE_BTN,
                tabindex: "0",
                "data-action": "remove",
                onclick: {
                    let process_id = process_id.clone();
                    move |evt: dioxus::events::MouseEvent| {
                        evt.stop_propagation();
                        ProcessesPageOps::process_remove(&mut input_diagram.write(), &process_id);
                    }
                },
                "\u{2715}"
            }
        }
    }
}
