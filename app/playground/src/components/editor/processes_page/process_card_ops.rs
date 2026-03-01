//! Mutation operations for the process card component.
//!
//! Grouped here so that related functions are discoverable when sorted by
//! name, per the project's `noun_verb` naming convention.

use dioxus::signals::{ReadableExt, Signal, WritableExt};
use disposition::input_model::InputDiagram;

use crate::components::editor::common::{
    id_rename_in_input_diagram, parse_process_id, parse_process_step_id,
};

/// Mutation operations for the process card component.
pub(crate) struct ProcessCardOps;

impl ProcessCardOps {
    // === Step helpers === //

    /// Adds a new step to a process with a unique placeholder step ID.
    pub(crate) fn step_add(mut input_diagram: Signal<InputDiagram<'static>>, process_id_str: &str) {
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
            if let Some(step_id) = parse_process_step_id(&candidate)
                && !process_diagram.steps.contains_key(&step_id)
            {
                drop(input_diagram_read);
                if let Some(process_diagram) = input_diagram.write().processes.get_mut(&process_id)
                {
                    process_diagram.steps.insert(step_id, String::new());
                }
                break;
            }
            n += 1;
        }
    }

    /// Removes a step from a process.
    pub(crate) fn step_remove(
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
            process_diagram.steps.shift_remove(&step_id);
        }
    }

    /// Renames a step across all processes and shared maps in the
    /// [`InputDiagram`].
    pub(crate) fn step_rename(
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
    pub(crate) fn step_label_update(
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
        if let Some(process_diagram) = input_diagram.write().processes.get_mut(&process_id)
            && let Some(entry) = process_diagram.steps.get_mut(&step_id)
        {
            *entry = label.to_owned();
        }
    }

    // === Step interaction helpers === //

    /// Adds a new step interaction mapping to a process.
    pub(crate) fn step_interaction_add(
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
                    if let Some(step_id) = parse_process_step_id(&candidate)
                        && !process_diagram
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
