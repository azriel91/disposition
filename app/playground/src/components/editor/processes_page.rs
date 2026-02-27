//! Processes editor page.
//!
//! Allows editing `processes`: a map from `ProcessId` to `ProcessDiagram`,
//! where each `ProcessDiagram` has:
//! - `name: Option<String>`
//! - `desc: Option<String>`
//! - `steps: ProcessSteps` (map of `ProcessStepId` to display label)
//! - `step_thing_interactions: StepThingInteractions` (map of `ProcessStepId`
//!   to `Vec<EdgeGroupId>`)

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::{
    input_model::{process::ProcessDiagram, InputDiagram},
    model_common::edge::EdgeGroupId,
};

use crate::components::editor::{
    common::{
        id_rename_in_input_diagram, parse_edge_group_id, parse_process_id, parse_process_step_id,
        ADD_BTN, CARD_CLASS, INNER_CARD_CLASS, INPUT_CLASS, REMOVE_BTN, ROW_CLASS_SIMPLE,
        SECTION_HEADING, TEXTAREA_CLASS,
    },
    datalists::list_ids,
};

/// Snapshot of a single process for rendering.
#[derive(Clone, PartialEq)]
struct ProcessEntry {
    process_id: String,
    name: String,
    desc: String,
    steps: Vec<(String, String)>,
    step_interactions: Vec<(String, Vec<String>)>,
}

/// The **Processes** editor page.
#[component]
pub fn ProcessesPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
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
                        }
                    }
                }
            }

            div {
                class: ADD_BTN,
                onclick: move |_| {
                    ProcessesPageOps::process_add(input_diagram);
                },
                "+ Add process"
            }
        }
    }
}

// ===========================================================================
// ProcessesPage mutation helpers
// ===========================================================================

/// Mutation operations for the Processes editor page.
///
/// Grouped here so that related functions are discoverable when sorted by
/// name, per the project's `noun_verb` naming convention.
struct ProcessesPageOps;

impl ProcessesPageOps {
    /// Adds a new process with a unique placeholder ProcessId.
    fn process_add(mut input_diagram: Signal<InputDiagram<'static>>) {
        let mut n = input_diagram.read().processes.len();
        loop {
            let candidate = format!("proc_{n}");
            if let Some(process_id) = parse_process_id(&candidate) {
                if !input_diagram.read().processes.contains_key(&process_id) {
                    input_diagram
                        .write()
                        .processes
                        .insert(process_id, ProcessDiagram::default());
                    break;
                }
            }
            n += 1;
        }
    }

    /// Removes a process from the `processes` map.
    fn process_remove(mut input_diagram: Signal<InputDiagram<'static>>, process_id_str: &str) {
        if let Some(process_id) = parse_process_id(process_id_str) {
            input_diagram.write().processes.swap_remove(&process_id);
        }
    }

    /// Renames a process across all maps in the [`InputDiagram`].
    fn process_rename(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_old_str: &str,
        process_id_new_str: &str,
    ) {
        if process_id_old_str == process_id_new_str {
            return;
        }
        let process_id_old = match parse_process_id(process_id_old_str) {
            Some(process_id) => process_id,
            None => return,
        };
        let process_id_new = match parse_process_id(process_id_new_str) {
            Some(process_id) => process_id,
            None => return,
        };

        let mut input_diagram_ref = input_diagram.write();

        // processes: rename ProcessId key.
        if let Some(index) = input_diagram_ref.processes.get_index_of(&process_id_old) {
            let _result = input_diagram_ref
                .processes
                .replace_index(index, process_id_new.clone());
        }

        // Shared rename across entity_descs, entity_tooltips, entity_types,
        // and all theme style maps.
        let id_old = process_id_old.into_inner();
        let id_new = process_id_new.into_inner();
        id_rename_in_input_diagram(&mut input_diagram_ref, &id_old, &id_new);
    }

    /// Updates the display name for an existing process.
    fn process_name_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
        name: &str,
    ) {
        if let Some(process_id) = parse_process_id(process_id_str) {
            if let Some(process_diagram) = input_diagram.write().processes.get_mut(&process_id) {
                process_diagram.name = if name.is_empty() {
                    None
                } else {
                    Some(name.to_owned())
                };
            }
        }
    }

    /// Updates the description for an existing process.
    fn process_desc_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
        desc: &str,
    ) {
        if let Some(process_id) = parse_process_id(process_id_str) {
            if let Some(process_diagram) = input_diagram.write().processes.get_mut(&process_id) {
                process_diagram.desc = if desc.is_empty() {
                    None
                } else {
                    Some(desc.to_owned())
                };
            }
        }
    }
}

