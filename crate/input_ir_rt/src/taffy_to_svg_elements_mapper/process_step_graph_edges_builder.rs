use disposition_ir_model::{entity::EntityTailwindClasses, node::NodeId, IrDiagram};
use disposition_model_common::{
    edge::{EdgeGroupId, EdgeId},
    Id,
};
use disposition_svg_model::{OrthoProtrusionParams, SvgEdgeInfo};
use disposition_taffy_model::{
    taffy::TaffyTree, NodeIdToTaffyNodeIds, TaffyNodeCtx, LANE_WIDTH,
};

use crate::{AbsoluteCoordinates, TaffyNodeAbsoluteCoordinatesCalculator};

/// Builds the git-graph connector [`SvgEdgeInfo`]s between process steps.
///
/// Each `ProcessStepGraphEdge` is drawn as an orthogonal, arc-rounded connector
/// that departs the `from` step's circle, runs vertically in its travel lane,
/// and enters the `to` step's circle. This bypasses the thing/tag edge router
/// (`SvgEdgeInfosBuilder`) entirely -- no spacers, protrusions, or arrowheads.
#[derive(Clone, Copy, Debug)]
pub(super) struct ProcessStepGraphEdgesBuilder;

/// Small straight stub before a connector bends, in pixels.
const BEND_GAP: f32 = 6.0;
/// Corner rounding radius for connector bends, in pixels.
const ARC_RADIUS: f32 = 4.0;
/// Neutral connector styling applied to every process step connector `<g>`.
const CONNECTOR_TAILWIND_CLASSES: &str = "visible fill-none stroke-[#64748b] [stroke-width:1.5]";

