//! Mutation operations for the Processes editor page.
//!
//! Grouped here so that related functions are discoverable when sorted by
//! name, per the project's `noun_verb` naming convention.

use dioxus::signals::{ReadableExt, Signal, WritableExt};
use disposition::input_model::{process::ProcessDiagram, InputDiagram};

use crate::components::editor::common::{id_rename_in_input_diagram, parse_process_id};

/// Mutation operations for the Processes editor page.
pub(crate) struct ProcessesPageOps;

impl ProcessesPageOps {
    /// Adds a new process with a unique placeholder ProcessId.
    pub(crate) fn process_add(mut input_diagram: Signal<InputDiagram<'static>>) {
        let mut n = input_diagram.read().processes.len();
        loop {
            let candidate = format!("proc_{n}");
            if let Some(process_id) = parse_process_id(&candidate)
                && !input_diagram.read().processes.contains_key(&process_id)
            {
                input_diagram
                    .write()
                    .processes
                    .insert(process_id, ProcessDiagram::default());
                break;
            }
            n += 1;
        }
    }

    /// Removes a process from the `processes` map.
    pub(crate) fn process_remove(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
    ) {
        if let Some(process_id) = parse_process_id(process_id_str) {
            input_diagram.write().processes.shift_remove(&process_id);
        }
    }

    /// Renames a process across all maps in the [`InputDiagram`].
    pub(crate) fn process_rename(
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
    pub(crate) fn process_name_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
        name: &str,
    ) {
        if let Some(process_id) = parse_process_id(process_id_str)
            && let Some(process_diagram) = input_diagram.write().processes.get_mut(&process_id)
        {
            process_diagram.name = if name.is_empty() {
                None
            } else {
                Some(name.to_owned())
            };
        }
    }

    /// Updates the description for an existing process.
    pub(crate) fn process_desc_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
        desc: &str,
    ) {
        if let Some(process_id) = parse_process_id(process_id_str)
            && let Some(process_diagram) = input_diagram.write().processes.get_mut(&process_id)
        {
            process_diagram.desc = if desc.is_empty() {
                None
            } else {
                Some(desc.to_owned())
            };
        }
    }
}
