use disposition_input_model::DiagramFocus;
use disposition_ir_model::node::NodeId;
use disposition_model_common::Set;

/// Heights for all steps within a process for y-coordinate calculations.
///
/// These are used to collapse processes to reduce the number of steps
/// displayed.
#[derive(Debug)]
pub(crate) struct ProcessStepsHeight<'id> {
    /// The node ID of the process.
    pub(crate) process_id: NodeId<'id>,
    /// List of process step node IDs belonging to this process.
    pub(crate) process_step_ids: Set<NodeId<'id>>,
    /// Total height of all process steps belonging to this process.
    pub(crate) total_height: f32,
}

/// Computes the cumulative height of steps from all processes before the
/// given process index.
pub(crate) fn predecessors_cumulative_height(
    process_steps_heights: &[ProcessStepsHeight],
    process_index: usize,
) -> f32 {
    process_steps_heights
        .iter()
        .take(process_index)
        .map(|p| p.total_height)
        .sum()
}

/// Returns whether `active` focuses the given process directly, or via one
/// of its steps.
pub(crate) fn focus_active_targets_process<'id>(
    active: &DiagramFocus<'id>,
    process_steps_height: &ProcessStepsHeight<'id>,
) -> bool {
    let ProcessStepsHeight {
        process_id,
        process_step_ids,
        ..
    } = process_steps_height;
    match active {
        DiagramFocus::Process(active_process_id) => {
            active_process_id.as_ref() == process_id.as_ref()
        }
        DiagramFocus::ProcessStep {
            process_step_id: active_step_id,
            ..
        } => process_step_ids
            .iter()
            .any(|process_step_id| process_step_id.as_ref() == active_step_id.as_ref()),
        DiagramFocus::None | DiagramFocus::Tag(_) => false,
    }
}