// ===========================================================================
// Process card component
// ===========================================================================

#[component]
fn ProcessCard(input_diagram: Signal<InputDiagram<'static>>, entry: ProcessEntry) -> Element {
    let process_id = entry.process_id.clone();

    rsx! {
        div {
            class: CARD_CLASS,

            // ── Header: Process ID + Remove ──────────────────────────
            div {
                class: ROW_CLASS_SIMPLE,

                label {
                    class: "text-xs text-gray-500 w-20",
                    "Process ID"
                }
                input {
                    class: INPUT_CLASS,
                    style: "max-width:16rem",
                    list: list_ids::PROCESS_IDS,
                    placeholder: "process_id",
                    value: "{process_id}",
                    onchange: {
                        let process_id_old = process_id.clone();
                        move |evt: dioxus::events::FormEvent| {
                            ProcessesPageOps::process_rename(input_diagram, &process_id_old, &evt.value());
                        }
                    },
                }

                span {
                    class: REMOVE_BTN,
                    onclick: {
                        let process_id = process_id.clone();
                        move |_| {
                            ProcessesPageOps::process_remove(input_diagram, &process_id);
                        }
                    },
                    "✕ Remove"
                }
            }

            // ── Name ─────────────────────────────────────────────────
            div {
                class: ROW_CLASS_SIMPLE,

                label {
                    class: "text-xs text-gray-500 w-20",
                    "Name"
                }
                input {
                    class: INPUT_CLASS,
                    placeholder: "Display name",
                    value: "{entry.name}",
                    oninput: {
                        let process_id = process_id.clone();
                        move |evt: dioxus::events::FormEvent| {
                            ProcessesPageOps::process_name_update(input_diagram, &process_id, &evt.value());
                        }
                    },
                }
            }

            // ── Description ──────────────────────────────────────────
            div {
                class: ROW_CLASS_SIMPLE,

                label {
                    class: "text-xs text-gray-500 w-20",
                    "Description"
                }
                textarea {
                    class: TEXTAREA_CLASS,
                    placeholder: "Process description (markdown)",
                    value: "{entry.desc}",
                    oninput: {
                        let process_id = process_id.clone();
                        move |evt: dioxus::events::FormEvent| {
                            ProcessesPageOps::process_desc_update(input_diagram, &process_id, &evt.value());
                        }
                    },
                }
            }

            // ── Steps ────────────────────────────────────────────────
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
                                    class: INPUT_CLASS,
                                    style: "max-width:14rem",
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
                                }

                                input {
                                    class: INPUT_CLASS,
                                    placeholder: "Step label",
                                    value: "{step_label}",
                                    oninput: {
                                        let process_id = process_id.clone();
                                        let step_id = step_id.clone();
                                        move |evt: dioxus::events::FormEvent| {
                                            ProcessCardOps::step_label_update(input_diagram, &process_id, &step_id, &evt.value());
                                        }
                                    },
                                }

                                span {
                                    class: REMOVE_BTN,
                                    onclick: {
                                        let process_id = process_id.clone();
                                        let step_id = step_id.clone();
                                        move |_| {
                                            ProcessCardOps::step_remove(input_diagram, &process_id, &step_id);
                                        }
                                    },
                                    "✕"
                                }
                            }
                        }
                    }
                }

                div {
                    class: ADD_BTN,
                    onclick: {
                        let process_id = process_id.clone();
                        move |_| {
                            ProcessCardOps::step_add(input_diagram, &process_id);
                        }
                    },
                    "+ Add step"
                }
            }

            // ── Step Thing Interactions ───────────────────────────────
            div {
                class: "flex flex-col gap-1 pl-4",

                h4 {
                    class: "text-xs font-semibold text-gray-400 mt-1",
                    "Step → Thing Interactions"
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

                div {
                    class: ADD_BTN,
                    onclick: {
                        let process_id = process_id.clone();
                        move |_| {
                            ProcessCardOps::step_interaction_add(input_diagram, &process_id);
                        }
                    },
                    "+ Add step interaction mapping"
                }
            }
        }
    }
}

// ===========================================================================
// ProcessCard mutation helpers
// ===========================================================================

/// Mutation operations for the process card component.
///
/// Grouped here so that related functions are discoverable when sorted by
/// name, per the project's `noun_verb` naming convention.
struct ProcessCardOps;

impl ProcessCardOps {
    // ── Step helpers ─────────────────────────────────────────────────

