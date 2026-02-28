//! Things editor page.
//!
//! Allows editing:
//! - `things`: `ThingId` -> display name
//! - `thing_copy_text`: `ThingId` -> clipboard text
//! - `entity_descs`: `Id` -> description (filtered to thing IDs here)
//! - `entity_tooltips`: `Id` -> tooltip
//! - `thing_hierarchy`: recursive nesting of things

mod collapse_bar;
mod drag_handle;
mod drag_row_border_class;
mod key_value_row;
mod key_value_row_container;
mod on_change_target;
mod thing_name_row;
mod things_page_ops;

use dioxus::{
    hooks::use_signal,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;

use crate::{
    components::editor::{
        common::{ADD_BTN, SECTION_HEADING, TEXTAREA_CLASS},
        datalists::list_ids,
    },
    editor_state::ThingsPageUiState,
};

use self::{
    collapse_bar::CollapseBar, key_value_row::KeyValueRow,
    key_value_row_container::KeyValueRowContainer, on_change_target::OnChangeTarget,
    thing_name_row::ThingNameRow, things_page_ops::ThingsPageOps,
};

/// Number of rows shown when a section is collapsed.
const COLLAPSE_THRESHOLD: usize = 4;

/// The **Things** editor page.
///
/// Renders editable rows for each `ThingId` in the diagram's `things` map, as
/// well as associated copy-text, descriptions, tooltips, and hierarchy.
#[component]
pub fn ThingsPage(
    input_diagram: Signal<InputDiagram<'static>>,
    things_ui_state: Signal<ThingsPageUiState>,
) -> Element {
    // Drag-and-drop state: tracks the index currently being dragged per section.
    let thing_drag_idx: Signal<Option<usize>> = use_signal(|| None);
    let copy_text_drag_idx: Signal<Option<usize>> = use_signal(|| None);
    let desc_drag_idx: Signal<Option<usize>> = use_signal(|| None);
    let tooltip_drag_idx: Signal<Option<usize>> = use_signal(|| None);

    // Drop-target state: tracks which row is being hovered over per section.
    let thing_drop_target: Signal<Option<usize>> = use_signal(|| None);
    let copy_text_drop_target: Signal<Option<usize>> = use_signal(|| None);
    let desc_drop_target: Signal<Option<usize>> = use_signal(|| None);
    let tooltip_drop_target: Signal<Option<usize>> = use_signal(|| None);

    // Focus-after-move state: when set, the row at this index receives
    // focus after the next DOM update (managed by KeyValueRowContainer).
    let thing_focus_idx: Signal<Option<usize>> = use_signal(|| None);
    let copy_text_focus_idx: Signal<Option<usize>> = use_signal(|| None);
    let desc_focus_idx: Signal<Option<usize>> = use_signal(|| None);
    let tooltip_focus_idx: Signal<Option<usize>> = use_signal(|| None);

    let diagram = input_diagram.read();

    // Snapshot current thing keys + values so we can iterate without holding
    // the borrow across the event handlers.
    let thing_entries: Vec<(String, String)> = diagram
        .things
        .iter()
        .map(|(id, name)| (id.as_str().to_owned(), name.clone()))
        .collect();

    let copy_text_entries: Vec<(String, String)> = diagram
        .thing_copy_text
        .iter()
        .map(|(id, text)| (id.as_str().to_owned(), text.clone()))
        .collect();

    let desc_entries: Vec<(String, String)> = diagram
        .entity_descs
        .iter()
        .map(|(id, desc)| (id.as_str().to_owned(), desc.clone()))
        .collect();

    let tooltip_entries: Vec<(String, String)> = diagram
        .entity_tooltips
        .iter()
        .map(|(id, tip)| (id.as_str().to_owned(), tip.clone()))
        .collect();

    // Serialize the current hierarchy to a YAML snippet for a simple textarea
    // editor (hierarchy is recursive and hard to represent with flat inputs).
    let hierarchy_yaml = serde_saphyr::to_string(&diagram.thing_hierarchy)
        .unwrap_or_default()
        .trim()
        .to_owned();

    // Drop the immutable borrow before rendering (we need `input_diagram` for
    // event handlers).
    drop(diagram);

    // Read collapsed states.
    let ui = things_ui_state.read();
    let thing_names_collapsed = ui.thing_names_collapsed;
    let copy_text_collapsed = ui.copy_text_collapsed;
    let entity_descs_collapsed = ui.entity_descs_collapsed;
    let entity_tooltips_collapsed = ui.entity_tooltips_collapsed;
    drop(ui);

    // Determine which entries are visible for each section.
    let thing_names_collapsible = thing_entries.len() > COLLAPSE_THRESHOLD;
    let copy_text_collapsible = copy_text_entries.len() > COLLAPSE_THRESHOLD;
    let entity_descs_collapsible = desc_entries.len() > COLLAPSE_THRESHOLD;
    let entity_tooltips_collapsible = tooltip_entries.len() > COLLAPSE_THRESHOLD;

    let visible_things: Vec<(usize, &(String, String))> =
        if thing_names_collapsible && thing_names_collapsed {
            thing_entries
                .iter()
                .enumerate()
                .take(COLLAPSE_THRESHOLD)
                .collect()
        } else {
            thing_entries.iter().enumerate().collect()
        };

    let visible_copy_text: Vec<(usize, &(String, String))> =
        if copy_text_collapsible && copy_text_collapsed {
            copy_text_entries
                .iter()
                .enumerate()
                .take(COLLAPSE_THRESHOLD)
                .collect()
        } else {
            copy_text_entries.iter().enumerate().collect()
        };

    let visible_descs: Vec<(usize, &(String, String))> =
        if entity_descs_collapsible && entity_descs_collapsed {
            desc_entries
                .iter()
                .enumerate()
                .take(COLLAPSE_THRESHOLD)
                .collect()
        } else {
            desc_entries.iter().enumerate().collect()
        };

    let visible_tooltips: Vec<(usize, &(String, String))> =
        if entity_tooltips_collapsible && entity_tooltips_collapsed {
            tooltip_entries
                .iter()
                .enumerate()
                .take(COLLAPSE_THRESHOLD)
                .collect()
        } else {
            tooltip_entries.iter().enumerate().collect()
        };

    let thing_count = thing_entries.len();
    let copy_text_count = copy_text_entries.len();
    let desc_count = desc_entries.len();
    let tooltip_count = tooltip_entries.len();

    rsx! {
        div {
            class: "flex flex-col gap-2",

            // === Thing Names === //
            h3 { class: SECTION_HEADING, "Thing Names" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Map of ThingId -> display label."
            }

            KeyValueRowContainer {
                section_id: "thing_names",
                focus_index: thing_focus_idx,

                for (idx, (id, name)) in visible_things.iter() {
                    {
                        let id = id.clone();
                        let name = name.clone();
                        let idx = *idx;
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

            if thing_names_collapsible {
                CollapseBar {
                    collapsed: thing_names_collapsed,
                    total: thing_count,
                    visible: if thing_names_collapsed { COLLAPSE_THRESHOLD } else { thing_count },
                    on_toggle: move |_| {
                        things_ui_state.write().thing_names_collapsed = !thing_names_collapsed;
                    },
                }
            }

            div {
                class: ADD_BTN,
                onclick: move |_| {
                    ThingsPageOps::thing_add(input_diagram);
                },
                "+ Add thing"
            }

            // === Copy Text === //
            h3 { class: SECTION_HEADING, "Thing Copy Text" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Optional clipboard text per ThingId (defaults to display label)."
            }

            KeyValueRowContainer {
                section_id: "copy_text",
                focus_index: copy_text_focus_idx,

                for (idx, (id, text)) in visible_copy_text.iter() {
                    {
                        let id = id.clone();
                        let text = text.clone();
                        let idx = *idx;
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

            if copy_text_collapsible {
                CollapseBar {
                    collapsed: copy_text_collapsed,
                    total: copy_text_count,
                    visible: if copy_text_collapsed { COLLAPSE_THRESHOLD } else { copy_text_count },
                    on_toggle: move |_| {
                        things_ui_state.write().copy_text_collapsed = !copy_text_collapsed;
                    },
                }
            }

            div {
                class: ADD_BTN,
                onclick: move |_| {
                    ThingsPageOps::copy_text_add(input_diagram);
                },
                "+ Add copy text"
            }

            // === Entity Descriptions === //
            h3 { class: SECTION_HEADING, "Entity Descriptions" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Descriptions rendered next to entities in the diagram."
            }

            KeyValueRowContainer {
                section_id: "entity_descs",
                focus_index: desc_focus_idx,

                for (idx, (id, desc)) in visible_descs.iter() {
                    {
                        let id = id.clone();
                        let desc = desc.clone();
                        let idx = *idx;
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

            if entity_descs_collapsible {
                CollapseBar {
                    collapsed: entity_descs_collapsed,
                    total: desc_count,
                    visible: if entity_descs_collapsed { COLLAPSE_THRESHOLD } else { desc_count },
                    on_toggle: move |_| {
                        things_ui_state.write().entity_descs_collapsed = !entity_descs_collapsed;
                    },
                }
            }

            div {
                class: ADD_BTN,
                onclick: move |_| {
                    ThingsPageOps::entity_desc_add(input_diagram);
                },
                "+ Add description"
            }

            // === Entity Tooltips === //
            h3 { class: SECTION_HEADING, "Entity Tooltips" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Tooltip text (markdown) shown on hover."
            }

            KeyValueRowContainer {
                section_id: "entity_tooltips",
                focus_index: tooltip_focus_idx,

                for (idx, (id, tip)) in visible_tooltips.iter() {
                    {
                        let id = id.clone();
                        let tip = tip.clone();
                        let idx = *idx;
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

            if entity_tooltips_collapsible {
                CollapseBar {
                    collapsed: entity_tooltips_collapsed,
                    total: tooltip_count,
                    visible: if entity_tooltips_collapsed { COLLAPSE_THRESHOLD } else { tooltip_count },
                    on_toggle: move |_| {
                        things_ui_state.write().entity_tooltips_collapsed = !entity_tooltips_collapsed;
                    },
                }
            }

            div {
                class: ADD_BTN,
                onclick: move |_| {
                    ThingsPageOps::entity_tooltip_add(input_diagram);
                },
                "+ Add tooltip"
            }

            // === Thing Hierarchy === //
            h3 { class: SECTION_HEADING, "Thing Hierarchy (YAML)" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Recursive nesting of things. Edit as YAML."
            }
            textarea {
                class: TEXTAREA_CLASS,
                value: "{hierarchy_yaml}",
                oninput: move |evt| {
                    let text = evt.value();
                    if let Ok(hierarchy) = serde_saphyr::from_str(&text) {
                        input_diagram.write().thing_hierarchy = hierarchy;
                    }
                },
            }
        }
    }
}
