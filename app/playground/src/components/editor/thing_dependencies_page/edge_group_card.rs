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
        RenameRefocus, RenameRefocusTarget, ADD_BTN, LABEL_CLASS, REMOVE_BTN, ROW_CLASS_SIMPLE,
        SELECT_CLASS,
    },
    datalists::list_ids,
    keyboard_nav::{self, CardKeyAction},
    reorderable::{drag_border_class, is_rename_target, DragHandle},
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

    // When this card was just recreated after an ID rename, start expanded
    // so the user can see/interact with the fields. The
    // `ReorderableContainer`'s `use_effect` handles DOM focus afterwards.
    let mut collapsed = use_signal({
        let edge_group_id = edge_group_id.clone();
        move || !is_rename_target(rename_refocus, &edge_group_id)
    });

    // Tracks which refocus target the next ID rename should use.
    // - `IdInput`: Enter or blur triggered the rename.
    // - `NextField`: forward Tab triggered the rename.
    // - `FocusParent`: Shift+Tab or Esc triggered the rename.
    let mut rename_target = use_signal(|| RenameRefocusTarget::IdInput);

    let can_move_up = index > 0;
    let can_move_down = index + 1 < entry_count;
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
            onkeydown: move |evt| {
                let action = keyboard_nav::card_keydown(evt, DATA_ATTR);
                match action {
                    CardKeyAction::MoveUp => {
                        if can_move_up {
                            EdgeGroupCardOps::edge_group_move(
                                input_diagram,
                                target,
                                index,
                                index - 1,
                            );
                            focus_index.set(Some(index - 1));
                        }
                    }
                    CardKeyAction::MoveDown => {
                        if can_move_down {
                            EdgeGroupCardOps::edge_group_move(
                                input_diagram,
                                target,
                                index,
                                index + 1,
                            );
                            focus_index.set(Some(index + 1));
                        }
                    }
                    CardKeyAction::Collapse => collapsed.set(true),
                    CardKeyAction::Expand => collapsed.set(false),
                    CardKeyAction::Toggle => {
                        let is_collapsed = *collapsed.read();
                        collapsed.set(!is_collapsed);
                    }
                    CardKeyAction::EnterEdit => collapsed.set(false),
                    CardKeyAction::None => {}
                }
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
                        onkeydown: move |evt| {
                            match evt.key() {
                                Key::Tab if evt.modifiers().shift() => {
                                    rename_target.set(RenameRefocusTarget::FocusParent);
                                }
                                Key::Tab => {
                                    rename_target.set(RenameRefocusTarget::NextField);
                                }
                                Key::Escape => {
                                    rename_target.set(RenameRefocusTarget::FocusParent);
                                }
                                Key::Enter => {
                                    rename_target.set(RenameRefocusTarget::IdInput);
                                }
                                _ => {}
                            }
                            keyboard_nav::field_keydown(evt, DATA_ATTR);
                        },
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
                        onkeydown: move |evt| {
                            keyboard_nav::field_keydown(evt, DATA_ATTR);
                        },
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
                        onkeydown: move |evt| {
                            keyboard_nav::field_keydown(evt, DATA_ATTR);
                        },
                        option { value: "cyclic", "Cyclic" }
                        option { value: "sequence", "Sequence" }
                        option { value: "symmetric", "Symmetric" }
                    }
                }

                // === things === //
                div {
                    class: "flex flex-col gap-1 pl-4",

                    label { class: LABEL_CLASS, "things" }

                    for (idx, thing_id) in things.iter().enumerate() {
                        {
                            let thing_id = thing_id.clone();
                            let edge_group_id = edge_group_id.clone();
                            rsx! {
                                div {
                                    key: "{edge_group_id}_{idx}",
                                    class: ROW_CLASS_SIMPLE,

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
                                        onkeydown: move |evt| {
                                            keyboard_nav::field_keydown(evt, DATA_ATTR);
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
                                        onkeydown: move |evt| {
                                            keyboard_nav::field_keydown(evt, DATA_ATTR);
                                        },
                                        "x"
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
                        onkeydown: move |evt| {
                            keyboard_nav::field_keydown(evt, DATA_ATTR);
                        },
                        "+ Add thing"
                    }
                }
            }
        }
    }
}