    /// Adds a new step to a process with a unique placeholder step ID.
    fn step_add(mut input_diagram: Signal<InputDiagram<'static>>, process_id_str: &str) {
        let process_id = match parse_process_id(process_id_str) {
            Some(process_id) => process_id,
            None => return,
        };
        let input_diagram_read = input_diagram.read();
        let process_diagram = match input_diagram_read.processes.get(&process_id) {
            Some(process_diagram) => process_diagram,
            None => return,
        };
        let mut n = process_diagram.steps.len();
        loop {
            let candidate = format!("{process_id_str}_step_{n}");
            if let Some(step_id) = parse_process_step_id(&candidate) {
                if !process_diagram.steps.contains_key(&step_id) {
                    drop(input_diagram_read);
                    if let Some(process_diagram) =
                        input_diagram.write().processes.get_mut(&process_id)
                    {
                        process_diagram.steps.insert(step_id, String::new());
                    }
                    break;
                }
            }
            n += 1;
        }
    }

    /// Removes a step from a process.
    fn step_remove(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
        step_id_str: &str,
    ) {
        let process_id = match parse_process_id(process_id_str) {
            Some(process_id) => process_id,
            None => return,
        };
        let step_id = match parse_process_step_id(step_id_str) {
            Some(step_id) => step_id,
            None => return,
        };
        if let Some(process_diagram) = input_diagram.write().processes.get_mut(&process_id) {
            process_diagram.steps.swap_remove(&step_id);
        }
    }

    /// Renames a step across all processes and shared maps in the
    /// [`InputDiagram`].
    fn step_rename(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
        step_id_old_str: &str,
        step_id_new_str: &str,
    ) {
        if step_id_old_str == step_id_new_str {
            return;
        }
        let _process_id = match parse_process_id(process_id_str) {
            Some(process_id) => process_id,
            None => return,
        };
        let step_id_old = match parse_process_step_id(step_id_old_str) {
            Some(step_id) => step_id,
            None => return,
        };
        let step_id_new = match parse_process_step_id(step_id_new_str) {
            Some(step_id) => step_id,
            None => return,
        };

        let mut input_diagram_ref = input_diagram.write();

        // processes: rename ProcessStepId in steps and step_thing_interactions
        // for all processes (the step ID may appear in any process).
        input_diagram_ref
            .processes
            .values_mut()
            .for_each(|process_diagram| {
                if let Some(index) = process_diagram.steps.get_index_of(&step_id_old) {
                    let _result = process_diagram
                        .steps
                        .replace_index(index, step_id_new.clone());
                }

                if let Some(index) = process_diagram
                    .step_thing_interactions
                    .get_index_of(&step_id_old)
                {
                    let _result = process_diagram
                        .step_thing_interactions
                        .replace_index(index, step_id_new.clone());
                }
            });

        // Shared rename across entity_descs, entity_tooltips, entity_types,
        // and all theme style maps.
        let id_old = step_id_old.into_inner();
        let id_new = step_id_new.into_inner();
        id_rename_in_input_diagram(&mut input_diagram_ref, &id_old, &id_new);
    }

    /// Updates the label for an existing step.
    fn step_label_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
        step_id_str: &str,
        label: &str,
    ) {
        let process_id = match parse_process_id(process_id_str) {
            Some(process_id) => process_id,
            None => return,
        };
        let step_id = match parse_process_step_id(step_id_str) {
            Some(step_id) => step_id,
            None => return,
        };
        if let Some(process_diagram) = input_diagram.write().processes.get_mut(&process_id) {
            if let Some(entry) = process_diagram.steps.get_mut(&step_id) {
                *entry = label.to_owned();
            }
        }
    }

    // ── Step interaction helpers ─────────────────────────────────────

    /// Adds a new step interaction mapping to a process.
    fn step_interaction_add(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
    ) {
        let process_id = match parse_process_id(process_id_str) {
            Some(process_id) => process_id,
            None => return,
        };
        let input_diagram_read = input_diagram.read();
        let process_diagram = match input_diagram_read.processes.get(&process_id) {
            Some(process_diagram) => process_diagram,
            None => return,
        };

        // Pick the first step that doesn't already have an interaction mapping,
        // or fall back to a placeholder.
        let step_id = process_diagram
            .steps
            .keys()
            .find(|step_id| {
                !process_diagram
                    .step_thing_interactions
                    .contains_key(*step_id)
            })
            .cloned();

        let step_id = match step_id {
            Some(step_id) => step_id,
            None => {
                // All steps already have mappings; generate a placeholder.
                let mut n = process_diagram.step_thing_interactions.len();
                loop {
                    let candidate = format!("{process_id_str}_step_{n}");
                    if let Some(step_id) = parse_process_step_id(&candidate) {
                        if !process_diagram
                            .step_thing_interactions
                            .contains_key(&step_id)
                        {
                            drop(input_diagram_read);
                            if let Some(process_diagram) =
                                input_diagram.write().processes.get_mut(&process_id)
                            {
                                process_diagram
                                    .step_thing_interactions
                                    .insert(step_id, Vec::new());
                            }
                            return;
                        }
                    }
                    n += 1;
                }
            }
        };

        drop(input_diagram_read);
        if let Some(process_diagram) = input_diagram.write().processes.get_mut(&process_id) {
            process_diagram
                .step_thing_interactions
                .insert(step_id, Vec::new());
        }
    }
}

