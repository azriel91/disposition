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
