//! Mutation operations for the Processes editor page.
//!
//! Grouped here so that related functions are discoverable when sorted by
//! name, per the project's `noun_verb` naming convention.
//!
//! All methods operate on `&mut InputDiagram<'static>` instead of a
//! framework-specific signal type, making them testable without a UI runtime.

use disposition_input_model::{process::ProcessDiagram, InputDiagram};

use crate::{id_parse::parse_process_id, id_rename::id_rename_in_input_diagram};

/// Mutation operations for the Processes editor page.
pub struct ProcessesPageOps;

impl ProcessesPageOps {
    /// Moves a process from one index to another in the `processes` map.
    pub fn process_move(input_diagram: &mut InputDiagram<'static>, from: usize, to: usize) {
        input_diagram.processes.move_index(from, to);
    }

    /// Adds a new process with a unique placeholder ProcessId.
    pub fn process_add(input_diagram: &mut InputDiagram<'static>) {
        let mut n = input_diagram.processes.len();
        loop {
            let candidate = format!("proc_{n}");
            if let Some(process_id) = parse_process_id(&candidate)
                && !input_diagram.processes.contains_key(&process_id)
            {
                input_diagram
                    .processes
                    .insert(process_id, ProcessDiagram::default());
                break;
            }
            n += 1;
        }
    }

    /// Removes a process from the `processes` map.
    pub fn process_remove(input_diagram: &mut InputDiagram<'static>, process_id_str: &str) {
        if let Some(process_id) = parse_process_id(process_id_str) {
            input_diagram.processes.remove(&process_id);
        }
    }

    /// Renames a process across all maps in the [`InputDiagram`].
    pub fn process_rename(
        input_diagram: &mut InputDiagram<'static>,
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

        // processes: rename ProcessId key.
        if let Some(index) = input_diagram.processes.get_index_of(&process_id_old) {
            let _result = input_diagram
                .processes
                .replace_index(index, process_id_new.clone());
        }

        // Shared rename across entity_descs, entity_tooltips, entity_types,
        // and all theme style maps.
        let id_old = process_id_old.into_inner();
        let id_new = process_id_new.into_inner();
        id_rename_in_input_diagram(input_diagram, &id_old, &id_new);
    }

    /// Updates the display name for an existing process.
    pub fn process_name_update(
        input_diagram: &mut InputDiagram<'static>,
        process_id_str: &str,
        name: &str,
    ) {
        if let Some(process_id) = parse_process_id(process_id_str)
            && let Some(process_diagram) = input_diagram.processes.get_mut(&process_id)
        {
            process_diagram.name = if name.is_empty() {
                None
            } else {
                Some(name.to_owned())
            };
        }
    }

    /// Updates the description for an existing process.
    pub fn process_desc_update(
        input_diagram: &mut InputDiagram<'static>,
        process_id_str: &str,
        desc: &str,
    ) {
        if let Some(process_id) = parse_process_id(process_id_str)
            && let Some(process_diagram) = input_diagram.processes.get_mut(&process_id)
        {
            process_diagram.desc = if desc.is_empty() {
                None
            } else {
                Some(desc.to_owned())
            };
        }
    }
}
