use disposition_input_model::process::{ProcessDiagram, Processes};
use disposition_ir_model::{
    node::NodeId,
    process::{
        ProcessStepEdges, ProcessStepGraph, ProcessStepGraphEdge, ProcessStepGraphs,
        ProcessStepLane, ProcessStepPlacement, ProcessStepRanks,
    },
};
use disposition_model_common::{Map, Set};

/// Computes the git-graph layout ([`ProcessStepGraphs`]) for each process.
///
/// Process steps are laid out like `git log --graph`: each step occupies a row
/// (ordered by [`ProcessStepRank`] then declaration order) and a lane. A step
/// shifts to a higher lane when a connector edge needs to bypass its row, so
/// the bypassing connector keeps a straight vertical line in a lower lane.
///
/// The lane packing walks rows top-to-bottom maintaining a set of active lanes,
/// one per pending connector. At each step:
///
/// * the step takes the leftmost incoming-connector lane, or the first free
///   lane if it has no incoming connector;
/// * the first outgoing connector reuses the step's lane, and additional
///   outgoing connectors (branches) each claim a new free lane;
/// * lanes whose connector targets a later row stay reserved -- these are the
///   bypass lanes that push other steps to the right.
///
/// Back connectors (target row at or before the source row, e.g. cycles) are
/// handled on a best-effort basis: they reuse the source step's lane and are
/// not reserved.
///
/// [`ProcessStepRank`]: disposition_ir_model::process::ProcessStepRank
#[derive(Clone, Copy, Debug)]
pub struct ProcessStepGraphCalculator;

