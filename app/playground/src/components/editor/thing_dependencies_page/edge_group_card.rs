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
//! - **Escape**: focus the parent section / tab.
//! - **Tab / Shift+Tab** (inside a field): cycle through focusable fields
//!   within the card. Wraps from last to first / first to last.
//! - **Esc** (inside a field): return focus to the card wrapper.
//!
//! Within the things list (when a thing row has focus):
//!
//! - **ArrowUp / ArrowDown**: navigate between thing rows.
//! - **Alt+Up / Alt+Down**: move the thing up or down in the list.
//! - **Enter**: focus the input inside the thing row.
//! - **Escape**: return focus to the parent card.

use dioxus::{
    hooks::use_signal,
    prelude::{
        component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Key,
        ModifiersInteraction, Props,
    },
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::{edge::EdgeKind, InputDiagram};

use crate::components::editor::{
    common::{
        CardComponent, FieldNav, RenameRefocus, ADD_BTN, LABEL_CLASS, REMOVE_BTN, ROW_CLASS_SIMPLE,
        SELECT_CLASS,
    },
    datalists::list_ids,
    reorderable::{drag_border_class, DragHandle, ReorderableContainer},
};

use super::{
    edge_group_card_ops::EdgeGroupCardOps, EdgeGroupEntry, MapTarget, COLLAPSED_HEADER_CLASS,
    DATA_ATTR, EDGE_GROUP_CARD_CLASS, FIELD_INPUT_CLASS,
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
    mut rename_refocus: Signal<Option<RenameRefocus>>,
) -> Element {
    let edge_group_id = entry.edge_group_id.clone();
    let edge_kind = entry.edge_kind;
    let things = entry.things.clone();

    // Focus-after-move state for thing reorder within this card.
    let mut thing_focus_idx: Signal<Option<usize>> = use_signal(|| None);

    let card_state =
        CardComponent::state_init_with_rename(index, entry_count, rename_refocus, &edge_group_id);
    let mut collapsed = card_state.collapsed;
    let rename_target = card_state.rename_target;
    let border_class = drag_border_class(drag_index, drop_target, index);

    let thing_count = things.len();
    let thing_suffix = if thing_count != 1 { "s" } else { "" };
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
            onkeydown: CardComponent::card_onkeydown(
                DATA_ATTR,
                card_state,
                move || {
                    EdgeGroupCardOps::edge_group_move(input_diagram, target, index, index - 1);
                    focus_index.set(Some(index - 1));
                },
                move || {
                    EdgeGroupCardOps::edge_group_move(input_diagram, target, index, index + 1);
                    focus_index.set(Some(index + 1));
                },
            ),

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
                    EdgeGroupCardOps::edge_group_move(input_diagram, target, from, index);
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
                div {
                    class: COLLAPSED_HEADER_CLASS,
                    onclick: move |_| collapsed.set(false),

                    DragHandle {}

                    // Expand chevron
                    span {
                        class: "text-gray-500 text-xs",
                        ">"
                    }

                    span {
                        class: "text-sm font-mono text-blue-400",
                        "{edge_group_id}"
                    }

                    span {
                        class: "text-xs text-gray-500 italic",
                        "{edge_kind_label}"
                    }

                    span {
                        class: "text-xs text-gray-500",
                        "({thing_count} thing{thing_suffix})"
                    }
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
                div {
                    class: ROW_CLASS_SIMPLE,

                    input {
                        class: FIELD_INPUT_CLASS,
                        style: "max-width:16rem",
                        tabindex: "-1",
                        list: list_ids::EDGE_GROUP_IDS,
                        placeholder: "edge_group_id",
                        value: "{edge_group_id}",
                        onchange: {
                            let edge_group_id_old = edge_group_id.clone();
                            move |evt: dioxus::events::FormEvent| {
                                let id_new = evt.value();
                                let target = *rename_target.read();
                                EdgeGroupCardOps::edge_group_rename(
                                    input_diagram,
                                    &edge_group_id_old,
                                    &id_new,
                                );
                                rename_refocus.set(Some(RenameRefocus {
                                    new_id: id_new,
                                    target,
                                }));
                            }
                        },
                        onkeydown: FieldNav::id_onkeydown(DATA_ATTR, rename_target)
                    }


                    button {
                        class: REMOVE_BTN,
                        tabindex: "-1",
                        "data-action": "remove",
                        onclick: {
                            let edge_group_id = edge_group_id.clone();
                            move |_| {
                                EdgeGroupCardOps::edge_group_remove(input_diagram, target, &edge_group_id);
                            }
                        },
                        onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                        "x Remove"
                    }
                }

                // === kind === //
                div {
                    class: "flex flex-col items-start gap-1 pl-4",

                    label { class: LABEL_CLASS, "kind" }

                    select {
                        class: SELECT_CLASS,
                        tabindex: "-1",
                        value: "{edge_kind}",
                        onchange: {
                            let edge_group_id = edge_group_id.clone();
                            let current_things = things.clone();
                            move |evt: dioxus::events::FormEvent| {
                                let kind_str = evt.value();
                                if let Ok(edge_kind_new) = kind_str.parse::<EdgeKind>() {
                                    EdgeGroupCardOps::edge_kind_change(
                                        input_diagram,
                                        target,
                                        &edge_group_id,
                                        edge_kind_new,
                                        &current_things,
                                    );
                                }
                            }
                        },
                        onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                        option { value: "cyclic", "Cyclic" }
                        option { value: "sequence", "Sequence" }
                        option { value: "symmetric", "Symmetric" }
                    }
                }

                // === things === //
                div {
                    class: "flex flex-col gap-1 pl-4",

                    label { class: LABEL_CLASS, "things" }

                    ReorderableContainer {
                        data_attr: "data-edge-thing-row".to_owned(),
                        section_id: format!("edge_things_{edge_group_id}"),
                        focus_index: thing_focus_idx,
                        focus_inner_selector: Some("input".to_owned()),

                        for (idx, thing_id) in things.iter().enumerate() {
                            {
                                let thing_id = thing_id.clone();
                                let edge_group_id = edge_group_id.clone();
                                let can_move_up = idx > 0;
                                let can_move_down = idx + 1 < thing_count;
                                rsx! {
                                    div {
                                        key: "{edge_group_id}_{idx}",
                                        class: ROW_CLASS_SIMPLE,
                                        "data-edge-thing-row": "",

                                        span {
                                            class: "text-xs text-gray-500 w-6 text-right",
                                            "{idx}."
                                        }

                                        input {
                                            class: FIELD_INPUT_CLASS,
                                            style: "max-width:14rem",
                                            tabindex: "-1",
                                            list: list_ids::THING_IDS,
                                            placeholder: "thing_id",
                                            value: "{thing_id}",
                                            onchange: {
                                                let edge_group_id = edge_group_id.clone();
                                                move |evt: dioxus::events::FormEvent| {
                                                    let thing_id_new = evt.value();
                                                    EdgeGroupCardOps::edge_thing_update(
                                                        input_diagram,
                                                        target,
                                                        &edge_group_id,
                                                        idx,
                                                        &thing_id_new,
                                                    );
                                                }
                                            },
                                            onkeydown: {
                                                let edge_group_id = edge_group_id.clone();
                                                move |evt: dioxus::events::KeyboardEvent| {
                                                    let alt = evt.modifiers().alt();
                                                    match evt.key() {
                                                        Key::ArrowUp if alt => {
                                                            evt.prevent_default();
                                                            evt.stop_propagation();
                                                            if can_move_up {
                                                                EdgeGroupCardOps::edge_thing_move(
                                                                    input_diagram,
                                                                    target,
                                                                    &edge_group_id,
                                                                    idx,
                                                                    idx - 1,
                                                                );
                                                                thing_focus_idx.set(Some(idx - 1));
                                                            }
                                                        }
                                                        Key::ArrowDown if alt => {
                                                            evt.prevent_default();
                                                            evt.stop_propagation();
                                                            if can_move_down {
                                                                EdgeGroupCardOps::edge_thing_move(
                                                                    input_diagram,
                                                                    target,
                                                                    &edge_group_id,
                                                                    idx,
                                                                    idx + 1,
                                                                );
                                                                thing_focus_idx.set(Some(idx + 1));
                                                            }
                                                        }
                                                        _ => {
                                                            crate::components::editor::keyboard_nav::field_keydown(evt, DATA_ATTR);
                                                        }
                                                    }
                                                }
                                            },
                                        }

                                        button {
                                            class: REMOVE_BTN,
                                            tabindex: "-1",
                                            "data-action": "remove",
                                            onclick: {
                                                let edge_group_id = edge_group_id.clone();
                                                move |_| {
                                                    EdgeGroupCardOps::edge_thing_remove(
                                                        input_diagram,
                                                        target,
                                                        &edge_group_id,
                                                        idx,
                                                    );
                                                }
                                            },
                                            onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                                            "x"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    button {
                        class: ADD_BTN,
                        tabindex: -1,
                        onclick: {
                            let edge_group_id = edge_group_id.clone();
                            move |_| {
                                EdgeGroupCardOps::edge_thing_add(input_diagram, target, &edge_group_id);
                            }
                        },
                        onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                        "+ Add thing"
                    }
                }
            }
        }
    }
}
