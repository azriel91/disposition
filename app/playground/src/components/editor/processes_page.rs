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
    input_model::{
        process::{ProcessDiagram, ProcessId, ProcessStepId},
        InputDiagram,
    },
    model_common::{edge::EdgeGroupId, Id},
};

use super::datalists::list_ids;

/// CSS classes shared by section headings.
const SECTION_HEADING: &str = "text-sm font-bold text-gray-300 mt-4 mb-1";

/// CSS classes for a card-like container for each process.
const CARD_CLASS: &str = "\
    rounded-lg \
    border \
    border-gray-700 \
    bg-gray-900 \
    p-3 \
    mb-2 \
    flex \
    flex-col \
    gap-2\
";

/// CSS classes for a nested card (steps within a process).
const INNER_CARD_CLASS: &str = "\
    rounded \
    border \
    border-gray-700 \
    bg-gray-850 \
    p-2 \
    flex \
    flex-col \
    gap-1\
";

/// CSS classes for text inputs.
const INPUT_CLASS: &str = "\
    flex-1 \
    rounded \
    border \
    border-gray-600 \
    bg-gray-800 \
    text-gray-200 \
    px-2 py-1 \
    text-sm \
    font-mono \
    focus:border-blue-400 \
    focus:outline-none\
";

/// CSS classes for the small "remove" button.
const REMOVE_BTN: &str = "\
    text-red-400 \
    hover:text-red-300 \
    text-xs \
    cursor-pointer \
    px-1\
";

/// CSS classes for the "add" button.
const ADD_BTN: &str = "\
    mt-1 \
    text-sm \
    text-blue-400 \
    hover:text-blue-300 \
    cursor-pointer \
    select-none\
";

