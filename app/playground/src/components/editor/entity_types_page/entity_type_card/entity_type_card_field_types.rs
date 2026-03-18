//! Entity types list field with reorderable rows and an add button.
//!
//! Extracted from [`EntityTypeCard`] to keep the parent component concise.
//!
//! [`EntityTypeCard`]: super::EntityTypeCard

use dioxus::{
    hooks::use_signal,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::EntityTypesPageOps;

use crate::components::editor::{
    common::{FieldNav, ADD_BTN},
    entity_types_page::{
        entity_type_card::entity_type_card_field_types_row::EntityTypeCardFieldTypesRow, DATA_ATTR,
    },
    reorderable::ReorderableContainer,
};

/// Entity types list with per-row editing, Alt+Up/Down reordering, and an
/// "add" button.
///
/// Displays each entity type as an [`EntityTypeCardFieldTypesRow`] (type ID +
/// remove) inside a [`ReorderableContainer`] and provides an "+ Add type"
/// button at the bottom to append a new entry.
#[component]
pub(crate) fn EntityTypeCardFieldTypes(
    input_diagram: Signal<InputDiagram<'static>>,
    entity_id: String,
    types: Vec<String>,
) -> Element {
    let type_focus_idx: Signal<Option<usize>> = use_signal(|| None);
    let type_drag_idx: Signal<Option<usize>> = use_signal(|| None);
    let type_drop_target: Signal<Option<usize>> = use_signal(|| None);
    let type_count = types.len();

    rsx! {
        div {
            class: "flex flex-col gap-1 pl-4",

            h4 {
                class: "text-xs font-semibold text-gray-400 mt-1",
                "Types"
            }

            ReorderableContainer {
                data_attr: "data-entity-type-row".to_owned(),
                section_id: format!("entity_types_{entity_id}"),
                focus_index: type_focus_idx,
                focus_inner_selector: Some("input".to_owned()),

                for (idx, type_str) in types.iter().enumerate() {
                    {
                        let type_str = type_str.clone();
                        let entity_id = entity_id.clone();
                        let entity_id_move = entity_id.clone();
                        let entity_id_add = entity_id.clone();
                        let entity_id_remove = entity_id.clone();
                        rsx! {
                            EntityTypeCardFieldTypesRow {
                                key: "{entity_id}_{idx}",
                                input_diagram,
                                entity_id,
                                type_str,
                                index: idx,
                                type_count,
                                type_focus_idx,
                                drag_index: type_drag_idx,
                                drop_target: type_drop_target,
                                on_move: move |(from, to): (usize, usize)| {
                                    EntityTypesPageOps::type_move(
                                        &mut input_diagram.write(),
                                        &entity_id_move,
                                        from,
                                        to,
                                    );
                                },
                                on_add: move |insert_at: usize| {
                                    EntityTypesPageOps::type_add(
                                        &mut input_diagram.write(),
                                        &entity_id_add,
                                    );
                                    let last = type_count;
                                    EntityTypesPageOps::type_move(
                                        &mut input_diagram.write(),
                                        &entity_id_add,
                                        last,
                                        insert_at,
                                    );
                                },
                                on_remove: move |row_index: usize| {
                                    EntityTypesPageOps::type_remove(
                                        &mut input_diagram.write(),
                                        &entity_id_remove,
                                        row_index,
                                    );
                                },
                            }
                        }
                    }
                }
            }

            button {
                class: ADD_BTN,
                tabindex: -1,
                onclick: {
                    let entity_id = entity_id.clone();
                    move |_| {
                        EntityTypesPageOps::type_add(&mut input_diagram.write(), &entity_id);
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "+ Add type"
            }
        }
    }
}
