//! Collapsible card component for a single process.
//!
//! Displays the process ID, display name, description, steps, and step
//! thing-interaction mappings. Supports keyboard shortcuts for navigation
//! between cards, expand/collapse, and field editing.
//!
//! Keyboard shortcuts:
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
use disposition::input_model::InputDiagram;

use crate::components::editor::{
    common::{
        RenameRefocus, RenameRefocusTarget, ADD_BTN, REMOVE_BTN, ROW_CLASS_SIMPLE, TEXTAREA_CLASS,
    },
    datalists::list_ids,
    keyboard_nav::{self, CardKeyAction},
    reorderable::{drag_border_class, is_rename_target, DragHandle},
};

use super::{
    process_card_ops::ProcessCardOps, processes_page_ops::ProcessesPageOps,
    step_interaction_card::StepInteractionCard, ProcessEntry, COLLAPSED_HEADER_CLASS, DATA_ATTR,
    FIELD_INPUT_CLASS, PROCESS_CARD_CLASS,
};

/// A collapsible card for editing a single process.
///
/// Shows a collapsed summary (process ID, display name, step count) or an
/// expanded form with editable fields for process ID, name, description,
/// steps, and step-thing interaction mappings.
#[component]
pub(crate) fn ProcessCard(
    input_diagram: Signal<InputDiagram<'static>>,
    entry: ProcessEntry,
    index: usize,
    entry_count: usize,
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
    mut focus_index: Signal<Option<usize>>,
    mut rename_refocus: Signal<Option<RenameRefocus>>,
) -> Element {
    let process_id = entry.process_id.clone();

    // When this card was just recreated after an ID rename, start expanded
    // so the user can see/interact with the fields. The
    // `ReorderableContainer`'s `use_effect` handles DOM focus afterwards.
    let mut collapsed = use_signal({
        let process_id = process_id.clone();
        move || !is_rename_target(rename_refocus, &process_id)
    });

    // Tracks which refocus target the next ID rename should use.
    // - `IdInput`: Enter or blur triggered the rename.
    // - `NextField`: forward Tab triggered the rename.
    // - `FocusParent`: Shift+Tab or Esc triggered the rename.
    let mut rename_target = use_signal(|| RenameRefocusTarget::IdInput);

    let can_move_up = index > 0;
    let can_move_down = index + 1 < entry_count;
    let border_class = drag_border_class(drag_index, drop_target, index);

    let entry_name = entry.name.clone();
    let entry_desc = entry.desc.clone();

    let step_count = entry.steps.len();
    let step_suffix = if step_count != 1 { "s" } else { "" };
    let display_name = if entry_name.is_empty() {
        process_id.clone()
    } else {
        entry_name.clone()
    };

    rsx! {
        div {
            class: "{PROCESS_CARD_CLASS} {border_class}",
            tabindex: "0",
            draggable: "true",
            "data-process-card": "true",

            // === Card identity for post-rename focus === //
            "data-process-card-id": "{process_id}",

            // === Card-level keyboard shortcuts === //
            onkeydown: move |evt| {
                let action = keyboard_nav::card_keydown(evt, DATA_ATTR);
                match action {
                    CardKeyAction::MoveUp => {
                        if can_move_up {
                            ProcessesPageOps::process_move(
                                input_diagram,
                                index,
                                index - 1,
                            );
                            focus_index.set(Some(index - 1));
                        }
                    }
                    CardKeyAction::MoveDown => {
                        if can_move_down {
                            ProcessesPageOps::process_move(
                                input_diagram,
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
                    ProcessesPageOps::process_move(input_diagram, from, index);
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
                        "{process_id}"
                    }

                    if !entry_name.is_empty() {
                        span {
                            class: "text-sm text-gray-300",
                            "-- {display_name}"
                        }
                    }

                    span {
                        class: "text-xs text-gray-500",
                        "({step_count} step{step_suffix})"
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

                // === Header: Process ID + Remove === //
                div {
                    class: ROW_CLASS_SIMPLE,

                    label {
                        class: "text-xs text-gray-500 w-20",
                        "Process ID"
                    }
                    input {
                        class: FIELD_INPUT_CLASS,
                        style: "max-width:16rem",
                        tabindex: "-1",
                        list: list_ids::PROCESS_IDS,
                        placeholder: "process_id",
                        value: "{process_id}",
                        onchange: {
                            let process_id_old = process_id.clone();
                            move |evt: dioxus::events::FormEvent| {
                                let id_new = evt.value();
                                let target = *rename_target.read();
                                ProcessesPageOps::process_rename(
                                    input_diagram,
                                    &process_id_old,
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
                            let process_id = process_id.clone();
                            move |_| {
                                ProcessesPageOps::process_remove(input_diagram, &process_id);
                            }
                        },
                        onkeydown: move |evt| {
                            keyboard_nav::field_keydown(evt, DATA_ATTR);
                        },
                        "\u{2715} Remove"
                    }
                }

                // === Name === //
                div {
                    class: ROW_CLASS_SIMPLE,

                    label {
                        class: "text-xs text-gray-500 w-20",
                        "Name"
                    }
                    input {
                        class: FIELD_INPUT_CLASS,
                        tabindex: "-1",
                        placeholder: "Display name",
                        value: "{entry_name}",
                        oninput: {
                            let process_id = process_id.clone();
                            move |evt: dioxus::events::FormEvent| {
                                ProcessesPageOps::process_name_update(input_diagram, &process_id, &evt.value());
                            }
                        },
                        onkeydown: move |evt| {
                            keyboard_nav::field_keydown(evt, DATA_ATTR);
                        },
                    }
                }

                // === Description === //
                div {
                    class: ROW_CLASS_SIMPLE,

                    label {
                        class: "text-xs text-gray-500 w-20",
                        "Description"
                    }
                    textarea {
                        class: TEXTAREA_CLASS,
                        tabindex: "-1",
                        placeholder: "Process description (markdown)",
                        value: "{entry_desc}",
                        oninput: {
                            let process_id = process_id.clone();
                            move |evt: dioxus::events::FormEvent| {
                                ProcessesPageOps::process_desc_update(input_diagram, &process_id, &evt.value());
                            }
                        },
                        onkeydown: move |evt| {
                            keyboard_nav::field_keydown(evt, DATA_ATTR);
                        },
                    }
                }

                // === Steps === //
                div {
                    class: "flex flex-col gap-1 pl-4",

                    h4 {
                        class: "text-xs font-semibold text-gray-400 mt-1",
                        "Steps"
                    }

                    for (step_id, step_label) in entry.steps.iter() {
                        {
                            let step_id = step_id.clone();
                            let step_label = step_label.clone();
                            let process_id = process_id.clone();
                            rsx! {
                                div {
                                    key: "{process_id}_{step_id}",
                                    class: ROW_CLASS_SIMPLE,

                                    input {
                                        class: FIELD_INPUT_CLASS,
                                        style: "max-width:14rem",
                                        tabindex: "-1",
                                        list: list_ids::PROCESS_STEP_IDS,
                                        placeholder: "step_id",
                                        value: "{step_id}",
                                        onchange: {
                                            let process_id = process_id.clone();
                                            let step_id_old = step_id.clone();
                                            move |evt: dioxus::events::FormEvent| {
                                                ProcessCardOps::step_rename(input_diagram, &process_id, &step_id_old, &evt.value());
                                            }
                                        },
                                        onkeydown: move |evt| {
                                            keyboard_nav::field_keydown(evt, DATA_ATTR);
                                        },
                                    }

                                    input {
                                        class: FIELD_INPUT_CLASS,
                                        tabindex: "-1",
                                        placeholder: "Step label",
                                        value: "{step_label}",
                                        oninput: {
                                            let process_id = process_id.clone();
                                            let step_id = step_id.clone();
                                            move |evt: dioxus::events::FormEvent| {
                                                ProcessCardOps::step_label_update(input_diagram, &process_id, &step_id, &evt.value());
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
                                            let process_id = process_id.clone();
                                            let step_id = step_id.clone();
                                            move |_| {
                                                ProcessCardOps::step_remove(input_diagram, &process_id, &step_id);
                                            }
                                        },
                                        onkeydown: move |evt| {
                                            keyboard_nav::field_keydown(evt, DATA_ATTR);
                                        },
                                        "\u{2715}"
                                    }
                                }
                            }
                        }
                    }

                    button {
                        class: ADD_BTN,
                        tabindex: -1,
                        onclick: {
                            let process_id = process_id.clone();
                            move |_| {
                                ProcessCardOps::step_add(input_diagram, &process_id);
                            }
                        },
                        onkeydown: move |evt| {
                            keyboard_nav::field_keydown(evt, DATA_ATTR);
                        },
                        "+ Add step"
                    }
                }

                // === Step Thing Interactions === //
                div {
                    class: "flex flex-col gap-1 pl-4",

                    h4 {
                        class: "text-xs font-semibold text-gray-400 mt-1",
                        "Step -> Thing Interactions"
                    }

                    for (step_id, edge_ids) in entry.step_interactions.iter() {
                        {
                            let step_id = step_id.clone();
                            let edge_ids = edge_ids.clone();
                            let process_id = process_id.clone();
                            rsx! {
                                StepInteractionCard {
                                    key: "{process_id}_sti_{step_id}",
                                    input_diagram,
                                    process_id,
                                    step_id,
                                    edge_ids,
                                }
                            }
                        }
                    }

                    button {
                        class: ADD_BTN,
                        tabindex: -1,
                        onclick: {
                            let process_id = process_id.clone();
                            move |_| {
                                ProcessCardOps::step_interaction_add(input_diagram, &process_id);
                            }
                        },
                        onkeydown: move |evt| {
                            keyboard_nav::field_keydown(evt, DATA_ATTR);
                        },
                        "+ Add step interaction mapping"
                    }
                }
            }
        }
    }
}
