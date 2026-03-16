//! Thing layout editor page.
//!
//! Provides an interactive tree editor for the `thing_hierarchy` field of an
//! [`InputDiagram`], as well as a layout direction editor for overriding the
//! flex direction of container things via the `thing_layouts` field.
//!
//! Users can reorder hierarchy entries via drag-and-drop or keyboard
//! shortcuts (Up/Down to navigate rows, Alt+Up/Down to move,
//! Tab/Shift+Tab to indent/outdent).

mod flat_entry;
mod thing_hierarchy_row;
mod thing_hierarchy_rows;
mod thing_layout_row;
mod thing_layout_rows;

use dioxus::{
    hooks::use_signal,
    prelude::{
        component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Event, Key, Props,
    },
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::{
    input_model::InputDiagram,
    ir_model::node::NodeInbuilt,
    model_common::{layout::FlexDirection, Id, Set},
};
use disposition_input_rt::ThingLayoutOps;

use crate::components::editor::{
    common::{ID_INPUT_CLASS, SECTION_HEADING},
    reorderable::ReorderableContainer,
};

use self::{
    flat_entry::hierarchy_flatten,
    thing_hierarchy_row::ThingHierarchyRow,
    thing_hierarchy_rows::ThingHierarchyRows,
    thing_layout_row::{ThingLayoutRow, DATA_ATTR},
    thing_layout_rows::ThingLayoutRows,
};

/// Datalist element ID for the layout direction override input.
const LAYOUT_OVERRIDE_IDS_DATALIST: &str = "layout_override_ids";

/// CSS classes for the add button when enabled.
const ADD_BTN_ENABLED: &str = "\
    rounded \
    px-2 py-1 \
    text-sm \
    font-semibold \
    cursor-pointer \
    select-none \
    bg-blue-600 \
    hover:bg-blue-500 \
    text-white \
    border \
    border-blue-500 \
    focus:outline-none \
    focus:border-blue-300\
";

/// CSS classes for the add button when disabled.
const ADD_BTN_DISABLED: &str = "\
    rounded \
    px-2 py-1 \
    text-sm \
    font-semibold \
    cursor-not-allowed \
    select-none \
    bg-gray-700 \
    text-gray-500 \
    border \
    border-gray-600 \
    opacity-50\
";

/// The **Thing Layout** editor page.
///
/// This page contains two sections:
///
/// 1. **Thing Hierarchy** -- an interactive tree editor for the
///    `thing_hierarchy` field, allowing drag-and-drop reorder, indent/outdent,
///    and keyboard navigation.
///
/// 2. **Thing Layout Directions** -- a list of flex-direction overrides for
///    container things (things with children in the hierarchy). Users can add
///    new overrides, change the direction, or remove them.
#[component]
pub fn ThingLayoutPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    // === Hierarchy editor state === //

    // Drag-and-drop state for the hierarchy rows.
    let drag_index: Signal<Option<usize>> = use_signal(|| None);
    let drop_target: Signal<Option<usize>> = use_signal(|| None);

    // When set, the row at this flat index should receive focus after the
    // next DOM update. Operations that move a row (Alt+Up/Down, indent,
    // outdent) write the entry's new index here.
    let focus_index: Signal<Option<usize>> = use_signal(|| None);

    let diagram = input_diagram.read();
    let flat_entries = hierarchy_flatten(&diagram.thing_hierarchy);

    let flat_len = flat_entries.len();

    // Pre-compute sibling flags for each entry so the row component can
    // enable/disable indent/outdent buttons.
    let sibling_flags: Vec<(bool, bool)> = flat_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_first_sibling = is_first_sibling_at_depth(&flat_entries, i, entry.depth);
            let is_last_sibling = is_last_sibling_at_depth(&flat_entries, i, entry.depth);
            (is_first_sibling, is_last_sibling)
        })
        .collect();

    // === Layout direction editor state === //

    // Drag-and-drop state for layout direction rows.
    let layout_drag_index: Signal<Option<usize>> = use_signal(|| None);
    let layout_drop_target: Signal<Option<usize>> = use_signal(|| None);

    // Focus-after-move state for layout direction rows.
    let layout_focus_index: Signal<Option<usize>> = use_signal(|| None);

    // Collect node inbuilt IDs and container thing IDs (things with children
    // in the hierarchy) for the datalist suggestions.
    let node_inbuilt_and_container_thing_ids: Set<Id<'static>> =
        enum_iterator::all::<NodeInbuilt>()
            .map(NodeInbuilt::id)
            .chain(
                flat_entries
                    .iter()
                    .enumerate()
                    .filter(|(i, _entry)| {
                        // A thing is a container if the next entry has a greater depth.
                        let next_depth = flat_entries.get(i + 1).map(|e| e.depth);
                        next_depth.is_some_and(|d| d > flat_entries[*i].depth)
                    })
                    .map(|(_, entry)| entry.thing_id.clone().into_inner()),
            )
            .collect();

    // Build the regex pattern for the input: `^(id_a|id_b|id_c)$`.
    // Only IDs that are not already in `thing_layouts` are valid.
    let valid_ids: Vec<String> = node_inbuilt_and_container_thing_ids
        .iter()
        .filter(|id| !diagram.thing_layouts.contains_key(*id))
        .map(|id| id.as_str().to_owned())
        .collect();

    let pattern = if valid_ids.is_empty() {
        // Match nothing -- there are no valid IDs to add.
        "^$".to_owned()
    } else {
        format!("^({})$", valid_ids.join("|"))
    };

    // Current layout overrides, sorted by the order they appear.
    let layout_entries: Vec<(Id<'static>, FlexDirection)> = diagram
        .thing_layouts
        .iter()
        .map(|(id, dir)| (id.clone(), *dir))
        .collect();

    let layout_entry_count = layout_entries.len();
    let has_suggestions = !node_inbuilt_and_container_thing_ids.is_empty();

    drop(diagram);

    // Signal for the add-override input value.
    let mut add_input_value: Signal<String> = use_signal(String::new);

    rsx! {
        div {
            class: "flex flex-col gap-1",

            // === Section 1: Thing Hierarchy === //
            div {
                class: "flex flex-row items-center gap-2",

                h3 { class: "{SECTION_HEADING} flex-1", "Thing Hierarchy" }
            }

            ThingHierarchyRows {
                focus_index,

                if flat_entries.is_empty() {
                    p {
                        class: "text-xs text-gray-600 italic py-2 text-center",
                        "No things defined. Add one in the Things tab."
                    }
                }

                for (idx, entry) in flat_entries.iter().enumerate() {
                    {
                        let thing_id = entry.thing_id.clone();
                        let depth = entry.depth;
                        let (is_first, is_last) = sibling_flags[idx];
                        rsx! {
                            ThingHierarchyRow {
                                key: "{thing_id}_{idx}",
                                input_diagram,
                                thing_id,
                                depth,
                                flat_index: idx,
                                flat_len,
                                is_first_sibling: is_first,
                                is_last_sibling: is_last,
                                drag_index,
                                drop_target,
                                focus_index,
                            }
                        }
                    }
                }
            }

            // === Section 2: Thing Layout Directions === //
            ThingLayoutRows {
                if layout_entries.is_empty() && !has_suggestions {
                    p {
                        class: "text-xs text-gray-600 italic py-2 text-center",
                        "No container things in the hierarchy."
                    }
                } else if layout_entries.is_empty() {
                    p {
                        class: "text-xs text-gray-600 italic py-2 text-center",
                        "No direction overrides. Type an ID below to add one."
                    }
                }

                ReorderableContainer {
                    data_attr: DATA_ATTR.to_owned(),
                    section_id: "thing_layouts".to_owned(),
                    focus_index: layout_focus_index,

                    for (idx, (node_inbuilt_or_thing_id, direction)) in layout_entries.iter().enumerate() {
                        {
                            let node_inbuilt_or_thing_id = node_inbuilt_or_thing_id.clone();
                            let direction = *direction;
                            rsx! {
                                ThingLayoutRow {
                                    key: "{node_inbuilt_or_thing_id}",
                                    node_inbuilt_or_thing_id,
                                    direction,
                                    index: idx,
                                    entry_count: layout_entry_count,
                                    drag_index: layout_drag_index,
                                    drop_target: layout_drop_target,
                                    focus_index: layout_focus_index,
                                    on_move: move |(from, to)| {
                                        ThingLayoutOps::thing_layout_move(
                                            &mut input_diagram.write(),
                                            from,
                                            to,
                                        );
                                    },
                                    on_direction_change: move |(id_str, new_dir): (String, FlexDirection)| {
                                        if let Ok(id) = Id::new(&id_str) {
                                            let id = id.into_static();
                                            input_diagram
                                                .write()
                                                .thing_layouts
                                                .insert(id, new_dir);
                                        }
                                    },
                                    on_remove: move |id_str: String| {
                                        ThingLayoutOps::thing_layout_remove(
                                            &mut input_diagram.write(),
                                            &id_str,
                                        );
                                    },
                                }
                            }
                        }
                    }
                }

                // === Add override input === //
                datalist {
                    id: LAYOUT_OVERRIDE_IDS_DATALIST,
                    for id in node_inbuilt_and_container_thing_ids.iter() {
                        option { value: "{id}" }
                    }
                }

                {
                    let pattern_clone = pattern.clone();
                    let valid_ids_clone = valid_ids.clone();
                    let ids_for_keydown = node_inbuilt_and_container_thing_ids.clone();
                    let ids_for_button = node_inbuilt_and_container_thing_ids.clone();
                    rsx! {
                        div {
                            class: "flex flex-row gap-2 items-center mt-1",

                            input {
                                class: ID_INPUT_CLASS,
                                style: "max-width:14rem",
                                list: LAYOUT_OVERRIDE_IDS_DATALIST,
                                placeholder: "node_inbuilt or thing_id",
                                pattern: "{pattern_clone}",
                                value: "{add_input_value}",
                                oninput: move |evt: dioxus::events::FormEvent| {
                                    add_input_value.set(evt.value());
                                },
                                onkeydown: {
                                    let valid_ids_for_enter = valid_ids_clone.clone();
                                    move |evt: Event<dioxus::html::KeyboardData>| {
                                        if evt.key() == Key::Enter {
                                            evt.prevent_default();
                                            let value = add_input_value.read().clone();
                                            if valid_ids_for_enter.contains(&value) {
                                                thing_layout_add(
                                                    input_diagram,
                                                    &ids_for_keydown,
                                                    &value,
                                                );
                                                add_input_value.set(String::new());
                                            }
                                        }
                                    }
                                },
                            }

                            {
                                let current_value = add_input_value.read().clone();
                                let is_valid = valid_ids.contains(&current_value);
                                let btn_class = if is_valid { ADD_BTN_ENABLED } else { ADD_BTN_DISABLED };
                                rsx! {
                                    button {
                                        class: btn_class,
                                        disabled: !is_valid,
                                        title: if is_valid { "Add layout direction override" } else { "Enter a valid node_inbuilt or container thing_id" },
                                        onclick: move |_| {
                                            let value = add_input_value.read().clone();
                                            thing_layout_add(
                                                input_diagram,
                                                &ids_for_button,
                                                &value,
                                            );
                                            add_input_value.set(String::new());
                                        },
                                        "+ Add"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Adds a layout direction override for the given ID if it is valid and
/// not already present.
fn thing_layout_add(
    mut input_diagram: Signal<InputDiagram<'static>>,
    node_inbuilt_and_container_thing_ids: &Set<Id<'static>>,
    value: &str,
) {
    if let Ok(id) = Id::new(value)
        && node_inbuilt_and_container_thing_ids.contains(&id)
        && !input_diagram.read().thing_layouts.contains_key(&id)
    {
        let id = id.into_static();
        input_diagram
            .write()
            .thing_layouts
            .entry(id)
            .or_insert(FlexDirection::Row);
    }
}

/// Returns `true` if the entry at `index` is the first sibling at
/// `depth` within its parent group.
///
/// A "first sibling" has no preceding entry at the same depth before
/// hitting an entry at a shallower depth (the parent) or the start of
/// the list.
fn is_first_sibling_at_depth(
    entries: &[flat_entry::FlatEntry],
    index: usize,
    depth: usize,
) -> bool {
    for i in (0..index).rev() {
        if entries[i].depth == depth {
            return false;
        }
        if entries[i].depth < depth {
            return true;
        }
    }
    true
}

/// Returns `true` if the entry at `index` is the last sibling at
/// `depth` within its parent group.
///
/// A "last sibling" has no following entry at the same depth before
/// hitting an entry at a shallower depth (end of parent subtree) or the
/// end of the list.
fn is_last_sibling_at_depth(entries: &[flat_entry::FlatEntry], index: usize, depth: usize) -> bool {
    // Skip over own subtree (entries with depth > current).
    let mut i = index + 1;
    while i < entries.len() {
        if entries[i].depth == depth {
            return false;
        }
        if entries[i].depth < depth {
            return true;
        }
        i += 1;
    }
    true
}
