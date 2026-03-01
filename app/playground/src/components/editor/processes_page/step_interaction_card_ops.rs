//! Mutation operations for the step interaction card component.
//!
//! Grouped here so that related functions are discoverable when sorted by
//! name, per the project's `noun_verb` naming convention.

use dioxus::signals::{ReadableExt, Signal, WritableExt};
use disposition::{input_model::InputDiagram, model_common::edge::EdgeGroupId};

use crate::components::editor::common::{
    parse_edge_group_id, parse_process_id, parse_process_step_id,
};

/// Mutation operations for the step interaction card component.
pub(crate) struct StepInteractionCardOps;

impl StepInteractionCardOps {
    /// Removes a step interaction mapping from a process.
    pub(crate) fn step_interaction_remove(
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
                .shift_remove(&step_id);
        }
    }

    /// Renames the step key of a step interaction mapping.
    pub(crate) fn step_interaction_rename(
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
                .insert(step_id_new, edge_group_ids);
            process_diagram
                .step_thing_interactions
                .swap_remove(&step_id_old);
        }
    }

    /// Updates a single edge group ID within a step interaction at the given
    /// index.
    pub(crate) fn step_interaction_edge_update(
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
        if let Some(process_diagram) = input_diagram.write().processes.get_mut(&process_id)
            && let Some(edge_group_ids) = process_diagram.step_thing_interactions.get_mut(&step_id)
            && idx < edge_group_ids.len()
        {
            edge_group_ids[idx] = edge_group_id_new;
        }
    }

    /// Removes an edge group from a step interaction by index.
    pub(crate) fn step_interaction_edge_remove(
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
        if let Some(process_diagram) = input_diagram.write().processes.get_mut(&process_id)
            && let Some(edge_group_ids) = process_diagram.step_thing_interactions.get_mut(&step_id)
            && idx < edge_group_ids.len()
        {
            edge_group_ids.remove(idx);
        }
    }

    /// Adds an edge group to a step interaction, using the first existing
    /// interaction edge group ID as a placeholder.
    pub(crate) fn step_interaction_edge_add(
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

        if let Some(process_diagram) = input_diagram.write().processes.get_mut(&process_id)
            && let Some(edge_group_ids) = process_diagram.step_thing_interactions.get_mut(&step_id)
        {
            edge_group_ids.push(edge_group_id_new);
        }
    }
}
