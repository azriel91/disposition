use disposition_ir_model::{node::NodeId, IrDiagram};
use disposition_model_common::edge::EdgeGroupId;
use disposition_svg_model::{OrthoProtrusionParams, SvgEdgeInfo};
use disposition_taffy_model::{taffy::TaffyTree, NodeIdToTaffyNodeIds, TaffyNodeCtx, LANE_WIDTH};
use kurbo::{BezPath, Circle, Point};

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
const BEND_GAP: f64 = 6.0;
/// Corner rounding radius for connector bends, in pixels.
const ARC_RADIUS: f64 = 4.0;

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

                let Some(from_circle) =
                    Self::circle_resolve(taffy_tree, node_id_to_taffy, &edge.from)
                else {
                    continue;
                };
                let Some(to_circle) = Self::circle_resolve(taffy_tree, node_id_to_taffy, &edge.to)
                else {
                    continue;
                };

                // Travel lane x, relative to the from-circle's lane.
                let lane_delta = f64::from(edge.lane.value()) - f64::from(from_lane);
                let lane_x = from_circle.center.x + lane_delta * f64::from(LANE_WIDTH);

                // The path runs from the `from` step to the `to` step, so the
                // arrowhead/locus builders (which place the arrow at the path's
                // final point) land it at the `to` step.
                let path = Self::connector_path(from_circle, to_circle, lane_x);
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

    /// Resolves a step's [`Circle`] (absolute centre and radius) from its taffy
    /// node.
    fn circle_resolve<'id>(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        node_id_to_taffy: &NodeIdToTaffyNodeIds<'id>,
        node_id: &NodeId<'id>,
    ) -> Option<Circle> {
        let taffy_node_ids = node_id_to_taffy.get(node_id)?;
        let circle_taffy_node_id = taffy_node_ids.circle_taffy_node_id()?;
        let layout = taffy_tree.layout(circle_taffy_node_id).ok()?;
        let AbsoluteCoordinates { x, y } = TaffyNodeAbsoluteCoordinatesCalculator::calculate(
            taffy_tree,
            circle_taffy_node_id,
            layout,
        );
        let radius = f64::from(layout.size.width) / 2.0;
        let center = Point::new(f64::from(x) + radius, f64::from(y) + radius);
        Some(Circle::new(center, radius))
    }

    /// Builds the connector [`BezPath`] for a single connector, running from
    /// the `from` step's circle to the `to` step's circle so the arrowhead
    /// lands at the `to` end.
    ///
    /// Forward connectors (the `to` step below the `from` step) connect the
    /// bottom of the `from` circle to the top of the `to` circle, running down
    /// the travel lane in between. Back connectors (cycles, `to` at or above
    /// `from`) bulge out to the right to avoid overlapping the steps
    /// between them.
    fn connector_path(from: Circle, to: Circle, lane_x: f64) -> BezPath {
        // Waypoints run from the `from` end to the `to` end.
        let waypoints = if to.center.y >= from.center.y {
            let start = Point::new(from.center.x, from.center.y + from.radius);
            let end = Point::new(to.center.x, to.center.y - to.radius);

            let straight =
                (from.center.x - lane_x).abs() < 0.5 && (to.center.x - lane_x).abs() < 0.5;
            if straight {
                vec![start, end]
            } else {
                let bend_y_1 = (start.y + BEND_GAP).min(end.y);
                let bend_y_2 = (end.y - BEND_GAP).max(start.y);
                vec![
                    start,
                    Point::new(from.center.x, bend_y_1),
                    Point::new(lane_x, bend_y_1),
                    Point::new(lane_x, bend_y_2),
                    Point::new(to.center.x, bend_y_2),
                    end,
                ]
            }
        } else {
            // Back connector (best-effort): bulge to the right of both circles.
            let bulge_x = from.center.x.max(to.center.x) + f64::from(LANE_WIDTH);
            let start = Point::new(from.center.x, from.center.y - from.radius);
            let end = Point::new(to.center.x, to.center.y + to.radius);
            let bend_y_1 = start.y - BEND_GAP;
            let bend_y_2 = end.y + BEND_GAP;
            vec![
                start,
                Point::new(from.center.x, bend_y_1),
                Point::new(bulge_x, bend_y_1),
                Point::new(bulge_x, bend_y_2),
                Point::new(to.center.x, bend_y_2),
                end,
            ]
        };

        Self::ortho_bez_path(&waypoints)
    }

    /// Builds an orthogonal [`BezPath`] through `points` with arc-rounded
    /// corners.
    ///
    /// Consecutive duplicate points are collapsed, so collapsed bends (e.g.
    /// when the travel lane equals an endpoint's lane) do not produce
    /// zero-length segments.
    fn ortho_bez_path(points: &[Point]) -> BezPath {
        // Collapse consecutive duplicate points.
        let mut points_collapsed: Vec<Point> = Vec::with_capacity(points.len());
        for &point_candidate in points {
            if points_collapsed
                .last()
                .map(|point_last_kept| {
                    (point_last_kept.x - point_candidate.x).abs() < 0.01
                        && (point_last_kept.y - point_candidate.y).abs() < 0.01
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

        path.move_to(points_collapsed[0]);
        for index in 1..points_collapsed.len() - 1 {
            let point_previous = points_collapsed[index - 1];
            let point_corner = points_collapsed[index];
            let point_next = points_collapsed[index + 1];

            // The straight leg into the corner stops at `arc_start`, then a
            // quadratic curve rounds through the corner to `arc_end`.
            let arc_start = Self::point_towards(point_corner, point_previous);
            let arc_end = Self::point_towards(point_corner, point_next);

            path.line_to(arc_start);
            path.quad_to(point_corner, arc_end);
        }
        path.line_to(points_collapsed[points_collapsed.len() - 1]);

        path
    }

    /// Returns a point `ARC_RADIUS` (capped at half the segment) from `corner`
    /// toward `neighbour`.
    fn point_towards(corner: Point, neighbour: Point) -> Point {
        let direction = neighbour - corner;
        let distance = direction.hypot();
        if distance < f64::EPSILON {
            return corner;
        }
        let inset_distance = ARC_RADIUS.min(distance / 2.0);
        corner + direction * (inset_distance / distance)
    }
}
