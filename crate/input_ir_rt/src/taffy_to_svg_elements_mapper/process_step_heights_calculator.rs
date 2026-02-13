use disposition_ir_model::{entity::EntityType, node::NodeId, IrDiagram};
use disposition_model_common::{Map, Set};
use disposition_taffy_model::{NodeContext, NodeToTaffyNodeIds};
use taffy::TaffyTree;

use crate::taffy_to_svg_elements_mapper::ProcessStepsHeight;

#[derive(Clone, Copy, Debug)]
pub struct ProcessStepHeightsCalculator;

impl ProcessStepHeightsCalculator {
    /// Collects information about process nodes and their steps for
    /// y-coordinate calculations.
    ///
    /// Returns a vector of ProcessInfo in the order processes appear in
    /// node_ordering.
    pub fn calculate<'id>(
        ir_diagram: &IrDiagram<'id>,
        taffy_tree: &TaffyTree<NodeContext>,
        node_id_to_taffy: &Map<NodeId<'id>, NodeToTaffyNodeIds>,
    ) -> Vec<ProcessStepsHeight<'id>> {
        let mut process_steps_height = Vec::new();

        // Iterate through node_ordering to find process nodes in order
        ir_diagram
            .node_hierarchy
            .iter()
            .filter_map(|(node_id, children)| {
                let is_process = ir_diagram
                    .entity_types
                    .get(node_id.as_ref())
                    .is_some_and(|types| types.contains(&EntityType::ProcessDefault));
                if is_process {
                    Some((
                        node_id.clone(),
                        children.keys().cloned().collect::<Set<NodeId<'_>>>(),
                    ))
                } else {
                    None
                }
            })
            .for_each(|(process_id, process_step_ids)| {
                // Calculate total height of all steps
                let total_height = if let Some(node_to_taffy_node_ids) =
                    node_id_to_taffy.get(process_id.as_ref())
                {
                    let text_taffy_node_id = node_to_taffy_node_ids.text_taffy_node_id();
                    let wrapper_taffy_node_id = node_to_taffy_node_ids.wrapper_taffy_node_id();
                    let process_node_text_height = taffy_tree
                        .layout(text_taffy_node_id)
                        .map(|layout| layout.size.height.min(layout.content_size.height))
                        .unwrap_or(0.0);
                    let process_node_total_height = taffy_tree
                        .layout(wrapper_taffy_node_id)
                        .map(|layout| layout.size.height.min(layout.content_size.height))
                        .unwrap_or(0.0);

                    process_node_total_height - process_node_text_height
                } else {
                    0.0
                };

                process_steps_height.push(ProcessStepsHeight {
                    process_id,
                    process_step_ids,
                    total_height,
                });
            });

        process_steps_height
    }
}
