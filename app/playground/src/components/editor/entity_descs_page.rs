use dioxus::{
    hooks::use_signal,
    prelude::{
        component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props, WritableExt,
    },
    signals::{ReadableExt, Signal},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::{EntityPageOps, OnChangeTarget};

use crate::components::editor::{
    common::{RenameRefocus, ADD_BTN, SECTION_HEADING},
    datalists::list_ids,
    id_value_row::IdValueRow,
    reorderable::ReorderableContainer,
};

// === Entity Descriptions sub-page === //

/// The **Things: Descriptions** editor sub-page.
///
/// Edits `entity_descs` -- descriptions rendered next to entities in the
/// diagram.
#[component]
pub fn EntityDescsPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    let desc_drag_idx: Signal<Option<usize>> = use_signal(|| None);
    let desc_drop_target: Signal<Option<usize>> = use_signal(|| None);
    let desc_focus_idx: Signal<Option<usize>> = use_signal(|| None);
    let desc_rename_refocus: Signal<Option<RenameRefocus>> = use_signal(|| None);

    let diagram = input_diagram.read();
    let desc_entries: Vec<(String, String)> = diagram
        .entity_descs
        .iter()
        .map(|(id, desc)| (id.as_str().to_owned(), desc.clone()))
        .collect();
    drop(diagram);

    let desc_count = desc_entries.len();

    rsx! {
        div {
            class: "flex flex-col gap-2",

            h3 { class: SECTION_HEADING, "Entity Descriptions" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Descriptions rendered next to entities in the diagram."
            }

            ReorderableContainer {
                data_attr: "data-entry-id".to_owned(),
                section_id: "entity_descs".to_owned(),
                focus_index: desc_focus_idx,
                rename_refocus: Some(desc_rename_refocus),

                for (idx, (id, desc)) in desc_entries.iter().enumerate() {
                    {
                        let id = id.clone();
                        let desc = desc.clone();
                        let on_change = OnChangeTarget::EntityDesc;
                        let current_value = desc.clone();
                        rsx! {
                            IdValueRow {
                                key: "entity_desc_{id}",
                                entry_id: id,
                                entry_value: desc,
                                id_list: list_ids::ENTITY_IDS.to_owned(),
                                id_placeholder: "id".to_owned(),
                                value_placeholder: "value".to_owned(),
                                index: idx,
                                entry_count: desc_count,
                                drag_index: desc_drag_idx,
                                drop_target: desc_drop_target,
                                focus_index: desc_focus_idx,
                                rename_refocus: desc_rename_refocus,
                                on_move: move |(from, to)| {
                                    EntityPageOps::kv_entry_move(&mut input_diagram.write(), on_change, from, to);
                                },
                                on_rename: {
                                    let current_value = current_value.clone();
                                    move |(id_old, id_new): (String, String)| {
                                        EntityPageOps::kv_entry_rename(
                                            &mut input_diagram.write(),
                                            on_change,
                                            &id_old,
                                            &id_new,
                                            &current_value,
                                        );
                                    }
                                },
                                on_update: move |(id, value): (String, String)| {
                                    EntityPageOps::kv_entry_update(&mut input_diagram.write(), on_change, &id, &value);
                                },
                                on_remove: move |id: String| {
                                    EntityPageOps::kv_entry_remove(&mut input_diagram.write(), on_change, &id);
                                },
                                on_add: move |insert_at: usize| {
                                    EntityPageOps::entity_desc_add(&mut input_diagram.write());
                                    EntityPageOps::kv_entry_move(&mut input_diagram.write(), on_change, desc_count, insert_at);
                                },
                            }
                        }
                    }
                }
            }

            button {
                class: ADD_BTN,
                tabindex: 0,
                onclick: move |_| {
                    EntityPageOps::entity_desc_add(&mut input_diagram.write());
                },
                "+ Add description"
            }
        }
    }
}
