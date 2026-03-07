//! Mutation operations for the Processes editor page.
//!
//! This module is a thin Signal-aware wrapper around
//! [`disposition_input_rt::processes_page_ops::ProcessesPageOps`]. Each method
//! acquires a read or write guard on the [`Signal`] and delegates to the
//! framework-agnostic implementation.

use dioxus::signals::{Signal, WritableExt};
use disposition::input_model::InputDiagram;

/// Mutation operations for the Processes editor page.
pub(crate) struct ProcessesPageOps;

impl ProcessesPageOps {
    /// Moves a process from one index to another in the `processes` map.
    pub(crate) fn process_move(
        mut input_diagram: Signal<InputDiagram<'static>>,
        from: usize,
        to: usize,
    ) {
        disposition_input_rt::processes_page_ops::ProcessesPageOps::process_move(
            &mut input_diagram.write(),
            from,
            to,
        );
    }

    /// Adds a new process with a unique placeholder ProcessId.
    pub(crate) fn process_add(mut input_diagram: Signal<InputDiagram<'static>>) {
        disposition_input_rt::processes_page_ops::ProcessesPageOps::process_add(
            &mut input_diagram.write(),
        );
    }

    /// Removes a process from the `processes` map.
    pub(crate) fn process_remove(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
    ) {
        disposition_input_rt::processes_page_ops::ProcessesPageOps::process_remove(
            &mut input_diagram.write(),
            process_id_str,
        );
    }

    /// Renames a process across all maps in the [`InputDiagram`].
    pub(crate) fn process_rename(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_old_str: &str,
        process_id_new_str: &str,
    ) {
        disposition_input_rt::processes_page_ops::ProcessesPageOps::process_rename(
            &mut input_diagram.write(),
            process_id_old_str,
            process_id_new_str,
        );
    }

    /// Updates the display name for an existing process.
    pub(crate) fn process_name_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
        name: &str,
    ) {
        disposition_input_rt::processes_page_ops::ProcessesPageOps::process_name_update(
            &mut input_diagram.write(),
            process_id_str,
            name,
        );
    }

    /// Updates the description for an existing process.
    pub(crate) fn process_desc_update(
        mut input_diagram: Signal<InputDiagram<'static>>,
        process_id_str: &str,
        desc: &str,
    ) {
        disposition_input_rt::processes_page_ops::ProcessesPageOps::process_desc_update(
            &mut input_diagram.write(),
            process_id_str,
            desc,
        );
    }
}
