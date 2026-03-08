//! A single entity type row within the types list of an [`EntityTypeCard`].
//!
//! Extracted from [`EntityTypeCard`] to keep the parent component concise.
//!
//! Keyboard shortcuts (on the inputs):
//!
//! - **Alt+Up / Alt+Down**: move the type up or down in the list.
//! - **Ctrl+Shift+K**: remove the type.
//! - All other keys fall through to the standard field navigation
//!   (`field_keydown` with the card-level data attribute).
//!
//! The row also supports drag-and-drop reordering via a [`DragHandle`]
//! grip indicator, with drop-target border highlighting provided by
//! [`drag_border_class`].
//!
//! [`EntityTypeCard`]: super::EntityTypeCard

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::EntityTypesPageOps;

use crate::components::editor::{
    common::{CardComponent, FieldNav, REMOVE_BTN, ROW_CLASS},
    datalists::list_ids,
    entity_types_page::{DATA_ATTR, FIELD_INPUT_CLASS},
    reorderable::{drag_border_class, DragHandle},
};

/// A single entity type row within the types section of an entity type card.
///
/// Displays a drag handle, row index, an entity type input (with datalist),
/// and a remove button for one entry in the entity's type set. Supports
/// Alt+Up/Down keyboard reordering and drag-and-drop reordering.
#[component]
pub(crate) fn EntityTypeCardFieldTypesRow(
    input_diagram: Signal<InputDiagram<'static>>,
    entity_id: String,
    type_str: String,
    index: usize,
    type_count: usize,
    mut type_focus_idx: Signal<Option<usize>>,
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
) -> Element {
    let can_move_up = index > 0;
    let can_move_down = index + 1 < type_count;
    let border_class = drag_border_class(drag_index, drop_target, index);

    rsx! {
        div {
            class: "{ROW_CLASS} {border_class}",
            draggable: "true",
            "data-entity-type-row": "",

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
                onkeydown: {
                    let entity_id = entity_id.clone();
                    let entity_id_down = entity_id.clone();
                    CardComponent::field_onkeydown(
                        DATA_ATTR,
                        can_move_up,
                        can_move_down,
                        move || {
                            EntityTypesPageOps::type_move(
                                &mut input_diagram.write(),
                                &entity_id,
                                index,
                                index - 1,
                            );
                            type_focus_idx.set(Some(index - 1));
                        },
                        move || {
                            EntityTypesPageOps::type_move(
                                &mut input_diagram.write(),
                                &entity_id_down,
                                index,
                                index + 1,
                            );
                            type_focus_idx.set(Some(index + 1));
                        },
                    )
                },
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
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "\u{2715}"
            }
        }
    }
}