impl ProcessStepGraphCalculator {
    /// Computes the [`ProcessStepGraphs`] for all processes.
    ///
    /// # Parameters
    ///
    /// * `processes`: The processes and their declared steps.
    /// * `process_step_ranks`: The computed rank of each process step, used to
    ///   order steps into rows.
    /// * `process_step_edges`: The connector edges between process steps.
    pub fn calculate<'id>(
        processes: &Processes<'id>,
        process_step_ranks: &ProcessStepRanks<'id>,
        process_step_edges: &ProcessStepEdges<'id>,
    ) -> ProcessStepGraphs<'id> {
        processes
            .iter()
            .filter_map(|(process_id, process_diagram)| {
                if process_diagram.steps.is_empty() {
                    return None;
                }
                let process_node_id = NodeId::from(process_id.as_ref().clone());
                let graph = Self::process_graph_build(
                    process_diagram,
                    process_step_ranks,
                    process_step_edges,
                );
                Some((process_node_id, graph))
            })
            .collect()
    }

    /// Builds the [`ProcessStepGraph`] for a single process.
    fn process_graph_build<'id>(
        process_diagram: &ProcessDiagram<'id>,
        process_step_ranks: &ProcessStepRanks<'id>,
        process_step_edges: &ProcessStepEdges<'id>,
    ) -> ProcessStepGraph<'id> {
        // Step node IDs in declaration order.
        let steps_decl: Vec<NodeId<'id>> = process_diagram
            .steps
            .keys()
            .map(|step_id| NodeId::from(step_id.as_ref().clone()))
            .collect();

        let step_set: Set<NodeId<'id>> = steps_decl.iter().cloned().collect();

        // Declaration index per step, used as a stable tiebreaker.
        let decl_index: Map<NodeId<'id>, usize> = steps_decl
            .iter()
            .enumerate()
            .map(|(index, node_id)| (node_id.clone(), index))
            .collect();

        // Order steps into rows by (rank, declaration index). `sort_by` is
        // stable, so equal ranks keep declaration order.
        let rank_of =
            |node_id: &NodeId<'id>| process_step_ranks.get(node_id).copied().unwrap_or_default();
        let mut rows = steps_decl.clone();
        rows.sort_by_key(|node_id| rank_of(node_id));

        let row_of: Map<NodeId<'id>, u32> = rows
            .iter()
            .enumerate()
            .map(|(row, node_id)| (node_id.clone(), row as u32))
            .collect();

        // Outgoing connector targets per step (only within this process).
        let mut out_targets: Map<NodeId<'id>, Vec<NodeId<'id>>> = Map::new();
        for edge in process_step_edges.iter() {
            if step_set.contains(&edge.from) && step_set.contains(&edge.to) {
                out_targets
                    .entry(edge.from.clone())
                    .or_default()
                    .push(edge.to.clone());
            }
        }
        // Sort each step's targets by (row, declaration index) so the nearest
        // target deterministically continues the step's lane.
        for targets in out_targets.values_mut() {
            targets.sort_by(|node_id_a, node_id_b| {
                let row_a = row_of.get(node_id_a).copied().unwrap_or(0);
                let row_b = row_of.get(node_id_b).copied().unwrap_or(0);
                row_a.cmp(&row_b).then_with(|| {
                    let decl_a = decl_index.get(node_id_a).copied().unwrap_or(0);
                    let decl_b = decl_index.get(node_id_b).copied().unwrap_or(0);
                    decl_a.cmp(&decl_b)
                })
            });
        }

        // === Lane packing === //
        let mut active_lanes: Vec<Option<NodeId<'id>>> = Vec::new();
        let mut step_placements: Map<NodeId<'id>, ProcessStepPlacement> = Map::new();
        let mut edges: Vec<ProcessStepGraphEdge<'id>> = Vec::new();
        let mut max_lane: u32 = 0;

        for (row, step) in rows.iter().enumerate() {
            let row = row as u32;

            // Incoming connector lanes (lanes whose pending target is this step).
            let incoming_lanes: Vec<usize> = active_lanes
                .iter()
                .enumerate()
                .filter_map(|(lane, target)| {
                    target
                        .as_ref()
                        .filter(|target| *target == step)
                        .map(|_| lane)
                })
                .collect();

            let lane = incoming_lanes
                .first()
                .copied()
                .unwrap_or_else(|| Self::lane_first_free(&mut active_lanes));

            // Free all incoming lanes -- they terminate at (merge into) this step.
            for incoming_lane in &incoming_lanes {
                active_lanes[*incoming_lane] = None;
            }

            step_placements.insert(
                step.clone(),
                ProcessStepPlacement::new(row, ProcessStepLane::new(lane as u32)),
            );
            max_lane = max_lane.max(lane as u32);

            // Assign outgoing connectors.
            if let Some(targets) = out_targets.get(step) {
                for (index, target) in targets.iter().enumerate() {
                    let is_back_edge = row_of.get(target).copied().unwrap_or(0) <= row;

                    let edge_lane = if index == 0 || is_back_edge {
                        // First forward branch (and any back edge) continues the
                        // step's own lane.
                        lane
                    } else {
                        Self::lane_first_free(&mut active_lanes)
                    };

                    // Reserve the lane for forward connectors so later steps know
                    // it is occupied (a bypass lane). Back connectors target an
                    // earlier row, so they are not reserved.
                    if !is_back_edge {
                        active_lanes[edge_lane] = Some(target.clone());
                    }

                    edges.push(ProcessStepGraphEdge::new(
                        step.clone(),
                        target.clone(),
                        ProcessStepLane::new(edge_lane as u32),
                    ));
                    max_lane = max_lane.max(edge_lane as u32);
                }
            }
        }

        ProcessStepGraph {
            lane_count: max_lane + 1,
            step_placements,
            edges,
        }
    }

    /// Returns the index of the first free lane, growing the lane set if
    /// needed.
    fn lane_first_free(active_lanes: &mut Vec<Option<NodeId<'_>>>) -> usize {
        if let Some(index) = active_lanes.iter().position(Option::is_none) {
            index
        } else {
            active_lanes.push(None);
            active_lanes.len() - 1
        }
    }
}
