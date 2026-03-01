//! Processes editor page.
//!
//! Allows editing `processes`: a map from `ProcessId` to `ProcessDiagram`,
//! where each `ProcessDiagram` has:
//! - `name: Option<String>`
//! - `desc: Option<String>`
//! - `steps: ProcessSteps` (map of `ProcessStepId` to display label)
//! - `step_thing_interactions: StepThingInteractions` (map of `ProcessStepId`
//!   to `Vec<EdgeGroupId>`)
//!
//! The heavy lifting is delegated to submodules:
//! - [`process_card`]: collapsible card for a single process.
//! - [`process_card_ops`]: mutation helpers for steps within a process card.
//! - [`processes_page_ops`]: mutation helpers for the page-level process map.
//! - [`step_interaction_card`]: card for a step's thing-interaction list.
//! - [`step_interaction_card_ops`]: mutation helpers for step interaction
//!   entries.

mod process_card;
mod process_card_ops;
mod processes_page_ops;
mod step_interaction_card;
mod step_interaction_card_ops;

use dioxus::{
    hooks::use_signal,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal},
};
use disposition::input_model::InputDiagram;

use crate::components::editor::common::{RenameRefocus, ADD_BTN, INPUT_CLASS, SECTION_HEADING};

use self::{process_card::ProcessCard, processes_page_ops::ProcessesPageOps};

/// Snapshot of a single process for rendering.
#[derive(Clone, PartialEq)]
pub(crate) struct ProcessEntry {
    pub(crate) process_id: String,
    pub(crate) name: String,
    pub(crate) desc: String,
    pub(crate) steps: Vec<(String, String)>,
    pub(crate) step_interactions: Vec<(String, Vec<String>)>,
}

// === ProcessCard constants === //

/// The `data-*` attribute placed on each `ProcessCard` wrapper.
///
/// Used by [`keyboard_nav`](crate::components::editor::keyboard_nav) helpers
/// to locate the nearest ancestor card.
pub(crate) const DATA_ATTR: &str = "data-process-card";

/// The `data-*` attribute that holds the card's ID value (for post-rename
/// focus).
pub(crate) const DATA_ID_ATTR: &str = "data-process-card-id";

/// CSS classes for the focusable process card wrapper.
///
/// Extends `CARD_CLASS` with focus ring styling and transitions.
pub(crate) const PROCESS_CARD_CLASS: &str = "\
    rounded-lg \
    border \
    border-gray-700 \
    bg-gray-900 \
    p-3 \
    mb-2 \
    flex \
    flex-col \
    gap-2 \
    focus:outline-none \
    focus:ring-1 \
    focus:ring-blue-400 \
    transition-all \
    duration-150\
";

/// CSS classes for the collapsed summary header.
pub(crate) const COLLAPSED_HEADER_CLASS: &str = "\
    flex \
    flex-row \
    items-center \
    gap-3 \
    cursor-pointer \
    select-none\
";

/// CSS classes for an input/textarea inside a process card.
///
/// These elements use `tabindex="-1"` so they are skipped by the normal tab
/// order; the user enters edit mode by pressing Enter on the focused card.
pub(crate) const FIELD_INPUT_CLASS: &str = INPUT_CLASS;

// === ProcessesPage component === //

/// The **Processes** editor page.
#[component]
pub fn ProcessesPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    // Post-rename focus state for process cards.
    let process_rename_refocus: Signal<Option<RenameRefocus>> = use_signal(|| None);

    let diagram = input_diagram.read();

    let entries: Vec<ProcessEntry> = diagram
        .processes
        .iter()
        .map(|(process_id, process_diagram)| {
            let steps: Vec<(String, String)> = process_diagram
                .steps
                .iter()
                .map(|(step_id, label)| (step_id.as_str().to_owned(), label.clone()))
                .collect();

            let step_interactions: Vec<(String, Vec<String>)> = process_diagram
                .step_thing_interactions
                .iter()
                .map(|(step_id, edge_group_ids)| {
                    let edge_ids: Vec<String> = edge_group_ids
                        .iter()
                        .map(|edge_group_id| edge_group_id.as_str().to_owned())
                        .collect();
                    (step_id.as_str().to_owned(), edge_ids)
                })
                .collect();

            ProcessEntry {
                process_id: process_id.as_str().to_owned(),
                name: process_diagram.name.clone().unwrap_or_default(),
                desc: process_diagram.desc.clone().unwrap_or_default(),
                steps,
                step_interactions,
            }
        })
        .collect();

    drop(diagram);

    rsx! {
        div {
            class: "flex flex-col gap-2",

            h3 { class: SECTION_HEADING, "Processes" }
            p {
                class: "text-xs text-gray-500 mb-1",
                "Processes group thing interactions into sequenced steps."
            }

            for entry in entries.iter() {
                {
                    let entry = entry.clone();
                    rsx! {
                        ProcessCard {
                            key: "{entry.process_id}",
                            input_diagram,
                            entry,
                            rename_refocus: process_rename_refocus,
                        }
                    }
                }
            }

            button {
                class: ADD_BTN,
                tabindex: -1,
                onclick: move |_| {
                    ProcessesPageOps::process_add(input_diagram);
                },
                "+ Add process"
            }
        }
    }
}
