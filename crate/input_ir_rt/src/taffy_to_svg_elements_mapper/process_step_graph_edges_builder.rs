use disposition_ir_model::{node::NodeId, IrDiagram};
use disposition_model_common::edge::EdgeGroupId;
use disposition_svg_model::{OrthoProtrusionParams, SvgEdgeInfo};
use disposition_taffy_model::{taffy::TaffyTree, NodeIdToTaffyNodeIds, TaffyNodeCtx, LANE_WIDTH};
use kurbo::{BezPath, Point};

use super::{ArrowHeadBuilder, EdgePathLocusCalculator};
use crate::{AbsoluteCoordinates, TaffyNodeAbsoluteCoordinatesCalculator};

/// Builds the git-graph connector [`SvgEdgeInfo`]s between process steps.
///
/// Each `ProcessStepGraphEdge` is drawn as an orthogonal, arc-rounded connector
/// that departs the `from` step's circle, runs vertically in its travel lane,
/// and enters the `to` step's circle. Like dependency edges, each connector
/// carries a positioned arrowhead at the `to` end and a locus path for the
/// focus indicator. The connector's tailwind classes (resolved from the theme's
/// `edge_defaults`) are looked up from `ir_diagram.tailwind_classes` by the
/// renderer, keyed by `ProcessStepGraphEdge::edge_id`.
#[derive(Clone, Copy, Debug)]
pub(super) struct ProcessStepGraphEdgesBuilder;

/// Small straight stub before a connector bends, in pixels.
const BEND_GAP: f32 = 6.0;
/// Corner rounding radius for connector bends, in pixels.
const ARC_RADIUS: f32 = 4.0;

impl ProcessStepGraphEdgesBuilder {
    /// Builds the connector edge infos for every process step graph.
    pub(super) fn build<'id>(
        ir_diagram: &IrDiagram<'id>,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        node_id_to_taffy: &NodeIdToTaffyNodeIds<'id>,
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

                // The path is built with its `MoveTo` at the `to` end (the
                // arrowhead/locus builders expect the to-node end first, since
                // edge paths are conventionally built in reverse).
                let path = Self::connector_path(
                    (from_x, from_y, from_radius),
                    (to_x, to_y, to_radius),
                    lane_x,
                );
                let arrow_head_path = ArrowHeadBuilder::build_static_arrow_head(&path);
                let locus_path = EdgePathLocusCalculator::calculate(&path, &arrow_head_path);

                let edge_id = edge.edge_id();
                let edge_group_id = EdgeGroupId::from(edge_id.clone().into_inner());

