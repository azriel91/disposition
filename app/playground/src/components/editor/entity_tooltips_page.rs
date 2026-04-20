//! Entity Tooltips editor page.
//!
//! Allows editing `entity_tooltips`: tooltip text (markdown) shown on hover
//! for both nodes (things) and edges.

use dioxus::{
    hooks::use_signal,
    prelude::{
        component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props, WritableExt,
    },
    signals::{ReadableExt, Signal},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::{OnChangeTarget, ThingsPageOps};

use crate::components::editor::{
    common::{RenameRefocus, ADD_BTN, SECTION_HEADING},
    datalists::list_ids,
    id_value_row::IdValueRow,
    reorderable::ReorderableContainer,
};

/// The **Entity: Tooltips** editor sub-page.
///
/// Edits `entity_tooltips` -- tooltip text (markdown) shown on hover for both
/// nodes (things) and edges.
#[component]
pub fn EntityTooltipsPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    let tooltip_drag_idx: Signal<Option<usize>> = use_signal(|| None);
    let tooltip_drop_target: Signal<Option<usize>> = use_signal(|| None);
    let tooltip_focus_idx: Signal<Option<usize>> = use_signal(|| None);
    let tooltip_rename_refocus: Signal<Option<RenameRefocus>> = use_signal(|| None);

    let diagram = input_diagram.read();
    let tooltip_entries: Vec<(String, String)> = diagram
        .entity_tooltips
        .iter()
        .map(|(id, tip)| (id.as_str().to_owned(), tip.clone()))
        .collect();
    drop(diagram);

    let tooltip_count = tooltip_entries.len();

    rsx! {
        div {
            class: "flex flex-col gap-2",

            h3 { class: SECTION_HEADING, "Entity Tooltips" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Tooltip text (markdown) shown on hover."
            }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Note that for edges, you need to add `__{{index}}` to the edge group ID where {{index}} is the n'th edge in the group."
                br {}
                "For example, for an edge group whose ID is `edge_dep_a_to_b`, the first edge's ID is `edge_dep_a_to_b__0`, the second `edge_dep_a_to_b__1`, and so on."
            }

            ReorderableContainer {
                data_attr: "data-entry-id".to_owned(),
                section_id: "entity_tooltips".to_owned(),
                focus_index: tooltip_focus_idx,
                rename_refocus: Some(tooltip_rename_refocus),

                for (idx, (id, tip)) in tooltip_entries.iter().enumerate() {
                    {
                        let id = id.clone();
                        let tip = tip.clone();
                        let on_change = OnChangeTarget::EntityTooltip;
                        let current_value = tip.clone();
                        rsx! {
                            IdValueRow {
                                key: "tip_{id}",
                                entry_id: id,
                                entry_value: tip,
                                id_list: list_ids::ENTITY_IDS.to_owned(),
                                id_placeholder: "id".to_owned(),
                                value_placeholder: "value".to_owned(),
                                index: idx,
                                entry_count: tooltip_count,
                                drag_index: tooltip_drag_idx,
                                drop_target: tooltip_drop_target,
                                focus_index: tooltip_focus_idx,
                                rename_refocus: tooltip_rename_refocus,
                                on_move: move |(from, to)| {
                                    ThingsPageOps::kv_entry_move(&mut input_diagram.write(), on_change, from, to);
                                },
                                on_rename: {
                                    let current_value = current_value.clone();
                                    move |(id_old, id_new): (String, String)| {
                                        ThingsPageOps::kv_entry_rename(
                                            &mut input_diagram.write(),
                                            on_change,
                                            &id_old,
                                            &id_new,
                                            &current_value,
                                        );
                                    }
                                },
                                on_update: move |(id, value): (String, String)| {
                                    ThingsPageOps::kv_entry_update(&mut input_diagram.write(), on_change, &id, &value);
                                },
                                on_remove: move |id: String| {
                                    ThingsPageOps::kv_entry_remove(&mut input_diagram.write(), on_change, &id);
                                },
                                on_add: move |insert_at: usize| {
                                    ThingsPageOps::entity_tooltip_add(&mut input_diagram.write());
                                    ThingsPageOps::kv_entry_move(&mut input_diagram.write(), on_change, tooltip_count, insert_at);
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
                    ThingsPageOps::entity_tooltip_add(&mut input_diagram.write());
                },
                "+ Add tooltip"
            }
        }
    }
}