// ===========================================================================
// Step interaction card component
// ===========================================================================

/// A card for one step's thing-interaction list.
#[component]
fn StepInteractionCard(
    input_diagram: Signal<InputDiagram<'static>>,
    process_id: String,
    step_id: String,
    edge_ids: Vec<String>,
) -> Element {
    rsx! {
        div {
            class: INNER_CARD_CLASS,

            // Step ID + remove
            div {
                class: ROW_CLASS_SIMPLE,

                input {
                    class: INPUT_CLASS,
                    style: "max-width:14rem",
                    list: list_ids::PROCESS_STEP_IDS,
                    placeholder: "step_id",
                    value: "{step_id}",
                    onchange: {
                        let process_id = process_id.clone();
                        let step_id_old = step_id.clone();
                        let edge_ids = edge_ids.clone();
                        move |evt: dioxus::events::FormEvent| {
                            StepInteractionCardOps::step_interaction_rename(
                                input_diagram,
                                &process_id,
                                &step_id_old,
                                &evt.value(),
                                &edge_ids,
                            );
                        }
                    },
                }

                span {
                    class: REMOVE_BTN,
                    onclick: {
                        let process_id = process_id.clone();
                        let step_id = step_id.clone();
                        move |_| {
                            StepInteractionCardOps::step_interaction_remove(input_diagram, &process_id, &step_id);
                        }
                    },
                    "✕"
                }
            }

            // Edge group IDs
            div {
                class: "flex flex-col gap-1 pl-4",

                for (idx, edge_group_id) in edge_ids.iter().enumerate() {
                    {
                        let edge_group_id = edge_group_id.clone();
                        let process_id = process_id.clone();
                        let step_id = step_id.clone();
                        rsx! {
                            div {
                                key: "{process_id}_{step_id}_{idx}",
                                class: ROW_CLASS_SIMPLE,

                                span {
                                    class: "text-xs text-gray-500 w-6 text-right",
                                    "{idx}."
                                }

                                input {
                                    class: INPUT_CLASS,
                                    style: "max-width:14rem",
                                    list: list_ids::EDGE_GROUP_IDS,
                                    placeholder: "edge_group_id",
                                    value: "{edge_group_id}",
                                    onchange: {
                                        let process_id = process_id.clone();
                                        let step_id = step_id.clone();
                                        move |evt: dioxus::events::FormEvent| {
                                            StepInteractionCardOps::step_interaction_edge_update(
                                                input_diagram,
                                                &process_id,
                                                &step_id,
                                                idx,
                                                &evt.value(),
                                            );
                                        }
                                    },
                                }

                                span {
                                    class: REMOVE_BTN,
                                    onclick: {
                                        let process_id = process_id.clone();
                                        let step_id = step_id.clone();
                                        move |_| {
                                            StepInteractionCardOps::step_interaction_edge_remove(
                                                input_diagram,
                                                &process_id,
                                                &step_id,
                                                idx,
                                            );
                                        }
                                    },
                                    "✕"
                                }
                            }
                        }
                    }
                }

                div {
                    class: ADD_BTN,
                    onclick: {
                        let process_id = process_id.clone();
                        let step_id = step_id.clone();
                        move |_| {
                            StepInteractionCardOps::step_interaction_edge_add(input_diagram, &process_id, &step_id);
                        }
                    },
                    "+ Add edge group"
                }
            }
        }
    }
}

// ===========================================================================
// StepInteractionCard mutation helpers
// ===========================================================================

/// Mutation operations for the step interaction card component.
///
/// Grouped here so that related functions are discoverable when sorted by
/// name, per the project's `noun_verb` naming convention.
struct StepInteractionCardOps;

