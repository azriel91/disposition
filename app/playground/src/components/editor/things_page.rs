//! Things editor sub-pages.
//!
//! Provides sub-pages for:
//! - Thing Names (`things`: `ThingId` -> display name)
//! - Thing Copy Text (`thing_copy_text`: `ThingId` -> clipboard text)
//! - Entity Descriptions (`entity_descs`: `Id` -> description)
//! - Entity Tooltips (`entity_tooltips`: `Id` -> tooltip)

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

pub(crate) use self::duplicate_button::DuplicateButton;

mod duplicate_button;

// === Thing Names sub-page === //

/// The **Things: Names** editor sub-page.
///
/// Edits `things` -- a map from `ThingId` to display label. Each entry gets
/// an [`IdValueRow`] with editable ID, display name, and a remove button.
#[component]
pub fn ThingNamesPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    let thing_drag_idx: Signal<Option<usize>> = use_signal(|| None);
    let thing_drop_target: Signal<Option<usize>> = use_signal(|| None);
    let thing_focus_idx: Signal<Option<usize>> = use_signal(|| None);
    let thing_rename_refocus: Signal<Option<RenameRefocus>> = use_signal(|| None);

    let diagram = input_diagram.read();
    let thing_entries: Vec<(String, String)> = diagram
        .things
        .iter()
        .map(|(id, name)| (id.as_str().to_owned(), name.clone()))
        .collect();
    drop(diagram);

    let thing_count = thing_entries.len();

    rsx! {
        div {
            class: "flex flex-col gap-2",

            h3 { class: SECTION_HEADING, "Thing Names" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Map of ThingId -> display label."
            }

            ReorderableContainer {
                data_attr: "data-entry-id".to_owned(),
                section_id: "thing_names".to_owned(),
                focus_index: thing_focus_idx,
                rename_refocus: Some(thing_rename_refocus),

                for (idx, (id, name)) in thing_entries.iter().enumerate() {
                    {
                        let id = id.clone();
                        let name = name.clone();
                        rsx! {
                            IdValueRow {
                                key: "thing_name_{id}",
                                entry_id: id,
                                entry_value: name,
                                id_list: list_ids::THING_IDS.to_owned(),
                                id_placeholder: "thing_id".to_owned(),
                                value_placeholder: "Display name".to_owned(),
                                index: idx,
                                entry_count: thing_count,
                                drag_index: thing_drag_idx,
                                drop_target: thing_drop_target,
                                focus_index: thing_focus_idx,
                                rename_refocus: thing_rename_refocus,
                                on_move: move |(from, to)| {
                                    ThingsPageOps::thing_move(&mut input_diagram.write(), from, to);
                                },
                                on_rename: move |(id_old, id_new): (String, String)| {
                                    ThingsPageOps::thing_rename(&mut input_diagram.write(), &id_old, &id_new);
                                },
                                on_update: move |(id, value): (String, String)| {
                                    ThingsPageOps::thing_name_update(&mut input_diagram.write(), &id, &value);
                                },
                                on_remove: move |id: String| {
                                    ThingsPageOps::thing_remove(&mut input_diagram.write(), &id);
                                },
                                on_add: move |insert_at: usize| {
                                    ThingsPageOps::thing_add(&mut input_diagram.write());
                                    ThingsPageOps::thing_move(&mut input_diagram.write(), thing_count, insert_at);
                                },
                                on_duplicate: move |id: String| {
                                    ThingsPageOps::thing_duplicate(&mut input_diagram.write(), &id);
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
                    ThingsPageOps::thing_add(&mut input_diagram.write());
                },
                "+ Add thing"
            }
        }
    }
}

// === Thing Copy Text sub-page === //

/// The **Things: Copy Text** editor sub-page.
///
/// Edits `thing_copy_text` -- optional clipboard text per `ThingId`.
/// Defaults to the display label when absent.
#[component]
pub fn ThingCopyTextPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    let copy_text_drag_idx: Signal<Option<usize>> = use_signal(|| None);
    let copy_text_drop_target: Signal<Option<usize>> = use_signal(|| None);
    let copy_text_focus_idx: Signal<Option<usize>> = use_signal(|| None);
    let copy_text_rename_refocus: Signal<Option<RenameRefocus>> = use_signal(|| None);

    let diagram = input_diagram.read();
    let copy_text_entries: Vec<(String, String)> = diagram
        .thing_copy_text
        .iter()
        .map(|(id, text)| (id.as_str().to_owned(), text.clone()))
        .collect();
    drop(diagram);

    let copy_text_count = copy_text_entries.len();

    rsx! {
        div {
            class: "flex flex-col gap-2",

            h3 { class: SECTION_HEADING, "Thing Copy Text" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Optional clipboard text per ThingId (defaults to display label)."
            }

            ReorderableContainer {
                data_attr: "data-entry-id".to_owned(),
                section_id: "copy_text".to_owned(),
                focus_index: copy_text_focus_idx,
                rename_refocus: Some(copy_text_rename_refocus),

                for (idx, (id, text)) in copy_text_entries.iter().enumerate() {
                    {
                        let id = id.clone();
                        let text = text.clone();
                        let on_change = OnChangeTarget::CopyText;
                        let current_value = text.clone();
                        rsx! {
                            IdValueRow {
                                key: "thing_copy_text_{id}",
                                entry_id: id,
                                entry_value: text,
                                id_list: list_ids::THING_IDS.to_owned(),
                                id_placeholder: "id".to_owned(),
                                value_placeholder: "value".to_owned(),
                                index: idx,
                                entry_count: copy_text_count,
                                drag_index: copy_text_drag_idx,
                                drop_target: copy_text_drop_target,
                                focus_index: copy_text_focus_idx,
                                rename_refocus: copy_text_rename_refocus,
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
                                    ThingsPageOps::copy_text_add(&mut input_diagram.write());
                                    ThingsPageOps::kv_entry_move(&mut input_diagram.write(), on_change, copy_text_count, insert_at);
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
                    ThingsPageOps::copy_text_add(&mut input_diagram.write());
                },
                "+ Add copy text"
            }
        }
    }
}

// === Entity Descriptions sub-page === //

/// The **Things: Descriptions** editor sub-page.
///
/// Edits `entity_descs` -- descriptions rendered next to entities in the
/// diagram.
#[component]
pub fn ThingEntityDescsPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
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
                                    ThingsPageOps::entity_desc_add(&mut input_diagram.write());
                                    ThingsPageOps::kv_entry_move(&mut input_diagram.write(), on_change, desc_count, insert_at);
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
                    ThingsPageOps::entity_desc_add(&mut input_diagram.write());
                },
                "+ Add description"
            }
        }
    }
}

// === Entity Tooltips sub-page === //

/// The **Things: Tooltips** editor sub-page.
///
/// Edits `entity_tooltips` -- tooltip text (markdown) shown on hover.
#[component]
pub fn ThingEntityTooltipsPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
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
