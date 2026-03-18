//! Mutation operations for step interaction card entries.
//!
//! Grouped here so that related functions are discoverable when sorted by
//! name, per the project's `noun_verb` naming convention.
//!
//! All methods operate on `&mut InputDiagram<'static>` instead of a
//! framework-specific signal type, making them testable without a UI runtime.

use disposition_input_model::InputDiagram;
use disposition_model_common::edge::EdgeGroupId;

use crate::id_parse::{parse_edge_group_id, parse_process_id, parse_process_step_id};

/// Mutation operations for step interaction card entries.
pub struct StepInteractionCardOps;

impl StepInteractionCardOps {
    /// Removes a step interaction mapping from a process.
    pub fn step_interaction_remove(
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
            process_diagram.step_thing_interactions.remove(&step_id);
        }
    }

    /// Renames the step key of a step interaction mapping.
    pub fn step_interaction_rename(
        input_diagram: &mut InputDiagram<'static>,
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
        if let Some(process_diagram) = input_diagram.processes.get_mut(&process_id) {
            process_diagram
                .step_thing_interactions
                .insert(step_id_new, edge_group_ids);
            process_diagram
                .step_thing_interactions
                .swap_remove(&step_id_old);
        }
    }

    /// Updates a single edge group ID within a step interaction at the given
    /// index.
    pub fn step_interaction_edge_update(
        input_diagram: &mut InputDiagram<'static>,
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
        if let Some(process_diagram) = input_diagram.processes.get_mut(&process_id)
            && let Some(edge_group_ids) = process_diagram.step_thing_interactions.get_mut(&step_id)
            && idx < edge_group_ids.len()
        {
            edge_group_ids[idx] = edge_group_id_new;
        }
    }

    /// Removes an edge group from a step interaction by index.
    pub fn step_interaction_edge_remove(
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
            && let Some(edge_group_ids) = process_diagram.step_thing_interactions.get_mut(&step_id)
            && idx < edge_group_ids.len()
        {
            edge_group_ids.remove(idx);
        }
    }

    /// Moves an edge group within a step interaction from one index to another.
    ///
    /// Uses `Vec::remove` + `Vec::insert` to reposition the entry while
    /// preserving all other entries.
    ///
    /// # Parameters
    ///
    /// * `input_diagram`: the diagram to mutate.
    /// * `process_id_str`: the process ID string, e.g. `"proc_0"`.
    /// * `step_id_str`: the step ID string, e.g. `"step_0"`.
    /// * `from`: the current index of the edge group to move.
    /// * `to`: the target index.
    pub fn step_interaction_edge_move(
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
            && let Some(edge_group_ids) = process_diagram.step_thing_interactions.get_mut(&step_id)
            && from < edge_group_ids.len()
            && to < edge_group_ids.len()
        {
            let item = edge_group_ids.remove(from);
            edge_group_ids.insert(to, item);
        }
    }

    /// Adds an edge group to a step interaction, using the first existing
    /// interaction edge group ID as a placeholder.
    pub fn step_interaction_edge_add(
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

        // Pick the first edge group id from thing_interactions as a placeholder.
        let placeholder = input_diagram
            .thing_interactions
            .keys()
            .next()
            .map(|edge_group_id| edge_group_id.as_str().to_owned())
            .unwrap_or_else(|| "edge_0".to_owned());
        let edge_group_id_new = match parse_edge_group_id(&placeholder) {
            Some(edge_group_id) => edge_group_id,
            None => return,
        };

        if let Some(process_diagram) = input_diagram.processes.get_mut(&process_id)
            && let Some(edge_group_ids) = process_diagram.step_thing_interactions.get_mut(&step_id)
        {
            edge_group_ids.push(edge_group_id_new);
        }
    }
}
