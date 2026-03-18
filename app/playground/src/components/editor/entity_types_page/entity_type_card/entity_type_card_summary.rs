//! Collapsed summary row for an [`EntityTypeCard`].
//!
//! Extracted from [`EntityTypeCard`] to keep the parent component concise.
//!
//! [`EntityTypeCard`]: super::EntityTypeCard

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::EntityTypesPageOps;

use crate::components::editor::{
    common::REMOVE_BTN, entity_types_page::COLLAPSED_HEADER_CLASS, reorderable::DragHandle,
};

/// Collapsed summary for an entity type card.
///
/// Displays the drag handle, expand chevron, entity ID, type count, and a
/// remove button. Clicking the row (except the remove button) expands
/// the card.
#[component]
pub(crate) fn EntityTypeCardSummary(
    input_diagram: Signal<InputDiagram<'static>>,
    entity_id: String,
    type_count: usize,
    mut collapsed: Signal<bool>,
) -> Element {
    let type_suffix = if type_count != 1 { "s" } else { "" };

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
                "{entity_id}"
            }

            span {
                class: "text-xs text-gray-500",
                "({type_count} type{type_suffix})"
            }

            // === Remove button === //
            button {
                class: REMOVE_BTN,
                tabindex: "0",
                "data-action": "remove",
                onclick: {
                    let entity_id = entity_id.clone();
                    move |evt: dioxus::events::MouseEvent| {
                        evt.stop_propagation();
                        EntityTypesPageOps::entry_remove(&mut input_diagram.write(), &entity_id);
                    }
                },
                "\u{2715}"
            }
        }
    }
}
