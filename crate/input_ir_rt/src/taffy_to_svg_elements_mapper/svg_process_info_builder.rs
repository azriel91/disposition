use disposition_ir_model::node::NodeId;
use disposition_svg_model::SvgProcessInfo;
use disposition_taffy_model::NodeContext;
use taffy::TaffyTree;

use super::{
    process_step_heights::{self, ProcessStepsHeight},
    svg_node_build_context::SvgProcessInfoBuildContext,
    SvgNodeRectPathBuilder,
};

/// Builds [`SvgProcessInfo`] values for process nodes in the IR diagram.
#[derive(Clone, Copy, Debug)]
pub struct SvgProcessInfoBuilder;

impl SvgProcessInfoBuilder {
    /// Returns the [`SvgProcessInfo`] for the given process IR node.
    pub(super) fn build<'ctx, 'id>(
        svg_process_info_build_context: SvgProcessInfoBuildContext<'ctx, 'id>,
        process_idx: usize,
        process_steps_height: &ProcessStepsHeight<'id>,
        process_node_id: &NodeId<'id>,
        taffy_node_id: taffy::NodeId,
        layout: &taffy::Layout,
    ) -> SvgProcessInfo<'id> {
        let SvgProcessInfoBuildContext {
            ir_diagram,
            taffy_tree,
            default_shape,
            process_steps_heights,
        } = svg_process_info_build_context;

        // Calculate y coordinate
        let y = Self::compute_absolute_y(taffy_tree, taffy_node_id, layout);

        let width = layout.size.width;
        let height_expanded = layout.size.height.min(layout.content_size.height);

        // Get the node shape (corner radii)
        let node_shape = ir_diagram
            .node_shapes
            .get(process_node_id)
            .unwrap_or(default_shape);

        let path_d_expanded = SvgNodeRectPathBuilder::build(width, height_expanded, node_shape);

        let process_steps_height_predecessors_cumulative =
            process_step_heights::predecessors_cumulative_height(
                process_steps_heights,
                process_idx,
            );
        let base_y = y - process_steps_height_predecessors_cumulative;

        SvgProcessInfo::new(
            height_expanded,
            path_d_expanded,
            process_steps_height.process_id.clone(),
            process_steps_height.process_step_ids.clone(),
            process_idx,
            process_steps_height.total_height,
            base_y,
        )
    }

    /// Computes the absolute y coordinate of a node by walking up the taffy
    /// tree and accumulating parent layout offsets.
    fn compute_absolute_y(
        taffy_tree: &TaffyTree<NodeContext>,
        taffy_node_id: taffy::NodeId,
        layout: &taffy::Layout,
    ) -> f32 {
        let mut y_acc = layout.location.y;
        let mut current_node_id = taffy_node_id;
        while let Some(parent_taffy_node_id) = taffy_tree.parent(current_node_id) {
            let Ok(parent_layout) = taffy_tree.layout(parent_taffy_node_id) else {
                break;
            };
            y_acc += parent_layout.location.y;
            current_node_id = parent_taffy_node_id;
        }
        y_acc
    }
}