impl StepInteractionCardOps {
    /// Removes a step interaction mapping from a process.
    fn step_interaction_remove(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
        step_id_str: &str,
    ) {
        let process_id = match parse_process_id(process_id_str) {
            Some(process_id) => process_id,
            None => return,
        };
        let step_id = match parse_process_step_id(step_id_str) {
            Some(step_id) => step_id,
            None => return,
        };
        if let Some(process_diagram) = input_diagram.write().processes.get_mut(&process_id) {
            process_diagram
                .step_thing_interactions
                .swap_remove(&step_id);
        }
    }

    /// Renames the step key of a step interaction mapping.
    fn step_interaction_rename(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
        step_id_old_str: &str,
        step_id_new_str: &str,
        edge_id_strs: &[String],
    ) {
        if step_id_old_str == step_id_new_str {
            return;
        }
        let process_id = match parse_process_id(process_id_str) {
            Some(process_id) => process_id,
            None => return,
        };
        let step_id_old = match parse_process_step_id(step_id_old_str) {
            Some(step_id) => step_id,
            None => return,
        };
        let step_id_new = match parse_process_step_id(step_id_new_str) {
            Some(step_id) => step_id,
            None => return,
        };
        let edge_group_ids: Vec<EdgeGroupId<'static>> = edge_id_strs
            .iter()
            .filter_map(|s| parse_edge_group_id(s))
            .collect();
        if let Some(process_diagram) = input_diagram.write().processes.get_mut(&process_id) {
            process_diagram
                .step_thing_interactions
                .swap_remove(&step_id_old);
            process_diagram
                .step_thing_interactions
                .insert(step_id_new, edge_group_ids);
        }
    }

    /// Updates a single edge group ID within a step interaction at the given
    /// index.
    fn step_interaction_edge_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
        step_id_str: &str,
        idx: usize,
        edge_group_id_new_str: &str,
    ) {
        let process_id = match parse_process_id(process_id_str) {
            Some(process_id) => process_id,
            None => return,
        };
        let step_id = match parse_process_step_id(step_id_str) {
            Some(step_id) => step_id,
            None => return,
        };
        let edge_group_id_new = match parse_edge_group_id(edge_group_id_new_str) {
            Some(edge_group_id) => edge_group_id,
            None => return,
        };
        if let Some(process_diagram) = input_diagram.write().processes.get_mut(&process_id) {
            if let Some(edge_group_ids) = process_diagram.step_thing_interactions.get_mut(&step_id)
            {
                if idx < edge_group_ids.len() {
                    edge_group_ids[idx] = edge_group_id_new;
                }
            }
        }
    }

    /// Removes an edge group from a step interaction by index.
    fn step_interaction_edge_remove(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
        step_id_str: &str,
        idx: usize,
    ) {
        let process_id = match parse_process_id(process_id_str) {
            Some(process_id) => process_id,
            None => return,
        };
        let step_id = match parse_process_step_id(step_id_str) {
            Some(step_id) => step_id,
            None => return,
        };
        if let Some(process_diagram) = input_diagram.write().processes.get_mut(&process_id) {
            if let Some(edge_group_ids) = process_diagram.step_thing_interactions.get_mut(&step_id)
            {
                if idx < edge_group_ids.len() {
                    edge_group_ids.remove(idx);
                }
            }
        }
    }

    /// Adds an edge group to a step interaction, using the first existing
    /// interaction edge group ID as a placeholder.
    fn step_interaction_edge_add(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
        step_id_str: &str,
    ) {
        let process_id = match parse_process_id(process_id_str) {
            Some(process_id) => process_id,
            None => return,
        };
        let step_id = match parse_process_step_id(step_id_str) {
            Some(step_id) => step_id,
            None => return,
        };

        // Pick the first edge group id from thing_interactions as a placeholder.
        let placeholder = {
            let input_diagram = input_diagram.read();
            input_diagram
                .thing_interactions
                .keys()
                .next()
                .map(|edge_group_id| edge_group_id.as_str().to_owned())
                .unwrap_or_else(|| "edge_0".to_owned())
        };
        let edge_group_id_new = match parse_edge_group_id(&placeholder) {
            Some(edge_group_id) => edge_group_id,
            None => return,
        };

        if let Some(process_diagram) = input_diagram.write().processes.get_mut(&process_id) {
            if let Some(edge_group_ids) = process_diagram.step_thing_interactions.get_mut(&step_id)
            {
                edge_group_ids.push(edge_group_id_new);
            }
        }
    }
}
