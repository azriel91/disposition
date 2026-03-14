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
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::{
    input_model::InputDiagram,
    ir_model::node::NodeInbuilt,
    model_common::{layout::FlexDirection, Id, Set},
};

use crate::components::editor::common::SECTION_HEADING;

use self::{
    flat_entry::hierarchy_flatten, thing_hierarchy_row::ThingHierarchyRow,
    thing_hierarchy_rows::ThingHierarchyRows, thing_layout_row::ThingLayoutRow,
    thing_layout_rows::ThingLayoutRows,
};

/// CSS classes for the add-layout-override button.
const ADD_BTN: &str = "\
    rounded \
    px-2 py-1 \
    text-sm \
    font-semibold \
    cursor-pointer \
    select-none \
    bg-gray-700 \
    hover:bg-gray-600 \
    text-gray-200 \
    border \
    border-gray-600 \
    focus:outline-none \
    focus:border-blue-400\
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

    // Collect thing IDs that are containers (have children in the hierarchy)
    // so the "Add" button can pick one that doesn't already have an override.
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

    // Current layout overrides, sorted by the order they appear.
    let layout_entries: Vec<(Id<'static>, FlexDirection)> = diagram
        .thing_layouts
        .iter()
        .map(|(id, dir)| (id.clone(), *dir))
        .collect();

    // Find the first container thing that doesn't already have an override,
    // to enable/disable the add button.
    let next_addable: Option<Id<'static>> = node_inbuilt_and_container_thing_ids
        .iter()
        .find(|id| !diagram.thing_layouts.contains_key(*id))
        .cloned();

    let has_addable = next_addable.is_some();

    drop(diagram);

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
                if layout_entries.is_empty() && !has_addable {
                    p {
                        class: "text-xs text-gray-600 italic py-2 text-center",
                        "No container things in the hierarchy."
                    }
                } else if layout_entries.is_empty() {
                    p {
                        class: "text-xs text-gray-600 italic py-2 text-center",
                        "No direction overrides. Click + to add one."
                    }
                }

                for (node_inbuilt_or_thing_id, direction) in layout_entries.iter() {
                    {
                        let node_inbuilt_or_thing_id = node_inbuilt_or_thing_id.clone();
                        let direction = *direction;
                        rsx! {
                            ThingLayoutRow {
                                key: "{node_inbuilt_or_thing_id}",
                                input_diagram,
                                node_inbuilt_or_thing_id,
                                direction,
                            }
                        }
                    }
                }

                // === Add button === //
                if has_addable {
                    button {
                        class: ADD_BTN,
                        title: "Add a layout direction override",
                        onclick: move |_| {
                            let diagram = input_diagram.read();
                            // Recompute to avoid stale closure.
                            let node_inbuilt_and_container_thing_ids: Vec<Id<'static>> = {
                                let entries = hierarchy_flatten(&diagram.thing_hierarchy);

                                enum_iterator::all::<NodeInbuilt>()
                                    .map(NodeInbuilt::id)
                                    .chain(entries
                                        .iter()
                                        .enumerate()
                                        .filter(|(i, _)| {
                                            let next_depth = entries.get(i + 1).map(|e| e.depth);
                                            next_depth.is_some_and(|d| d > entries[*i].depth)
                                        })
                                        .map(|(_, e)| e.thing_id.clone().into_inner()))
                                    .collect()
                            };
                            let addable = node_inbuilt_and_container_thing_ids
                                .iter()
                                .find(|id| !diagram.thing_layouts.contains_key(*id))
                                .cloned();
                            drop(diagram);

                            if let Some(node_inbuilt_or_thing_id) = addable {
                                input_diagram
                                    .write()
                                    .thing_layouts
                                    .insert(node_inbuilt_or_thing_id, FlexDirection::Row);
                            }
                        },
                        "+ Add Direction Override"
                    }
                }
            }
        }
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