impl ProcessStepGraphEdgesBuilder {
    /// Builds the connector edge infos for every process step graph.
    ///
    /// Inserts a neutral stroke style into `tailwind_classes` for each connector
    /// and returns the connectors to be appended to the diagram's edge infos.
    pub(super) fn build<'id>(
        ir_diagram: &IrDiagram<'id>,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        node_id_to_taffy: &NodeIdToTaffyNodeIds<'id>,
        tailwind_classes: &mut EntityTailwindClasses<'id>,
    ) -> Vec<SvgEdgeInfo<'id>> {
        let mut svg_edge_infos = Vec::new();

        for process_step_graph in ir_diagram.process_step_graphs.values() {
            for edge in &process_step_graph.edges {
                let from_lane = process_step_graph
                    .step_placements
                    .get(&edge.from)
                    .map(|placement| placement.lane.value())
                    .unwrap_or(0);

                let Some((from_x, from_y, from_radius)) =
                    Self::circle_center(taffy_tree, node_id_to_taffy, &edge.from)
                else {
                    continue;
                };
                let Some((to_x, to_y, to_radius)) =
                    Self::circle_center(taffy_tree, node_id_to_taffy, &edge.to)
                else {
                    continue;
                };

                // Travel lane x, relative to the from-circle's lane.
                let lane_delta = edge.lane.value() as f32 - from_lane as f32;
                let lane_x = from_x + lane_delta * LANE_WIDTH;

                let path_d = Self::connector_path_d(
                    (from_x, from_y, from_radius),
                    (to_x, to_y, to_radius),
                    lane_x,
                );

                let edge_id_string = format!("psgraph_{}__{}", edge.from.as_str(), edge.to.as_str());
                let Ok(edge_id_inner) = Id::try_from(edge_id_string) else {
                    continue;
                };

                tailwind_classes.insert(
                    edge_id_inner.clone(),
                    CONNECTOR_TAILWIND_CLASSES.to_string(),
                );

                svg_edge_infos.push(SvgEdgeInfo::new(
                    EdgeId::from(edge_id_inner.clone()),
                    EdgeGroupId::from(edge_id_inner),
                    edge.from.clone(),
                    edge.to.clone(),
                    path_d,
                    String::new(),
                    String::new(),
                    String::new(),
                    OrthoProtrusionParams::default(),
                ));
            }
        }

        svg_edge_infos
    }

    /// Resolves a step circle's absolute centre and radius from its taffy node.
    fn circle_center<'id>(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        node_id_to_taffy: &NodeIdToTaffyNodeIds<'id>,
        node_id: &NodeId<'id>,
    ) -> Option<(f32, f32, f32)> {
        let taffy_node_ids = node_id_to_taffy.get(node_id)?;
        let circle_taffy_node_id = taffy_node_ids.circle_taffy_node_id()?;
        let layout = taffy_tree.layout(circle_taffy_node_id).ok()?;
        let AbsoluteCoordinates { x, y } =
            TaffyNodeAbsoluteCoordinatesCalculator::calculate(taffy_tree, circle_taffy_node_id, layout);
        let radius = layout.size.width / 2.0;
        Some((x + radius, y + radius, radius))
    }

    /// Builds the SVG path `d` for a single connector.
    ///
    /// Forward connectors (the `to` step below the `from` step) exit the bottom
    /// of the `from` circle, run down the travel lane, and enter the top of the
    /// `to` circle. Back connectors (cycles, `to` at or above `from`) bulge out
    /// to the right to avoid overlapping the steps between them.
    fn connector_path_d(
        from: (f32, f32, f32),
        to: (f32, f32, f32),
        lane_x: f32,
    ) -> String {
        let (from_x, from_y, from_radius) = from;
        let (to_x, to_y, to_radius) = to;

        if to_y >= from_y {
            // Forward connector.
            let start = (from_x, from_y + from_radius);
            let end = (to_x, to_y - to_radius);

            let straight = (from_x - lane_x).abs() < 0.5 && (to_x - lane_x).abs() < 0.5;
            if straight {
                Self::ortho_path_d(&[start, end])
            } else {
                let bend_y_1 = (start.1 + BEND_GAP).min(end.1);
                let bend_y_2 = (end.1 - BEND_GAP).max(start.1);
                Self::ortho_path_d(&[
                    start,
                    (from_x, bend_y_1),
                    (lane_x, bend_y_1),
                    (lane_x, bend_y_2),
                    (to_x, bend_y_2),
                    end,
                ])
            }
        } else {
            // Back connector (best-effort): bulge to the right of both circles.
            let bulge_x = from_x.max(to_x) + LANE_WIDTH;
            let start = (from_x, from_y - from_radius);
            let end = (to_x, to_y + to_radius);
            let bend_y_1 = start.1 - BEND_GAP;
            let bend_y_2 = end.1 + BEND_GAP;
            Self::ortho_path_d(&[
                start,
                (from_x, bend_y_1),
                (bulge_x, bend_y_1),
                (bulge_x, bend_y_2),
                (to_x, bend_y_2),
                end,
            ])
        }
    }

    /// Builds an orthogonal SVG path through `points` with arc-rounded corners.
    ///
    /// Consecutive duplicate points are collapsed, so collapsed bends (e.g. when
    /// the travel lane equals an endpoint's lane) do not produce zero-length
    /// segments.
    fn ortho_path_d(points: &[(f32, f32)]) -> String {
        // Collapse consecutive duplicate points.
        let mut pts: Vec<(f32, f32)> = Vec::with_capacity(points.len());
        for &point in points {
            if pts
                .last()
                .map(|last| (last.0 - point.0).abs() < 0.01 && (last.1 - point.1).abs() < 0.01)
                .unwrap_or(false)
            {
                continue;
            }
            pts.push(point);
        }

        if pts.len() < 2 {
            return String::new();
        }

        let mut path_d = format!("M{:.2},{:.2}", pts[0].0, pts[0].1);
        for index in 1..pts.len() - 1 {
            let previous = pts[index - 1];
            let corner = pts[index];
            let next = pts[index + 1];

            let leg_in = Self::point_towards(corner, previous);
            let leg_out = Self::point_towards(corner, next);

            path_d.push_str(&format!(
                " L{:.2},{:.2} Q{:.2},{:.2} {:.2},{:.2}",
                leg_in.0, leg_in.1, corner.0, corner.1, leg_out.0, leg_out.1
            ));
        }
        let last = pts[pts.len() - 1];
        path_d.push_str(&format!(" L{:.2},{:.2}", last.0, last.1));

        path_d
    }

    /// Returns a point `ARC_RADIUS` (capped at half the segment) from `corner`
    /// toward `neighbour`.
    fn point_towards(corner: (f32, f32), neighbour: (f32, f32)) -> (f32, f32) {
        let dx = neighbour.0 - corner.0;
        let dy = neighbour.1 - corner.1;
        let distance = (dx * dx + dy * dy).sqrt();
        if distance < f32::EPSILON {
            return corner;
        }
        let step = ARC_RADIUS.min(distance / 2.0);
        (corner.0 + dx / distance * step, corner.1 + dy / distance * step)
    }
}
