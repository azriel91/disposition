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
    id_value_row::IdValueRowTextMulti,
    reorderable::ReorderableContainer,
};

// === Edge Labels sub-page === //

/// The **Edges: Labels** editor sub-page.
///
/// Edits edge label entries -- labels rendered next to edges where they exit
/// or enter a node.
#[component]
pub fn EdgeLabelsPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    let label_drag_idx: Signal<Option<usize>> = use_signal(|| None);
    let label_drop_target: Signal<Option<usize>> = use_signal(|| None);
    let label_focus_idx: Signal<Option<usize>> = use_signal(|| None);
    let label_rename_refocus: Signal<Option<RenameRefocus>> = use_signal(|| None);

    let diagram = input_diagram.read();
    let label_entries: Vec<(String, String)> = diagram
        .entity_descs
        .iter()
        .map(|(id, desc)| (id.as_str().to_owned(), desc.clone()))
        .collect();
    drop(diagram);

    let label_count = label_entries.len();

    rsx! {
        div {
            class: "flex flex-col gap-2",

            h3 { class: SECTION_HEADING, "Edge Labels" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Labels rendered next to edges where they exit or enter a node."
            }

            ReorderableContainer {
                data_attr: "data-entry-id".to_owned(),
                section_id: "edge_labels".to_owned(),
                focus_index: label_focus_idx,
                rename_refocus: Some(label_rename_refocus),

                for (idx, (id, label)) in label_entries.iter().enumerate() {
                    {
                        let id = id.clone();
                        let label = label.clone();
                        let on_change = OnChangeTarget::EntityDesc;
                        let current_value = label.clone();
                        rsx! {
                            IdValueRowTextMulti {
                                key: "edge_label_{id}",
                                entry_id: id,
                                entry_value: label,
                                id_list: list_ids::ENTITY_IDS.to_owned(),
                                id_placeholder: "id".to_owned(),
                                value_placeholder: "value".to_owned(),
                                index: idx,
                                entry_count: label_count,
                                drag_index: label_drag_idx,
                                drop_target: label_drop_target,
                                focus_index: label_focus_idx,
                                rename_refocus: label_rename_refocus,
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
                                    EntityPageOps::kv_entry_move(&mut input_diagram.write(), on_change, label_count, insert_at);
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
                "+ Add label"
            }
        }
    }
}
