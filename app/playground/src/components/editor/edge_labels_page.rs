use dioxus::{
    hooks::use_signal,
    prelude::{
        component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props, WritableExt,
    },
    signals::{ReadableExt, Signal},
};
use disposition::{input_model::InputDiagram, model_common::edge::EdgeLabel};
use disposition_input_rt::EdgeLabelsPageOps;

use crate::components::editor::{
    common::{RenameRefocus, ADD_BTN, SECTION_HEADING},
    datalists::list_ids,
    id_value_row::IdValueRowEdgeLabel,
    reorderable::ReorderableContainer,
};

// === Edge Labels sub-page === //

/// The **Edges: Labels** editor sub-page.
///
/// Edits `edge_labels` -- `from` and `to` endpoint labels per edge -- and the
/// corresponding `edge_descs` entry for the same edge ID.
#[component]
pub fn EdgeLabelsPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    let label_drag_idx: Signal<Option<usize>> = use_signal(|| None);
    let label_drop_target: Signal<Option<usize>> = use_signal(|| None);
    let label_focus_idx: Signal<Option<usize>> = use_signal(|| None);
    let label_rename_refocus: Signal<Option<RenameRefocus>> = use_signal(|| None);

    let diagram = input_diagram.read();
    let label_entries = EdgeLabelsPageOps::edge_label_entries(&diagram);
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

                for (idx, (id, from, to, entity_desc)) in label_entries.iter().enumerate() {
                    {
                        let id = id.clone();
                        let from = from.clone();
                        let to = to.clone();
                        let entity_desc = entity_desc.clone();
                        let current_edge_label = EdgeLabel {
                            from: from.clone(),
                            to: to.clone(),
                        };
                        let current_entity_desc = entity_desc.clone();

                        rsx! {
                            IdValueRowEdgeLabel {
                                key: "edge_label_{id}",
                                entry_id: id,
                                entry_from: from,
                                entry_to: to,
                                entry_entity_desc: entity_desc,
                                id_list: list_ids::ENTITY_IDS.to_owned(),
                                id_placeholder: "edge_id".to_owned(),
                                index: idx,
                                entry_count: label_count,
                                drag_index: label_drag_idx,
                                drop_target: label_drop_target,
                                focus_index: label_focus_idx,
                                rename_refocus: label_rename_refocus,
                                on_move: move |(from_idx, to_idx)| {
                                    EdgeLabelsPageOps::edge_label_move(
                                        &mut input_diagram.write(),
                                        from_idx,
                                        to_idx,
                                    );
                                },
                                on_rename: {
                                    let current_edge_label = current_edge_label.clone();
                                    let current_entity_desc = current_entity_desc.clone();
                                    move |(id_old, id_new): (String, String)| {
                                        EdgeLabelsPageOps::edge_label_rename(
                                            &mut input_diagram.write(),
                                            &id_old,
                                            &id_new,
                                            current_edge_label.clone(),
                                            &current_entity_desc,
                                        );
                                    }
                                },
                                on_update_from: move |(id, new_from): (String, String)| {
                                    EdgeLabelsPageOps::edge_label_from_update(
                                        &mut input_diagram.write(),
                                        &id,
                                        &new_from,
                                    );
                                },
                                on_update_to: move |(id, new_to): (String, String)| {
                                    EdgeLabelsPageOps::edge_label_to_update(
                                        &mut input_diagram.write(),
                                        &id,
                                        &new_to,
                                    );
                                },
                                on_update_entity_desc: move |(id, new_desc): (String, String)| {
                                    EdgeLabelsPageOps::edge_label_entity_desc_update(
                                        &mut input_diagram.write(),
                                        &id,
                                        &new_desc,
                                    );
                                },
                                on_remove: move |id: String| {
                                    EdgeLabelsPageOps::edge_label_remove(
                                        &mut input_diagram.write(),
                                        &id,
                                    );
                                },
                                on_add: move |insert_at: usize| {
                                    EdgeLabelsPageOps::edge_label_add(&mut input_diagram.write());
                                    EdgeLabelsPageOps::edge_label_move(
                                        &mut input_diagram.write(),
                                        label_count,
                                        insert_at,
                                    );
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
                    EdgeLabelsPageOps::edge_label_add(&mut input_diagram.write());
                },
                "+ Add edge label"
            }
        }
    }
}
