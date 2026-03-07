//! Collapsed summary view for an edge group card.
//!
//! Displays a single-line overview with the edge group ID, kind label,
//! thing count, and a remove button. Clicking the summary (outside the
//! remove button) expands the card.
//!
//! [`EdgeGroupCard`]: super::EdgeGroupCard

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::{EdgeGroupCardOps, MapTarget};

use crate::components::editor::{
    common::REMOVE_BTN, reorderable::DragHandle, thing_dependencies_page::COLLAPSED_HEADER_CLASS,
};

/// Collapsed summary view for an edge group card.
///
/// Shows the drag handle, expand chevron, edge group ID, kind label,
/// thing count, and a remove button. Clicking anywhere on the row
/// (except the remove button) expands the card.
#[component]
pub(crate) fn EdgeGroupCardSummary(
    input_diagram: Signal<InputDiagram<'static>>,
    target: MapTarget,
    edge_group_id: String,
    edge_kind_label: String,
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
                "{edge_group_id}"
            }

            span {
                class: "text-xs text-gray-500 italic",
                "{edge_kind_label}"
            }

            span {
                class: "text-xs text-gray-500",
                "({thing_count} thing{thing_suffix})"
            }

            // === Remove button === //
            button {
                class: REMOVE_BTN,
                tabindex: "0",
                "data-action": "remove",
                onclick: {
                    let edge_group_id = edge_group_id.clone();
                    move |evt: dioxus::events::MouseEvent| {
                        evt.stop_propagation();
                        EdgeGroupCardOps::edge_group_remove(
                            &mut input_diagram.write(),
                            target,
                            &edge_group_id,
                        );
                    }
                },
                "\u{2715}"
            }
        }
    }
}
