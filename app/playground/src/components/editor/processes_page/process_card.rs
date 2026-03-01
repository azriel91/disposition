//! Collapsible card component for a single process.
//!
//! Displays the process ID, display name, description, steps, and step
//! thing-interaction mappings. Supports keyboard shortcuts for navigation
//! between cards, expand/collapse, and field editing.

use dioxus::{
    document,
    hooks::{use_effect, use_signal},
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
};

use super::{
    process_card_field_keydown, process_card_ops::ProcessCardOps,
    processes_page_ops::ProcessesPageOps, step_interaction_card::StepInteractionCard, ProcessEntry,
    COLLAPSED_HEADER_CLASS, FIELD_INPUT_CLASS, JS_FOCUS_NEXT_CARD, JS_FOCUS_PREV_CARD,
    PROCESS_CARD_CLASS,
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
    mut rename_refocus: Signal<Option<RenameRefocus>>,
) -> Element {
    let process_id = entry.process_id.clone();
    let mut collapsed = use_signal(|| true);
    // Tracks which refocus target the next ID rename should use.
    // - `IdInput`: Enter or blur triggered the rename.
    // - `NextField`: forward Tab triggered the rename.
    // - `FocusParent`: Shift+Tab or Esc triggered the rename.
    let mut rename_target = use_signal(|| RenameRefocusTarget::IdInput);

    // Clone before moving into the closure so `process_id` remains available
    // for the `rsx!` block below.
    let process_id_for_effect = process_id.clone();

    // After an ID rename this card is destroyed and recreated under the new
    // key. If the rename_refocus signal carries our new ID, focus the correct
    // sub-element once the DOM has settled.
    use_effect(move || {
        let refocus = rename_refocus.read().clone();
        if let Some(RenameRefocus { new_id, target }) = refocus
            && new_id == process_id_for_effect
        {
            rename_refocus.set(None);
            // The card was destroyed and recreated -- ensure it is
            // expanded so the user can see/interact with the fields.
            collapsed.set(false);
            let js = match target {
                RenameRefocusTarget::NextField => {
                    format!(
                        "setTimeout(() => {{\
                                let card = document.querySelector(\
                                    '[data-process-card-id=\"{new_id}\"]'\
                                );\
                                if (!card) return;\
                                let items = Array.from(\
                                    card.querySelectorAll(\
                                        'input, textarea, button, [data-action=\"remove\"]'\
                                    )\
                                );\
                                if (items.length > 1) {{\
                                    items[1].focus();\
                                }} else if (items.length === 1) {{\
                                    items[0].focus();\
                                }} else {{\
                                    card.focus();\
                                }}\
                            }}, 0)"
                    )
                }
                RenameRefocusTarget::IdInput => {
                    format!(
                        "setTimeout(() => {{\
                                let card = document.querySelector(\
                                    '[data-process-card-id=\"{new_id}\"]'\
                                );\
                                if (!card) return;\
                                let input = card.querySelector('input, textarea');\
                                if (input) {{\
                                    input.focus();\
                                }} else {{\
                                    card.focus();\
                                }}\
                            }}, 0)"
                    )
                }
                RenameRefocusTarget::FocusParent => {
                    format!(
                        "setTimeout(() => {{\
                                let card = document.querySelector(\
                                    '[data-process-card-id=\"{new_id}\"]'\
                                );\
                                if (!card) return;\
                                card.focus();\
                            }}, 0)"
                    )
                }
            };
            document::eval(&js);
        }
    });

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
            class: PROCESS_CARD_CLASS,
            tabindex: "0",
            "data-process-card": "true",

            // === Card identity for post-rename focus === //
            "data-process-card-id": "{process_id}",

            // === Card-level keyboard shortcuts === //
            onkeydown: move |evt| {
                match evt.key() {
                    Key::ArrowUp => {
                        evt.prevent_default();
                        document::eval(JS_FOCUS_PREV_CARD);
                    }
                    Key::ArrowDown => {
                        evt.prevent_default();
                        document::eval(JS_FOCUS_NEXT_CARD);
                    }
                    Key::ArrowLeft => {
                        evt.prevent_default();
                        collapsed.set(true);
                    }
                    Key::ArrowRight => {
                        evt.prevent_default();
                        collapsed.set(false);
                    }
                    Key::Character(ref c) if c == " " => {
                        evt.prevent_default();
                        let is_collapsed = *collapsed.read();
                        collapsed.set(!is_collapsed);
                    }
                    Key::Enter => {
                        evt.prevent_default();
                        // Expand if collapsed, then focus the first input.
                        collapsed.set(false);
                        document::eval(
                            "setTimeout(() => {\
                                document.activeElement\
                                    ?.querySelector('input, textarea')\
                                    ?.focus();\
                            }, 0)"
                        );
                    }
                    _ => {}
                }
            },

            if *collapsed.read() {
                // === Collapsed summary === //
                div {
                    class: COLLAPSED_HEADER_CLASS,
                    onclick: move |_| collapsed.set(false),

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

                // Collapse toggle
                div {
                    class: "flex flex-row items-center gap-1 cursor-pointer select-none mb-1",
                    onclick: move |_| collapsed.set(true),

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
                            process_card_field_keydown(evt);
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
                            process_card_field_keydown(evt);
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
                            process_card_field_keydown(evt);
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
                            process_card_field_keydown(evt);
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
                                            process_card_field_keydown(evt);
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
                                            process_card_field_keydown(evt);
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
                                            process_card_field_keydown(evt);
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
                            process_card_field_keydown(evt);
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
                            process_card_field_keydown(evt);
                        },
                        "+ Add step interaction mapping"
                    }
                }
            }
        }
    }
}
