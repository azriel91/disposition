//! Mutation operations for step dependency card entries.
//!
//! Grouped here so that related functions are discoverable when sorted by
//! name, per the project's `noun_verb` naming convention.
//!
//! All methods operate on `&mut InputDiagram<'static>` instead of a
//! framework-specific signal type, making them testable without a UI runtime.
//!
//! Each process step maps to the `Set` of process steps it depends on. These
//! operations edit the `process_step_dependencies` map of a single
//! `ProcessDiagram`.

use disposition_input_model::{process::ProcessStepId, InputDiagram};
use disposition_model_common::{Set, SetOrderedRemove};

use crate::id_parse::{parse_process_id, parse_process_step_id};

/// Mutation operations for step dependency card entries.
pub struct StepDependencyCardOps;

impl StepDependencyCardOps {
    /// Removes a step's dependency entry from a process.
    pub fn step_dependency_remove(
        input_diagram: &mut InputDiagram<'static>,
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
        if let Some(process_diagram) = input_diagram.processes.get_mut(&process_id) {
            process_diagram
                .process_step_dependencies
                .swap_remove(&step_id);
        }
    }

    /// Renames the step key of a dependency entry, preserving its dependency
    /// set.
    pub fn step_dependency_rename(
        input_diagram: &mut InputDiagram<'static>,
        process_id_str: &str,
        step_id_old_str: &str,
        step_id_new_str: &str,
        dep_id_strs: &[String],
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
        let dep_ids: Set<ProcessStepId<'static>> = dep_id_strs
            .iter()
            .filter_map(|s| parse_process_step_id(s))
            .collect();
        if let Some(process_diagram) = input_diagram.processes.get_mut(&process_id) {
            process_diagram
                .process_step_dependencies
                .insert(step_id_new, dep_ids);
            process_diagram
                .process_step_dependencies
                .swap_remove(&step_id_old);
        }
    }

    /// Updates a single dependency step ID within a step's dependency set at
    /// the given index.
    pub fn step_dependency_dep_update(
        input_diagram: &mut InputDiagram<'static>,
        process_id_str: &str,
        step_id_str: &str,
        idx: usize,
        dep_id_new_str: &str,
    ) {
        let process_id = match parse_process_id(process_id_str) {
            Some(process_id) => process_id,
            None => return,
        };
        let step_id = match parse_process_step_id(step_id_str) {
            Some(step_id) => step_id,
            None => return,
        };
        let dep_id_new = match parse_process_step_id(dep_id_new_str) {
            Some(dep_id) => dep_id,
            None => return,
        };
        if let Some(process_diagram) = input_diagram.processes.get_mut(&process_id)
            && let Some(dep_ids) = process_diagram.process_step_dependencies.get_mut(&step_id)
            && idx < dep_ids.len()
        {
            // `Set` (OrderSet) does not support indexed mutation directly.
            // Rebuild the set with the replacement at the given position.
            let mut dep_ids_new = Set::with_capacity(dep_ids.len());
            for (i, existing) in dep_ids.iter().enumerate() {
                if i == idx {
                    dep_ids_new.insert(dep_id_new.clone());
                } else {
                    dep_ids_new.insert(existing.clone());
                }
            }
            *dep_ids = dep_ids_new;
        }
    }

    /// Removes a dependency from a step's dependency set by index.
    pub fn step_dependency_dep_remove(
        input_diagram: &mut InputDiagram<'static>,
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
        if let Some(process_diagram) = input_diagram.processes.get_mut(&process_id)
            && let Some(dep_ids) = process_diagram.process_step_dependencies.get_mut(&step_id)
            && idx < dep_ids.len()
        {
            dep_ids.remove_index_ordered(idx);
        }
    }

    /// Moves a dependency within a step's dependency set from one index to
    /// another.
    ///
    /// Uses `OrderSet::move_index` to reposition the entry while preserving all
    /// other entries.
    ///
    /// # Parameters
    ///
    /// * `input_diagram`: the diagram to mutate.
    /// * `process_id_str`: the process ID string, e.g. `"proc_app_dev"`.
    /// * `step_id_str`: the step ID string, e.g. `"proc_app_dev_step_build"`.
    /// * `from`: the current index of the dependency to move.
    /// * `to`: the target index.
    pub fn step_dependency_dep_move(
        input_diagram: &mut InputDiagram<'static>,
        process_id_str: &str,
        step_id_str: &str,
        from: usize,
        to: usize,
    ) {
        let process_id = match parse_process_id(process_id_str) {
            Some(process_id) => process_id,
            None => return,
        };
        let step_id = match parse_process_step_id(step_id_str) {
            Some(step_id) => step_id,
            None => return,
        };
        if let Some(process_diagram) = input_diagram.processes.get_mut(&process_id)
            && let Some(dep_ids) = process_diagram.process_step_dependencies.get_mut(&step_id)
            && from < dep_ids.len()
            && to < dep_ids.len()
        {
            dep_ids.move_index(from, to);
        }
    }

    /// Adds a dependency to a step's dependency set, using the first process
    /// step that is neither the step itself nor already a dependency as a
    /// placeholder.
    pub fn step_dependency_dep_add(
        input_diagram: &mut InputDiagram<'static>,
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

        let process_diagram = match input_diagram.processes.get(&process_id) {
            Some(process_diagram) => process_diagram,
            None => return,
        };

        let existing_deps = process_diagram.process_step_dependencies.get(&step_id);

        // Pick the first step in the process that is neither the dependent step
        // itself nor already a dependency, falling back to the first step, then
        // to a generated placeholder.
        let dep_id_new = process_diagram
            .steps
            .keys()
            .find(|candidate| {
                **candidate != step_id
                    && existing_deps.is_none_or(|deps| !deps.contains(*candidate))
            })
            .or_else(|| process_diagram.steps.keys().next())
            .cloned();

        let dep_id_new = match dep_id_new {
            Some(dep_id) => dep_id,
            None => match parse_process_step_id(&format!("{process_id_str}_step_0")) {
                Some(dep_id) => dep_id,
                None => return,
            },
        };

        if let Some(process_diagram) = input_diagram.processes.get_mut(&process_id)
            && let Some(dep_ids) = process_diagram.process_step_dependencies.get_mut(&step_id)
        {
            dep_ids.insert(dep_id_new);
        }
    }
}
