//! Collapsible card component for a single entity's type set.
//!
//! Displays the entity ID, a remove button, and a list of entity types
//! that can be individually edited, removed, reordered, or added to.
//!
//! Supports keyboard shortcuts:
//!
//! - **ArrowUp / ArrowDown**: navigate between sibling cards.
//! - **Alt+Up / Alt+Down**: move the card up or down in the list.
//! - **ArrowRight**: expand the card (when collapsed).
//! - **ArrowLeft**: collapse the card (when expanded).
//! - **Space**: toggle expand/collapse.
//! - **Enter**: expand + focus the first input inside the card.
//! - **Ctrl+Shift+K**: remove the card.
//! - **Escape**: focus the parent section / tab.
//! - **Tab / Shift+Tab** (inside a field): cycle through focusable fields
//!   within the card. Wraps from last to first / first to last.
//! - **Esc** (inside a field): return focus to the card wrapper.

mod entity_type_card_field_id;
mod entity_type_card_field_types;
mod entity_type_card_field_types_row;
mod entity_type_card_summary;

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::EntityTypesPageOps;

use crate::components::editor::{
    common::{CardComponent, RenameRefocus},
    entity_types_page::{EntityTypeEntry, DATA_ATTR, ENTITY_TYPE_CARD_CLASS},
    reorderable::{drag_border_class, DragHandle},
};

use self::{
    entity_type_card_field_id::EntityTypeCardFieldId,
    entity_type_card_field_types::EntityTypeCardFieldTypes,
    entity_type_card_summary::EntityTypeCardSummary,
};

/// A collapsible card for editing a single entity's type set.
///
/// When collapsed, shows the entity ID and number of types.
/// When expanded, shows the entity ID input, a remove button, and the list
/// of entity types with individual edit/remove/reorder controls.
#[component]
pub(crate) fn EntityTypeCard(
    input_diagram: Signal<InputDiagram<'static>>,
    entry: EntityTypeEntry,
    index: usize,
    entry_count: usize,
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
    mut focus_index: Signal<Option<usize>>,
    mut rename_refocus: Signal<Option<RenameRefocus>>,
) -> Element {
    let entity_id = entry.entity_id.clone();

    let card_state =
        CardComponent::state_init_with_rename(index, entry_count, rename_refocus, &entity_id);
    let mut collapsed = card_state.collapsed;
    let rename_target = card_state.rename_target;
    let border_class = drag_border_class(drag_index, drop_target, index);

    let type_count = entry.types.len();

    rsx! {
        div {
            class: "{ENTITY_TYPE_CARD_CLASS} {border_class}",
            tabindex: "0",
            draggable: "true",
            "data-entity-type-card": "true",
            "data-input-diagram-field": "{entity_id}",

            // === Card-level keyboard shortcuts === //
            onkeydown: {
                let entity_id = entity_id.clone();
                CardComponent::card_onkeydown(
                    DATA_ATTR,
                    card_state,
                    move || {
                        EntityTypesPageOps::entry_move(&mut input_diagram.write(), index, index - 1);
                        focus_index.set(Some(index - 1));
                    },
                    move || {
                        EntityTypesPageOps::entry_move(&mut input_diagram.write(), index, index + 1);
                        focus_index.set(Some(index + 1));
                    },
                    move || {
                        EntityTypesPageOps::entry_remove(&mut input_diagram.write(), &entity_id);
                    },
                )
            },

            // === Drag-and-drop === //
            ondragstart: move |_| {
                drag_index.set(Some(index));
            },
            ondragover: move |evt| {
                evt.prevent_default();
                drop_target.set(Some(index));
            },
            ondrop: move |evt| {
                evt.prevent_default();
                if let Some(from) = *drag_index.read()
                    && from != index
                {
                    EntityTypesPageOps::entry_move(&mut input_diagram.write(), from, index);
                }
                drag_index.set(None);
                drop_target.set(None);
            },
            ondragend: move |_| {
                drag_index.set(None);
                drop_target.set(None);
            },

            if *collapsed.read() {
                // === Collapsed summary === //
                EntityTypeCardSummary {
                    input_diagram,
                    entity_id: entity_id.clone(),
                    type_count,
                    collapsed,
                }
            } else {
                // === Expanded content === //

                // Collapse toggle + drag handle
                div {
                    class: "flex flex-row items-center gap-1 cursor-pointer select-none mb-1",
                    onclick: move |_| collapsed.set(true),

                    DragHandle {}

                    span {
                        class: "text-gray-500 text-xs rotate-90 inline-block",
                        ">"
                    }
                    span {
                        class: "text-xs text-gray-500",
                        "Collapse"
                    }
                }

                // === Header: Entity ID + Remove === //
                EntityTypeCardFieldId {
                    input_diagram,
                    entity_id: entity_id.clone(),
                    types: entry.types.clone(),
                    rename_target,
                    rename_refocus,
                }

                // === Entity type list === //
                EntityTypeCardFieldTypes {
                    input_diagram,
                    entity_id: entity_id.clone(),
                    types: entry.types.clone(),
                }
            }
        }
    }
}
