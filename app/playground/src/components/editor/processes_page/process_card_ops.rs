//! Mutation operations for the process card component.
//!
//! This module is a thin Signal-aware wrapper around
//! [`disposition_input_rt::process_card_ops::ProcessCardOps`]. Each method
//! acquires a read or write guard on the [`Signal`] and delegates to the
//! framework-agnostic implementation.

use dioxus::signals::{Signal, WritableExt};
use disposition::input_model::InputDiagram;

/// Mutation operations for the process card component.
pub(crate) struct ProcessCardOps;

impl ProcessCardOps {
    // === Step helpers === //

    /// Moves a step within a process from one index to another.
    ///
    /// Uses `IndexMap::move_index` on the process's `steps` map to
    /// reposition the entry while preserving all other entries.
    ///
    /// # Parameters
    ///
    /// * `input_diagram`: the diagram signal to mutate.
    /// * `process_id_str`: the process ID string, e.g. `"proc_app_dev"`.
    /// * `from`: the current index of the step.
    /// * `to`: the desired index of the step.
    pub(crate) fn step_move(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
        from: usize,
        to: usize,
    ) {
        disposition_input_rt::process_card_ops::ProcessCardOps::step_move(
            &mut input_diagram.write(),
            process_id_str,
            from,
            to,
        );
    }

    /// Adds a new step to a process with a unique placeholder step ID.
    pub(crate) fn step_add(mut input_diagram: Signal<InputDiagram<'static>>, process_id_str: &str) {
        disposition_input_rt::process_card_ops::ProcessCardOps::step_add(
            &mut input_diagram.write(),
            process_id_str,
        );
    }

    /// Removes a step from a process.
    pub(crate) fn step_remove(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
        step_id_str: &str,
    ) {
        disposition_input_rt::process_card_ops::ProcessCardOps::step_remove(
            &mut input_diagram.write(),
            process_id_str,
            step_id_str,
        );
    }

    /// Renames a step across all processes and shared maps in the
    /// [`InputDiagram`].
    pub(crate) fn step_rename(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
        step_id_old_str: &str,
        step_id_new_str: &str,
    ) {
        disposition_input_rt::process_card_ops::ProcessCardOps::step_rename(
            &mut input_diagram.write(),
            process_id_str,
            step_id_old_str,
            step_id_new_str,
        );
    }

    /// Updates the label for an existing step.
    pub(crate) fn step_label_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
        step_id_str: &str,
        label: &str,
    ) {
        disposition_input_rt::process_card_ops::ProcessCardOps::step_label_update(
            &mut input_diagram.write(),
            process_id_str,
            step_id_str,
            label,
        );
    }

    // === Step interaction helpers === //

    /// Adds a new step interaction mapping to a process.
    pub(crate) fn step_interaction_add(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
    ) {
        disposition_input_rt::process_card_ops::ProcessCardOps::step_interaction_add(
            &mut input_diagram.write(),
            process_id_str,
        );
    }
}
