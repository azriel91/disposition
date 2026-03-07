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
//! - **Ctrl+Shift+K**: remove the card.
//! - **Escape**: focus the parent section / tab.
//! - **Tab / Shift+Tab** (inside a field): cycle through focusable fields
//!   within the card. Wraps from last to first / first to last.
//! - **Esc** (inside a field): return focus to the card wrapper.

mod process_card_field_desc;
mod process_card_field_id;
mod process_card_field_name;
mod process_card_field_step_interactions;
mod process_card_field_steps;
mod process_card_field_steps_row;
mod process_card_summary;

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::ProcessesPageOps;

use crate::components::editor::{
    common::{CardComponent, RenameRefocus},
    processes_page::{ProcessEntry, DATA_ATTR, PROCESS_CARD_CLASS},
    reorderable::{drag_border_class, DragHandle},
};

use self::{
    process_card_field_desc::ProcessCardFieldDesc, process_card_field_id::ProcessCardFieldId,
    process_card_field_name::ProcessCardFieldName,
    process_card_field_step_interactions::ProcessCardFieldStepInteractions,
    process_card_field_steps::ProcessCardFieldSteps,
    process_card_field_steps_row::ProcessCardFieldStepsRow,
    process_card_summary::ProcessCardSummary,
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

    let card_state =
        CardComponent::state_init_with_rename(index, entry_count, rename_refocus, &process_id);
    let mut collapsed = card_state.collapsed;
    let rename_target = card_state.rename_target;
    let border_class = drag_border_class(drag_index, drop_target, index);

    let entry_name = entry.name.clone();
    let entry_desc = entry.desc.clone();

    let step_count = entry.steps.len();
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
            "data-input-diagram-field": "{process_id}",

            // === Card identity for post-rename focus === //
            "data-process-card-id": "{process_id}",

            // === Card-level keyboard shortcuts === //
            onkeydown: {
                let process_id = process_id.clone();
                CardComponent::card_onkeydown(
                    DATA_ATTR,
                    card_state,
                    move || {
                        ProcessesPageOps::process_move(&mut input_diagram.write(), index, index - 1);
                        focus_index.set(Some(index - 1));
                    },
                    move || {
                        ProcessesPageOps::process_move(&mut input_diagram.write(), index, index + 1);
                        focus_index.set(Some(index + 1));
                    },
                    move || {
                        ProcessesPageOps::process_remove(&mut input_diagram.write(), &process_id);
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
                    ProcessesPageOps::process_move(&mut input_diagram.write(), from, index);
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
                ProcessCardSummary {
                    input_diagram,
                    process_id: process_id.clone(),
                    display_name,
                    step_count,
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

                // === Header: Process ID + Remove === //
                ProcessCardFieldId {
                    input_diagram,
                    process_id: process_id.clone(),
                    rename_target,
                    rename_refocus,
                }

                // === Name === //
                ProcessCardFieldName {
                    input_diagram,
                    process_id: process_id.clone(),
                    entry_name,
                }

                // === Description === //
                ProcessCardFieldDesc {
                    input_diagram,
                    process_id: process_id.clone(),
                    entry_desc,
                }

                // === Steps === //
                ProcessCardFieldSteps {
                    input_diagram,
                    process_id: process_id.clone(),
                    steps: entry.steps.clone(),
                }

                // === Step Thing Interactions === //
                ProcessCardFieldStepInteractions {
                    input_diagram,
                    process_id: process_id.clone(),
                    step_interactions: entry.step_interactions.clone(),
                }
            }
        }
    }
}
