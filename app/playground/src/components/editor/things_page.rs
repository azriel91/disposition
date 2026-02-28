//! Things editor sub-pages.
//!
//! Provides sub-pages for:
//! - Thing Names (`things`: `ThingId` -> display name)
//! - Thing Copy Text (`thing_copy_text`: `ThingId` -> clipboard text)
//! - Entity Descriptions (`entity_descs`: `Id` -> description)
//! - Entity Tooltips (`entity_tooltips`: `Id` -> tooltip)

mod drag_handle;
mod drag_row_border_class;
mod key_value_row;
mod key_value_row_container;
mod on_change_target;
mod thing_name_row;
mod things_page_ops;

use dioxus::{
    document,
    hooks::use_signal,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Key, Props},
    signals::{ReadableExt, Signal},
};
use disposition::input_model::InputDiagram;

use crate::components::editor::{
    common::{ADD_BTN, SECTION_HEADING},
    datalists::list_ids,
};

use self::{
    key_value_row::KeyValueRow, key_value_row_container::KeyValueRowContainer,
    on_change_target::OnChangeTarget, thing_name_row::ThingNameRow, things_page_ops::ThingsPageOps,
};

/// JavaScript snippet: from the Add button, focus the last focusable child of
/// the preceding sibling container (the `KeyValueRowContainer`).
const JS_FOCUS_LAST_ROW: &str = "\
    (() => {\
        let btn = document.activeElement;\
        if (!btn) return;\
        let prev = btn.previousElementSibling;\
        while (prev) {\
            let children = prev.querySelectorAll('[tabindex=\"0\"]');\
            if (children.length > 0) {\
                children[children.length - 1].focus();\
                return;\
            }\
            prev = prev.previousElementSibling;\
        }\
    })()";

// === Thing Names sub-page === //

/// The **Things: Names** editor sub-page.
///
/// Edits `things` -- a map from `ThingId` to display label. Each entry gets
/// a [`ThingNameRow`] with editable ID, display name, and a remove button.
#[component]
pub fn ThingNamesPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    let thing_drag_idx: Signal<Option<usize>> = use_signal(|| None);
    let thing_drop_target: Signal<Option<usize>> = use_signal(|| None);
    let thing_focus_idx: Signal<Option<usize>> = use_signal(|| None);

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

            KeyValueRowContainer {
                section_id: "thing_names",
                focus_index: thing_focus_idx,

                for (idx, (id, name)) in thing_entries.iter().enumerate() {
                    {
                        let id = id.clone();
                        let name = name.clone();
                        rsx! {
                            ThingNameRow {
                                key: "{id}",
                                input_diagram,
                                thing_id: id,
                                thing_name: name,
                                index: idx,
                                entry_count: thing_count,
                                drag_index: thing_drag_idx,
                                drop_target: thing_drop_target,
                                focus_index: thing_focus_idx,
                            }
                        }
                    }
                }
            }

            button {
                class: ADD_BTN,
                tabindex: 0,
                onclick: move |_| {
                    ThingsPageOps::thing_add(input_diagram);
                },
                onkeydown: move |evt| {
                    if evt.key() == Key::ArrowUp {
                        evt.prevent_default();
                        evt.stop_propagation();
                        document::eval(JS_FOCUS_LAST_ROW);
                    }
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

            KeyValueRowContainer {
                section_id: "copy_text",
                focus_index: copy_text_focus_idx,

                for (idx, (id, text)) in copy_text_entries.iter().enumerate() {
                    {
                        let id = id.clone();
                        let text = text.clone();
                        rsx! {
                            KeyValueRow {
                                key: "ct_{id}",
                                input_diagram,
                                entry_id: id,
                                entry_value: text,
                                id_list: list_ids::THING_IDS,
                                on_change: OnChangeTarget::CopyText,
                                index: idx,
                                entry_count: copy_text_count,
                                drag_index: copy_text_drag_idx,
                                drop_target: copy_text_drop_target,
                                focus_index: copy_text_focus_idx,
                            }
                        }
                    }
                }
            }

            button {
                class: ADD_BTN,
                tabindex: 0,
                onclick: move |_| {
                    ThingsPageOps::copy_text_add(input_diagram);
                },
                onkeydown: move |evt| {
                    if evt.key() == Key::ArrowUp {
                        evt.prevent_default();
                        evt.stop_propagation();
                        document::eval(JS_FOCUS_LAST_ROW);
                    }
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

            KeyValueRowContainer {
                section_id: "entity_descs",
                focus_index: desc_focus_idx,

                for (idx, (id, desc)) in desc_entries.iter().enumerate() {
                    {
                        let id = id.clone();
                        let desc = desc.clone();
                        rsx! {
                            KeyValueRow {
                                key: "desc_{id}",
                                input_diagram,
                                entry_id: id,
                                entry_value: desc,
                                id_list: list_ids::ENTITY_IDS,
                                on_change: OnChangeTarget::EntityDesc,
                                index: idx,
                                entry_count: desc_count,
                                drag_index: desc_drag_idx,
                                drop_target: desc_drop_target,
                                focus_index: desc_focus_idx,
                            }
                        }
                    }
                }
            }

            button {
                class: ADD_BTN,
                tabindex: 0,
                onclick: move |_| {
                    ThingsPageOps::entity_desc_add(input_diagram);
                },
                onkeydown: move |evt| {
                    if evt.key() == Key::ArrowUp {
                        evt.prevent_default();
                        evt.stop_propagation();
                        document::eval(JS_FOCUS_LAST_ROW);
                    }
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

            KeyValueRowContainer {
                section_id: "entity_tooltips",
                focus_index: tooltip_focus_idx,

                for (idx, (id, tip)) in tooltip_entries.iter().enumerate() {
                    {
                        let id = id.clone();
                        let tip = tip.clone();
                        rsx! {
                            KeyValueRow {
                                key: "tip_{id}",
                                input_diagram,
                                entry_id: id,
                                entry_value: tip,
                                id_list: list_ids::ENTITY_IDS,
                                on_change: OnChangeTarget::EntityTooltip,
                                index: idx,
                                entry_count: tooltip_count,
                                drag_index: tooltip_drag_idx,
                                drop_target: tooltip_drop_target,
                                focus_index: tooltip_focus_idx,
                            }
                        }
                    }
                }
            }

            button {
                class: ADD_BTN,
                tabindex: 0,
                onclick: move |_| {
                    ThingsPageOps::entity_tooltip_add(input_diagram);
                },
                onkeydown: move |evt| {
                    if evt.key() == Key::ArrowUp {
                        evt.prevent_default();
                        evt.stop_propagation();
                        document::eval(JS_FOCUS_LAST_ROW);
                    }
                },
                "+ Add tooltip"
            }
        }
    }
}
