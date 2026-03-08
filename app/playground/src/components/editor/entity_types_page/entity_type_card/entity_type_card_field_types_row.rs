//! A single entity type row within the types list of an [`EntityTypeCard`].
//!
//! Extracted from [`EntityTypeCard`] to keep the parent component concise.
//!
//! Keyboard shortcuts (on the row wrapper):
//!
//! - **Up / Down**: move focus to the previous / next sibling row.
//! - **Ctrl+Up / Ctrl+Down**: jump to the first / last row.
//! - **Alt+Up / Alt+Down**: move the type up or down in the list.
//! - **Alt+Shift+Up / Alt+Shift+Down**: insert a new type above / below the
//!   current row.
//! - **Ctrl+Shift+K**: remove the type.
//! - **Enter**: focus the first input inside the row for editing.
//! - **Escape**: focus the parent card wrapper.
//!
//! Keyboard shortcuts (on inputs inside the row):
//!
//! - **Alt+Up / Alt+Down**: move the type up or down in the list.
//! - **Alt+Shift+Up / Alt+Shift+Down**: insert a new type above / below.
//! - **Ctrl+Shift+K**: remove the type.
//! - **Tab / Shift+Tab**: cycle through focusable fields within the row.
//! - **Escape**: return focus to the row wrapper.
//!
//! The row also supports drag-and-drop reordering via a [`DragHandle`]
//! grip indicator, with drop-target border highlighting provided by
//! [`drag_border_class`].
//!
//! [`EntityTypeCard`]: super::EntityTypeCard

use dioxus::{
    prelude::{
        component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Callback, Element, Props,
    },
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::EntityTypesPageOps;

use crate::components::editor::{
    common::{RowComponent, REMOVE_BTN, ROW_CLASS},
    datalists::list_ids,
    entity_types_page::{DATA_ATTR, FIELD_INPUT_CLASS},
    reorderable::{drag_border_class, DragHandle},
};

/// Data attribute placed on each entity type row wrapper.
///
/// Used by [`ReorderableContainer`] and keyboard navigation helpers
/// to locate sibling rows within the types list.
///
/// [`ReorderableContainer`]: crate::components::editor::reorderable::ReorderableContainer
const ROW_DATA_ATTR: &str = "data-entity-type-row";

/// A single entity type row within the types section of an entity type card.
///
/// Displays a drag handle, row index, an entity type input (with datalist),
/// and a remove button for one entry in the entity's type set. Supports
/// full keyboard navigation (Up/Down focus cycling, Alt reorder,
/// Alt+Shift insert, Ctrl+Shift+K remove, Enter to edit, Escape to card)
/// and drag-and-drop reordering.
#[component]
pub(crate) fn EntityTypeCardFieldTypesRow(
    input_diagram: Signal<InputDiagram<'static>>,
    entity_id: String,
    type_str: String,
    index: usize,
    type_count: usize,
    type_focus_idx: Signal<Option<usize>>,
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
    on_move: Callback<(usize, usize)>,
    on_add: Callback<usize>,
    on_remove: Callback<usize>,
) -> Element {
    let border_class = drag_border_class(drag_index, drop_target, index);

    rsx! {
        div {
            class: "{ROW_CLASS} {border_class} rounded focus:border-blue-400 focus:bg-gray-800 focus:outline-none",
            tabindex: "0",
            draggable: "true",
            "data-entity-type-row": "",
            "data-input-diagram-field": "{entity_id}_type_{index}",

            // === Row-level keyboard shortcuts === //
            onkeydown: RowComponent::row_onkeydown(
                ROW_DATA_ATTR,
                DATA_ATTR,
                index,
                type_count,
                type_focus_idx,
                on_move,
                on_add,
                on_remove,
            ),

            // === Drag-and-drop === //
            ondragstart: move |_| {
                drag_index.set(Some(index));
            },
            ondragover: move |evt| {
                evt.prevent_default();
                drop_target.set(Some(index));
            },
            ondrop: {
                let entity_id = entity_id.clone();
                move |evt| {
                    evt.prevent_default();
                    if let Some(from) = *drag_index.read()
                        && from != index
                    {
                        EntityTypesPageOps::type_move(&mut input_diagram.write(), &entity_id, from, index);
                    }
                    drag_index.set(None);
                    drop_target.set(None);
                }
            },
            ondragend: move |_| {
                drag_index.set(None);
                drop_target.set(None);
            },

            DragHandle {}

            span {
                class: "text-xs text-gray-500 w-6 text-right",
                "{index}."
            }

            input {
                class: FIELD_INPUT_CLASS,
                style: "max-width:14rem",
                tabindex: "-1",
                list: list_ids::ENTITY_TYPE_IDS_CUSTOM,
                placeholder: "entity_type",
                value: "{type_str}",
                onchange: {
                    let entity_id = entity_id.clone();
                    move |evt: dioxus::events::FormEvent| {
                        EntityTypesPageOps::type_update(
                            &mut input_diagram.write(),
                            &entity_id,
                            index,
                            &evt.value(),
                        );
                    }
                },
                onkeydown: RowComponent::row_field_onkeydown(
                    ROW_DATA_ATTR,
                    index,
                    type_count,
                    type_focus_idx,
                    on_move,
                    on_add,
                    on_remove,
                ),
            }

            button {
                class: REMOVE_BTN,
                tabindex: "-1",
                "data-action": "remove",
                onclick: {
                    let entity_id = entity_id.clone();
                    move |_| {
                        EntityTypesPageOps::type_remove(
                            &mut input_diagram.write(),
                            &entity_id,
                            index,
                        );
                    }
                },
                onkeydown: RowComponent::row_field_onkeydown(
                    ROW_DATA_ATTR,
                    index,
                    type_count,
                    type_focus_idx,
                    on_move,
                    on_add,
                    on_remove,
                ),
                "\u{2715}"
            }
        }
    }
}