/// Row-level flex layout.
const ROW_CLASS: &str = "flex flex-row gap-2 items-center";

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
        .map(|(pid, proc)| {
            let steps: Vec<(String, String)> = proc
                .steps
                .iter()
                .map(|(sid, label)| (sid.as_str().to_owned(), label.clone()))
                .collect();

            let step_interactions: Vec<(String, Vec<String>)> = proc
                .step_thing_interactions
                .iter()
                .map(|(sid, edge_ids)| {
                    let eids: Vec<String> =
                        edge_ids.iter().map(|e| e.as_str().to_owned()).collect();
                    (sid.as_str().to_owned(), eids)
                })
                .collect();

            ProcessEntry {
                process_id: pid.as_str().to_owned(),
                name: proc.name.clone().unwrap_or_default(),
                desc: proc.desc.clone().unwrap_or_default(),
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
                    add_process(input_diagram);
                },
                "+ Add process"
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Process card component
// ---------------------------------------------------------------------------

#[component]
fn ProcessCard(input_diagram: Signal<InputDiagram<'static>>, entry: ProcessEntry) -> Element {
    let pid_str = entry.process_id.clone();

    rsx! {
        div {
            class: CARD_CLASS,

            // ── Header: Process ID + Remove ──────────────────────────
            div {
                class: ROW_CLASS,

                label {
                    class: "text-xs text-gray-500 w-20",
                    "Process ID"
                }
                input {
                    class: INPUT_CLASS,
                    style: "max-width:16rem",
                    list: list_ids::PROCESS_IDS,
                    placeholder: "process_id",
                    value: "{pid_str}",
                    onchange: {
                        let old_id = pid_str.clone();
                        move |evt: dioxus::events::FormEvent| {
                            rename_process(input_diagram, &old_id, &evt.value());
                        }
                    },
                }

                span {
                    class: REMOVE_BTN,
                    onclick: {
                        let id = pid_str.clone();
                        move |_| {
                            remove_process(input_diagram, &id);
                        }
                    },
                    "✕ Remove"
                }
            }

            // ── Name ─────────────────────────────────────────────────
            div {
                class: ROW_CLASS,

                label {
                    class: "text-xs text-gray-500 w-20",
                    "Name"
                }
                input {
                    class: INPUT_CLASS,
                    placeholder: "Display name",
                    value: "{entry.name}",
                    oninput: {
                        let id = pid_str.clone();
                        move |evt: dioxus::events::FormEvent| {
                            update_process_name(input_diagram, &id, &evt.value());
                        }
                    },
                }
            }

            // ── Description ──────────────────────────────────────────
            div {
                class: ROW_CLASS,

                label {
                    class: "text-xs text-gray-500 w-20",
                    "Description"
                }
                textarea {
                    class: "\
                        flex-1 \
                        rounded \
                        border \
                        border-gray-600 \
                        bg-gray-800 \
                        text-gray-200 \
                        px-2 py-1 \
                        text-sm \
                        font-mono \
                        min-h-12 \
                        focus:border-blue-400 \
                        focus:outline-none\
                    ",
                    placeholder: "Process description (markdown)",
                    value: "{entry.desc}",
                    oninput: {
                        let id = pid_str.clone();
                        move |evt: dioxus::events::FormEvent| {
                            update_process_desc(input_diagram, &id, &evt.value());
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
                        let pid = pid_str.clone();
                        rsx! {
                            div {
                                key: "{pid}_{step_id}",
                                class: ROW_CLASS,

                                input {
                                    class: INPUT_CLASS,
                                    style: "max-width:14rem",
                                    list: list_ids::PROCESS_STEP_IDS,
                                    placeholder: "step_id",
                                    value: "{step_id}",
                                    onchange: {
                                        let pid2 = pid.clone();
                                        let old_sid = step_id.clone();
                                        let label = step_label.clone();
                                        move |evt: dioxus::events::FormEvent| {
                                            rename_step(input_diagram, &pid2, &old_sid, &evt.value(), &label);
                                        }
                                    },
                                }

                                input {
                                    class: INPUT_CLASS,
                                    placeholder: "Step label",
                                    value: "{step_label}",
                                    oninput: {
                                        let pid2 = pid.clone();
                                        let sid = step_id.clone();
                                        move |evt: dioxus::events::FormEvent| {
                                            update_step_label(input_diagram, &pid2, &sid, &evt.value());
                                        }
                                    },
                                }

                                span {
                                    class: REMOVE_BTN,
                                    onclick: {
                                        let pid2 = pid.clone();
                                        let sid = step_id.clone();
                                        move |_| {
                                            remove_step(input_diagram, &pid2, &sid);
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
                        let pid = pid_str.clone();
                        move |_| {
                            add_step(input_diagram, &pid);
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
                        let pid = pid_str.clone();
                        rsx! {
                            StepInteractionCard {
                                key: "{pid}_sti_{step_id}",
                                input_diagram,
                                process_id: pid,
                                step_id,
                                edge_ids,
                            }
                        }
                    }
                }

                div {
                    class: ADD_BTN,
                    onclick: {
                        let pid = pid_str.clone();
                        move |_| {
                            add_step_interaction(input_diagram, &pid);
                        }
                    },
                    "+ Add step interaction mapping"
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Step interaction card
// ---------------------------------------------------------------------------

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
                class: ROW_CLASS,

                input {
                    class: INPUT_CLASS,
                    style: "max-width:14rem",
                    list: list_ids::PROCESS_STEP_IDS,
                    placeholder: "step_id",
                    value: "{step_id}",
                    onchange: {
                        let pid = process_id.clone();
                        let old_sid = step_id.clone();
                        let eids = edge_ids.clone();
                        move |evt: dioxus::events::FormEvent| {
                            rename_step_interaction(input_diagram, &pid, &old_sid, &evt.value(), &eids);
                        }
                    },
                }

                span {
                    class: REMOVE_BTN,
                    onclick: {
                        let pid = process_id.clone();
                        let sid = step_id.clone();
                        move |_| {
                            remove_step_interaction(input_diagram, &pid, &sid);
                        }
                    },
                    "✕"
                }
            }

            // Edge group IDs
            div {
                class: "flex flex-col gap-1 pl-4",

                for (idx, eid) in edge_ids.iter().enumerate() {
                    {
                        let eid = eid.clone();
                        let pid = process_id.clone();
                        let sid = step_id.clone();
                        rsx! {
                            div {
                                key: "{pid}_{sid}_{idx}",
                                class: ROW_CLASS,

                                span {
                                    class: "text-xs text-gray-500 w-6 text-right",
                                    "{idx}."
                                }

                                input {
                                    class: INPUT_CLASS,
                                    style: "max-width:14rem",
                                    list: list_ids::EDGE_GROUP_IDS,
                                    placeholder: "edge_group_id",
                                    value: "{eid}",
                                    onchange: {
                                        let pid2 = pid.clone();
                                        let sid2 = sid.clone();
                                        move |evt: dioxus::events::FormEvent| {
                                            update_edge_in_step_interaction(
                                                input_diagram, &pid2, &sid2, idx, &evt.value(),
                                            );
                                        }
                                    },
                                }

                                span {
                                    class: REMOVE_BTN,
                                    onclick: {
                                        let pid2 = pid.clone();
                                        let sid2 = sid.clone();
                                        move |_| {
                                            remove_edge_from_step_interaction(
                                                input_diagram, &pid2, &sid2, idx,
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
                        let pid = process_id.clone();
                        let sid = step_id.clone();
                        move |_| {
                            add_edge_to_step_interaction(input_diagram, &pid, &sid);
                        }
                    },
                    "+ Add edge group"
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// ID parsers
// ---------------------------------------------------------------------------

fn parse_process_id(s: &str) -> Option<ProcessId<'static>> {
    Id::new(s).ok().map(|id| ProcessId::from(id.into_static()))
}

fn parse_process_step_id(s: &str) -> Option<ProcessStepId<'static>> {
    Id::new(s)
        .ok()
        .map(|id| ProcessStepId::from(id.into_static()))
}

fn parse_edge_group_id(s: &str) -> Option<EdgeGroupId<'static>> {
    EdgeGroupId::new(s).ok().map(|id| id.into_static())
}

// ---------------------------------------------------------------------------
// Mutation helpers: processes
// ---------------------------------------------------------------------------

fn add_process(mut diag: Signal<InputDiagram<'static>>) {
    let mut n = diag.read().processes.len();
    loop {
        let candidate = format!("proc_{n}");
        if let Some(pid) = parse_process_id(&candidate) {
            if !diag.read().processes.contains_key(&pid) {
                diag.write()
                    .processes
                    .insert(pid, ProcessDiagram::default());
                break;
            }
        }
        n += 1;
    }
}

fn remove_process(mut diag: Signal<InputDiagram<'static>>, id: &str) {
    if let Some(pid) = parse_process_id(id) {
        diag.write().processes.swap_remove(&pid);
    }
}

fn rename_process(mut diag: Signal<InputDiagram<'static>>, old_id: &str, new_id: &str) {
    if old_id == new_id {
        return;
    }
    let old = match parse_process_id(old_id) {
        Some(id) => id,
        None => return,
    };
    let new = match parse_process_id(new_id) {
        Some(id) => id,
        None => return,
    };
    let mut d = diag.write();
    if let Some(proc_diag) = d.processes.swap_remove(&old) {
        d.processes.insert(new, proc_diag);
    }
}

fn update_process_name(mut diag: Signal<InputDiagram<'static>>, id: &str, name: &str) {
    if let Some(pid) = parse_process_id(id) {
        if let Some(proc) = diag.write().processes.get_mut(&pid) {
            proc.name = if name.is_empty() {
                None
            } else {
                Some(name.to_owned())
            };
        }
    }
}

fn update_process_desc(mut diag: Signal<InputDiagram<'static>>, id: &str, desc: &str) {
    if let Some(pid) = parse_process_id(id) {
        if let Some(proc) = diag.write().processes.get_mut(&pid) {
            proc.desc = if desc.is_empty() {
                None
            } else {
                Some(desc.to_owned())
            };
        }
    }
}

// ---------------------------------------------------------------------------
// Mutation helpers: steps
// ---------------------------------------------------------------------------

fn add_step(mut diag: Signal<InputDiagram<'static>>, process_id: &str) {
    let pid = match parse_process_id(process_id) {
        Some(id) => id,
        None => return,
    };
    let d = diag.read();
    let proc = match d.processes.get(&pid) {
        Some(p) => p,
        None => return,
    };
    let mut n = proc.steps.len();
    loop {
        let candidate = format!("{process_id}_step_{n}");
        if let Some(sid) = parse_process_step_id(&candidate) {
            if !proc.steps.contains_key(&sid) {
                drop(d);
                if let Some(proc) = diag.write().processes.get_mut(&pid) {
                    proc.steps.insert(sid, String::new());
                }
                break;
            }
        }
        n += 1;
    }
}

fn remove_step(mut diag: Signal<InputDiagram<'static>>, process_id: &str, step_id: &str) {
    let pid = match parse_process_id(process_id) {
        Some(id) => id,
        None => return,
    };
    let sid = match parse_process_step_id(step_id) {
        Some(id) => id,
        None => return,
    };
    if let Some(proc) = diag.write().processes.get_mut(&pid) {
        proc.steps.swap_remove(&sid);
    }
}

fn rename_step(
    mut diag: Signal<InputDiagram<'static>>,
    process_id: &str,
    old_step_id: &str,
    new_step_id: &str,
    current_label: &str,
) {
    if old_step_id == new_step_id {
        return;
    }
    let pid = match parse_process_id(process_id) {
        Some(id) => id,
        None => return,
    };
    let old = match parse_process_step_id(old_step_id) {
        Some(id) => id,
        None => return,
    };
    let new = match parse_process_step_id(new_step_id) {
        Some(id) => id,
        None => return,
    };
    if let Some(proc) = diag.write().processes.get_mut(&pid) {
        proc.steps.swap_remove(&old);
        proc.steps.insert(new, current_label.to_owned());
    }
}

fn update_step_label(
    mut diag: Signal<InputDiagram<'static>>,
    process_id: &str,
    step_id: &str,
    label: &str,
) {
    let pid = match parse_process_id(process_id) {
        Some(id) => id,
        None => return,
    };
    let sid = match parse_process_step_id(step_id) {
        Some(id) => id,
        None => return,
    };
    if let Some(proc) = diag.write().processes.get_mut(&pid) {
        if let Some(entry) = proc.steps.get_mut(&sid) {
            *entry = label.to_owned();
        }
    }
}

// ---------------------------------------------------------------------------
// Mutation helpers: step thing interactions
// ---------------------------------------------------------------------------

fn add_step_interaction(mut diag: Signal<InputDiagram<'static>>, process_id: &str) {
    let pid = match parse_process_id(process_id) {
        Some(id) => id,
        None => return,
    };
    let d = diag.read();
    let proc = match d.processes.get(&pid) {
        Some(p) => p,
        None => return,
    };

    // Pick the first step that doesn't already have an interaction mapping,
    // or fall back to a placeholder.
    let step_id = proc
        .steps
        .keys()
        .find(|sid| !proc.step_thing_interactions.contains_key(*sid))
        .cloned();

    let step_id = match step_id {
        Some(sid) => sid,
        None => {
            // All steps already have mappings; generate a placeholder.
            let mut n = proc.step_thing_interactions.len();
            loop {
                let candidate = format!("{process_id}_step_{n}");
                if let Some(sid) = parse_process_step_id(&candidate) {
                    if !proc.step_thing_interactions.contains_key(&sid) {
                        drop(d);
                        if let Some(proc) = diag.write().processes.get_mut(&pid) {
                            proc.step_thing_interactions.insert(sid, Vec::new());
                        }
                        return;
                    }
                }
                n += 1;
            }
        }
    };

    drop(d);
    if let Some(proc) = diag.write().processes.get_mut(&pid) {
        proc.step_thing_interactions.insert(step_id, Vec::new());
    }
}

fn remove_step_interaction(
    mut diag: Signal<InputDiagram<'static>>,
    process_id: &str,
    step_id: &str,
) {
    let pid = match parse_process_id(process_id) {
        Some(id) => id,
        None => return,
    };
    let sid = match parse_process_step_id(step_id) {
        Some(id) => id,
        None => return,
    };
    if let Some(proc) = diag.write().processes.get_mut(&pid) {
        proc.step_thing_interactions.swap_remove(&sid);
    }
}

fn rename_step_interaction(
    mut diag: Signal<InputDiagram<'static>>,
    process_id: &str,
    old_step_id: &str,
    new_step_id: &str,
    edge_ids: &[String],
) {
    if old_step_id == new_step_id {
        return;
    }
    let pid = match parse_process_id(process_id) {
        Some(id) => id,
        None => return,
    };
    let old = match parse_process_step_id(old_step_id) {
        Some(id) => id,
        None => return,
    };
    let new = match parse_process_step_id(new_step_id) {
        Some(id) => id,
        None => return,
    };
    let eids: Vec<EdgeGroupId<'static>> = edge_ids
        .iter()
        .filter_map(|s| parse_edge_group_id(s))
        .collect();
    if let Some(proc) = diag.write().processes.get_mut(&pid) {
        proc.step_thing_interactions.swap_remove(&old);
        proc.step_thing_interactions.insert(new, eids);
    }
}

fn update_edge_in_step_interaction(
    mut diag: Signal<InputDiagram<'static>>,
    process_id: &str,
    step_id: &str,
    idx: usize,
    new_edge_str: &str,
) {
    let pid = match parse_process_id(process_id) {
        Some(id) => id,
        None => return,
    };
    let sid = match parse_process_step_id(step_id) {
        Some(id) => id,
        None => return,
    };
    let new_eid = match parse_edge_group_id(new_edge_str) {
        Some(id) => id,
        None => return,
    };
    if let Some(proc) = diag.write().processes.get_mut(&pid) {
        if let Some(eids) = proc.step_thing_interactions.get_mut(&sid) {
            if idx < eids.len() {
                eids[idx] = new_eid;
            }
        }
    }
}

fn remove_edge_from_step_interaction(
    mut diag: Signal<InputDiagram<'static>>,
    process_id: &str,
    step_id: &str,
    idx: usize,
) {
    let pid = match parse_process_id(process_id) {
        Some(id) => id,
        None => return,
    };
    let sid = match parse_process_step_id(step_id) {
        Some(id) => id,
        None => return,
    };
    if let Some(proc) = diag.write().processes.get_mut(&pid) {
        if let Some(eids) = proc.step_thing_interactions.get_mut(&sid) {
            if idx < eids.len() {
                eids.remove(idx);
            }
        }
    }
}

fn add_edge_to_step_interaction(
    mut diag: Signal<InputDiagram<'static>>,
    process_id: &str,
    step_id: &str,
) {
    let pid = match parse_process_id(process_id) {
        Some(id) => id,
        None => return,
    };
    let sid = match parse_process_step_id(step_id) {
        Some(id) => id,
        None => return,
    };

    // Pick the first edge group id from thing_interactions as a placeholder.
    let placeholder = {
        let d = diag.read();
        d.thing_interactions
            .keys()
            .next()
            .map(|e| e.as_str().to_owned())
            .unwrap_or_else(|| "edge_0".to_owned())
    };
    let new_eid = match parse_edge_group_id(&placeholder) {
        Some(id) => id,
        None => return,
    };

    if let Some(proc) = diag.write().processes.get_mut(&pid) {
        if let Some(eids) = proc.step_thing_interactions.get_mut(&sid) {
            eids.push(new_eid);
        }
    }
}
