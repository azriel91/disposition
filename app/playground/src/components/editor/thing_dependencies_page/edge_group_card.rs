//! Collapsible card component for a single edge group.
//!
//! Used by both the Thing Dependencies and Thing Interactions pages.
//! Displays the edge group ID, edge kind selector, and a list of thing IDs
//! that can be individually edited, removed, or added to.
//!
//! Supports keyboard shortcuts:
//!
//! - **ArrowUp / ArrowDown**: navigate between sibling cards.
//! - **Alt+Up / Alt+Down**: move the card up or down in the list.
//! - **ArrowRight**: expand the card (when collapsed).
//! - **ArrowLeft**: collapse the card (when expanded).
//! - **Space**: toggle expand/collapse.
//! - **Enter**: expand + focus the first input inside the card.
//! - **Ctrl+Shift+K**: remove the card.
//! - **Escape**: focus the parent section / tab.
//! - **Tab / Shift+Tab** (inside a field): cycle through focusable fields
//!   within the card. Wraps from last to first / first to last.
//! - **Esc** (inside a field): return focus to the card wrapper.
//!
//! Within the things list (when a thing row has focus):
//!
//! - **Alt+Up / Alt+Down**: move the thing up or down in the list.
//! - All other keys fall through to the standard field navigation.

mod edge_group_card_field_id;
mod edge_group_card_field_kind;
mod edge_group_card_field_things;
mod edge_group_card_field_things_row;
mod edge_group_card_summary;

use dioxus::{
    hooks::use_signal,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::{EdgeGroupCardOps, MapTarget};

use crate::components::editor::{
    common::{CardComponent, RenameRefocus},
    reorderable::{drag_border_class, DragHandle},
    thing_dependencies_page::{EdgeGroupEntry, DATA_ATTR, EDGE_GROUP_CARD_CLASS},
};

use self::{
    edge_group_card_field_id::EdgeGroupCardFieldId,
    edge_group_card_field_kind::EdgeGroupCardFieldKind,
    edge_group_card_field_things::EdgeGroupCardFieldThings,
    edge_group_card_summary::EdgeGroupCardSummary,
};

/// A collapsible card for editing a single edge group.
///
/// When collapsed, shows the edge group ID, kind, and number of things.
/// When expanded, shows the edge group ID input, kind selector, and the
/// list of thing IDs with individual edit/remove controls.
#[component]
pub(crate) fn EdgeGroupCard(
    input_diagram: Signal<InputDiagram<'static>>,
    entry: EdgeGroupEntry,
    target: MapTarget,
    index: usize,
    entry_count: usize,
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
    mut focus_index: Signal<Option<usize>>,
    rename_refocus: Signal<Option<RenameRefocus>>,
) -> Element {
    let edge_group_id = entry.edge_group_id.clone();
    let edge_kind = entry.edge_kind;
    let things = entry.things.clone();

    // Focus-after-move state for thing reorder within this card.
    let thing_focus_idx: Signal<Option<usize>> = use_signal(|| None);

    let card_state =
        CardComponent::state_init_with_rename(index, entry_count, rename_refocus, &edge_group_id);
    let mut collapsed = card_state.collapsed;
    let rename_target = card_state.rename_target;
    let border_class = drag_border_class(drag_index, drop_target, index);

    let thing_count = things.len();
    let edge_kind_label = edge_kind.to_string();

    rsx! {
        div {
            class: "{EDGE_GROUP_CARD_CLASS} {border_class}",
            tabindex: "0",
            draggable: "true",
            "data-edge-group-card": "true",

            // === Card identity for post-rename focus === //
            "data-edge-group-card-id": "{edge_group_id}",

            // === Card-level keyboard shortcuts === //
            onkeydown: {
                let edge_group_id = edge_group_id.clone();
                CardComponent::card_onkeydown(
                    DATA_ATTR,
                    card_state,
                    move || {
                        EdgeGroupCardOps::edge_group_move(
                            &mut input_diagram.write(),
                            target,
                            index,
                            index - 1,
                        );
                        focus_index.set(Some(index - 1));
                    },
                    move || {
                        EdgeGroupCardOps::edge_group_move(
                            &mut input_diagram.write(),
                            target,
                            index,
                            index + 1,
                        );
                        focus_index.set(Some(index + 1));
                    },
                    move || {
                        EdgeGroupCardOps::edge_group_remove(
                            &mut input_diagram.write(),
                            target,
                            &edge_group_id,
                        );
                    },
                )
            },

            // === Drag-and-drop === //
            ondragstart: move |_| {
                drag_index.set(Some(index));
            },
            ondragover: move |evt| {
                evt.prevent_default();
                drop_target.set(Some(index));
            },
            ondrop: move |evt| {
                evt.prevent_default();
                if let Some(from) = *drag_index.read()
                    && from != index
                {
                    EdgeGroupCardOps::edge_group_move(
                        &mut input_diagram.write(),
                        target,
                        from,
                        index,
                    );
                }
                drag_index.set(None);
                drop_target.set(None);
            },
            ondragend: move |_| {
                drag_index.set(None);
                drop_target.set(None);
            },

            if *collapsed.read() {
                // === Collapsed summary === //
                EdgeGroupCardSummary {
                    input_diagram,
                    target,
                    edge_group_id: edge_group_id.clone(),
                    edge_kind_label,
                    thing_count,
                    collapsed,
                }
            } else {
                // === Expanded content === //

                // Collapse toggle + drag handle
                div {
                    class: "flex flex-row items-center gap-1 cursor-pointer select-none mb-1",
                    onclick: move |_| collapsed.set(true),

                    DragHandle {}

                    span {
                        class: "text-gray-500 text-xs rotate-90 inline-block",
                        ">"
                    }
                    span {
                        class: "text-xs text-gray-500",
                        "Collapse"
                    }
                }

                // === EdgeGroupId + Remove === //
                EdgeGroupCardFieldId {
                    input_diagram,
                    target,
                    edge_group_id: edge_group_id.clone(),
                    rename_target,
                    rename_refocus,
                }

                // === kind === //
                EdgeGroupCardFieldKind {
                    input_diagram,
                    target,
                    edge_group_id: edge_group_id.clone(),
                    edge_kind,
                    things: things.clone(),
                }

                // === things === //
                EdgeGroupCardFieldThings {
                    input_diagram,
                    target,
                    edge_group_id: edge_group_id.clone(),
                    things,
                    thing_focus_idx,
                }
            }
        }
    }
}
