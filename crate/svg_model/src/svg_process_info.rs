use disposition_ir_model::node::NodeId;
use serde::{Deserialize, Serialize};

/// Information for animating process node expansion.
///
/// When a process or any of its steps are focused, the process node
/// expands to show all steps. This struct contains the information needed
/// to generate the CSS classes for this animation.
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct SvgProcessInfo<'id> {
    /// The height to expand to when the process is focused.
    pub height_to_expand_to: f32,
    /// The path `d` attribute for the expanded state.
    pub path_d_expanded: String,
    /// The node ID of the process (used in CSS selectors).
    pub process_id: NodeId<'id>,
    /// List of process step node IDs (used in CSS selectors).
    pub process_step_ids: Vec<NodeId<'id>>,
    /// The index of this process in the list of all processes.
    /// Used for calculating y-translations when previous processes expand.
    pub process_index: usize,
    /// Total height of all steps in this process (used for y-translation
    /// calculations).
    pub total_height: f32,
    /// Base y position for the collapsed state.
    pub base_y: f32,
}

impl<'id> SvgProcessInfo<'id> {
    /// Creates a new `SvgProcessInfo`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        height_to_expand_to: f32,
        path_d_expanded: String,
        process_id: NodeId<'id>,
        process_step_ids: Vec<NodeId<'id>>,
        process_index: usize,
        total_height: f32,
        base_y: f32,
    ) -> Self {
        Self {
            height_to_expand_to,
            path_d_expanded,
            process_id,
            process_step_ids,
            process_index,
            total_height,
            base_y,
        }
    }
}
