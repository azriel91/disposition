//! Mutation operations for the step interaction card component.
//!
//! This module is a thin Signal-aware wrapper around
//! [`disposition_input_rt::step_interaction_card_ops::StepInteractionCardOps`].
//! Each method acquires a read or write guard on the [`Signal`] and delegates
//! to the framework-agnostic implementation.

use dioxus::signals::{Signal, WritableExt};
use disposition::input_model::InputDiagram;

/// Mutation operations for the step interaction card component.
pub(crate) struct StepInteractionCardOps;

impl StepInteractionCardOps {
    /// Removes a step interaction mapping from a process.
    pub(crate) fn step_interaction_remove(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
        step_id_str: &str,
    ) {
        disposition_input_rt::step_interaction_card_ops::StepInteractionCardOps::step_interaction_remove(
            &mut input_diagram.write(),
            process_id_str,
            step_id_str,
        );
    }

    /// Renames the step key of a step interaction mapping.
    pub(crate) fn step_interaction_rename(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
        step_id_old_str: &str,
        step_id_new_str: &str,
        edge_id_strs: &[String],
    ) {
        disposition_input_rt::step_interaction_card_ops::StepInteractionCardOps::step_interaction_rename(
            &mut input_diagram.write(),
            process_id_str,
            step_id_old_str,
            step_id_new_str,
            edge_id_strs,
        );
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
        disposition_input_rt::step_interaction_card_ops::StepInteractionCardOps::step_interaction_edge_update(
            &mut input_diagram.write(),
            process_id_str,
            step_id_str,
            idx,
            edge_group_id_new_str,
        );
    }

    /// Removes an edge group from a step interaction by index.
    pub(crate) fn step_interaction_edge_remove(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
        step_id_str: &str,
        idx: usize,
    ) {
        disposition_input_rt::step_interaction_card_ops::StepInteractionCardOps::step_interaction_edge_remove(
            &mut input_diagram.write(),
            process_id_str,
            step_id_str,
            idx,
        );
    }

    /// Adds an edge group to a step interaction, using the first existing
    /// interaction edge group ID as a placeholder.
    pub(crate) fn step_interaction_edge_add(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
        step_id_str: &str,
    ) {
        disposition_input_rt::step_interaction_card_ops::StepInteractionCardOps::step_interaction_edge_add(
            &mut input_diagram.write(),
            process_id_str,
            step_id_str,
        );
    }
}
