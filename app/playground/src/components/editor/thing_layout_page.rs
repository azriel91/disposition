//! Thing layout editor page.
//!
//! Provides an interactive tree editor for the `thing_hierarchy` field of an
//! [`InputDiagram`]. Users can reorder entries via drag-and-drop or keyboard
//! shortcuts (Alt+Up/Down to move, Tab/Shift+Tab to indent/outdent).

mod drag_row_border_class;
mod flat_entry;
mod help_tooltip;
mod thing_layout_ops;
mod thing_layout_page_ops;
mod thing_layout_row;

use dioxus::{
    hooks::use_signal,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal},
};
use disposition::input_model::InputDiagram;

use crate::components::editor::common::{ADD_BTN, SECTION_HEADING};

use self::{
    flat_entry::hierarchy_flatten, help_tooltip::HelpTooltip,
    thing_layout_page_ops::ThingLayoutPageOps, thing_layout_row::ThingLayoutRow,
};

/// The **Thing Layout** editor page.
///
/// Renders the `thing_hierarchy` as an indented list of rows. Each row
/// supports:
///
/// - **Drag-and-drop**: grab the handle to reorder.
/// - **Alt+Up / Alt+Down**: move the entry up or down within its nesting level,
///   or reparent it when at the boundary of its current level.
/// - **Tab**: indent (become a child of the previous sibling).
/// - **Shift+Tab**: outdent (become a sibling of the parent).
/// - **Remove** button: delete the entry (and its subtree) from the hierarchy.
#[component]
pub fn ThingLayoutPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    // Drag-and-drop state for the hierarchy rows.
    let drag_index: Signal<Option<usize>> = use_signal(|| None);
    let drop_target: Signal<Option<usize>> = use_signal(|| None);

    // Help tooltip visibility.
    let show_help: Signal<bool> = use_signal(|| false);

    let diagram = input_diagram.read();
    let flat_entries = hierarchy_flatten(&diagram.thing_hierarchy);
    drop(diagram);

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

    rsx! {
        div {
            class: "flex flex-col gap-1",

            // === Header row with title and help button === //
            div {
                class: "flex flex-row items-center gap-2",

                h3 { class: "{SECTION_HEADING} flex-1", "Thing Hierarchy" }

                HelpTooltip { show_help }
            }

            div {
                class: "\
                    flex \
                    flex-col \
                    rounded-lg \
                    border \
                    border-gray-700 \
                    bg-gray-900 \
                    p-2 \
                    gap-0\
                ",

                if flat_entries.is_empty() {
                    p {
                        class: "text-xs text-gray-600 italic py-2 text-center",
                        "No thing hierarchy entries. Add one below."
                    }
                }

                for (idx, entry) in flat_entries.iter().enumerate() {
                    {
                        let thing_id = entry.thing_id.clone();
                        let depth = entry.depth;
                        let (is_first, is_last) = sibling_flags[idx];
                        rsx! {
                            ThingLayoutRow {
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
                            }
                        }
                    }
                }
            }

            div {
                class: ADD_BTN,
                onclick: move |_| {
                    ThingLayoutPageOps::entry_add(input_diagram);
                },
                "+ Add to hierarchy"
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