                svg_edge_infos.push(SvgEdgeInfo::new(
                    edge_id,
                    edge_group_id,
                    edge.from.clone(),
                    edge.to.clone(),
                    path.to_svg(),
                    arrow_head_path.to_svg(),
                    locus_path.to_svg(),
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
        let AbsoluteCoordinates { x, y } = TaffyNodeAbsoluteCoordinatesCalculator::calculate(
            taffy_tree,
            circle_taffy_node_id,
            layout,
        );
        let radius = layout.size.width / 2.0;
        Some((x + radius, y + radius, radius))
    }

    /// Builds the connector [`BezPath`] for a single connector, with its
    /// `MoveTo` at the `to` step's circle so the arrowhead lands there.
    ///
    /// Forward connectors (the `to` step below the `from` step) connect the top
    /// of the `to` circle to the bottom of the `from` circle, running down the
    /// travel lane in between. Back connectors (cycles, `to` at or above
    /// `from`) bulge out to the right to avoid overlapping the steps
    /// between them.
    fn connector_path(from: (f32, f32, f32), to: (f32, f32, f32), lane_x: f32) -> BezPath {
        let (from_x, from_y, from_radius) = from;
        let (to_x, to_y, to_radius) = to;

        // Forward-order waypoints (from-end first); reversed before building.
        let waypoints = if to_y >= from_y {
            let start = (from_x, from_y + from_radius);
            let end = (to_x, to_y - to_radius);

            let straight = (from_x - lane_x).abs() < 0.5 && (to_x - lane_x).abs() < 0.5;
            if straight {
                vec![start, end]
            } else {
                let bend_y_1 = (start.1 + BEND_GAP).min(end.1);
                let bend_y_2 = (end.1 - BEND_GAP).max(start.1);
                vec![
                    start,
                    (from_x, bend_y_1),
                    (lane_x, bend_y_1),
                    (lane_x, bend_y_2),
                    (to_x, bend_y_2),
                    end,
                ]
            }
        } else {
            // Back connector (best-effort): bulge to the right of both circles.
            let bulge_x = from_x.max(to_x) + LANE_WIDTH;
            let start = (from_x, from_y - from_radius);
            let end = (to_x, to_y + to_radius);
            let bend_y_1 = start.1 - BEND_GAP;
            let bend_y_2 = end.1 + BEND_GAP;
            vec![
                start,
                (from_x, bend_y_1),
                (bulge_x, bend_y_1),
                (bulge_x, bend_y_2),
                (to_x, bend_y_2),
                end,
            ]
        };

        // Reverse so the path starts at the `to` end.
        let mut waypoints_reversed = waypoints;
        waypoints_reversed.reverse();
        Self::ortho_bez_path(&waypoints_reversed)
    }

    /// Builds an orthogonal [`BezPath`] through `points` with arc-rounded
    /// corners.
    ///
    /// Consecutive duplicate points are collapsed, so collapsed bends (e.g.
    /// when the travel lane equals an endpoint's lane) do not produce
    /// zero-length segments.
    fn ortho_bez_path(points: &[(f32, f32)]) -> BezPath {
        // Collapse consecutive duplicate points.
        let mut points_collapsed: Vec<(f64, f64)> = Vec::with_capacity(points.len());
        for &(x, y) in points {
            let point_candidate = (x as f64, y as f64);
            if points_collapsed
                .last()
                .map(|point_last_kept| {
                    (point_last_kept.0 - point_candidate.0).abs() < 0.01
                        && (point_last_kept.1 - point_candidate.1).abs() < 0.01
                })
                .unwrap_or(false)
            {
                continue;
            }
            points_collapsed.push(point_candidate);
        }

        let mut path = BezPath::new();
        if points_collapsed.len() < 2 {
            return path;
        }

        path.move_to(Point::new(points_collapsed[0].0, points_collapsed[0].1));
        for index in 1..points_collapsed.len() - 1 {
            let point_previous = points_collapsed[index - 1];
            let point_corner = points_collapsed[index];
            let point_next = points_collapsed[index + 1];

            // The straight leg into the corner stops at `arc_start`, then a
            // quadratic curve rounds through the corner to `arc_end`.
            let arc_start = Self::point_towards(point_corner, point_previous);
            let arc_end = Self::point_towards(point_corner, point_next);

            path.line_to(Point::new(arc_start.0, arc_start.1));
            path.quad_to(
                Point::new(point_corner.0, point_corner.1),
                Point::new(arc_end.0, arc_end.1),
            );
        }
        let point_last = points_collapsed[points_collapsed.len() - 1];
        path.line_to(Point::new(point_last.0, point_last.1));

        path
    }

    /// Returns a point `ARC_RADIUS` (capped at half the segment) from `corner`
    /// toward `neighbour`.
    fn point_towards(corner: (f64, f64), neighbour: (f64, f64)) -> (f64, f64) {
        let delta_x = neighbour.0 - corner.0;
        let delta_y = neighbour.1 - corner.1;
        let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();
        if distance < f64::EPSILON {
            return corner;
        }
        let inset_distance = (ARC_RADIUS as f64).min(distance / 2.0);
        (
            corner.0 + delta_x / distance * inset_distance,
            corner.1 + delta_y / distance * inset_distance,
        )
    }
}
