use kurbo::{BezPath, Point};

use crate::taffy_to_svg_elements_mapper::{
    edge_model::NodeFace, edge_path_builder_pass_1::SpacerCoordinates,
};

/// Arc radius in pixels for orthogonal path corners.
///
/// # Example values
///
/// `4.0` -- produces a small visible rounding at each 90-degree turn.
const ARC_RADIUS: f32 = 4.0;

/// Kappa constant for approximating a quarter-circle arc with a cubic
/// bezier curve.
///
/// Equal to `(4.0 / 3.0) * (sqrt(2) - 1)`, approximately `0.5522847498`.
const KAPPA: f32 = 0.552_284_8;

/// Protrusion lengths for the entry and exit sides of a single spacer.
///
/// The entry-side protrusion extends the path past the spacer's entry
/// boundary (away from the spacer, into the gap before it). The
/// exit-side protrusion extends the path past the spacer's exit
/// boundary (away from the spacer, into the gap after it).
///
/// Protrusion depths are assigned by `OrthoProtrusionCalculator` so
/// that edges sharing the same inter-rank gap use distinct depths.
///
/// # Example values
///
/// ```text
/// SpacerProtrusionParams { entry_protrusion: 5.0, exit_protrusion: 8.0 }
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub(in crate::taffy_to_svg_elements_mapper) struct SpacerProtrusionParams {
    /// Protrusion length in pixels on the entry side of the spacer.
    ///
    /// `0.0` means no protrusion on the entry side.
    pub(in crate::taffy_to_svg_elements_mapper) entry_protrusion: f32,

    /// Protrusion length in pixels on the exit side of the spacer.
    ///
    /// `0.0` means no protrusion on the exit side.
    pub(in crate::taffy_to_svg_elements_mapper) exit_protrusion: f32,
}

/// Protrusion lengths for the from-node and to-node endpoints of an
/// orthogonal edge path, plus per-spacer protrusion depths.
///
/// A protrusion is a short stub that exits the node face perpendicular
/// to the face line before the main orthogonal routing begins. This
/// separates parallel edges that share the same node face.
///
/// Spacer protrusions serve the same purpose at intermediate spacer
/// boundaries: they extend the path past the spacer so that the
/// routing leg between spacers does not run along a node face, and
/// multiple edges crossing the same inter-rank gap use distinct
/// depths.
///
/// # Example values
///
/// ```text
/// OrthoProtrusionParams {
///     from_protrusion: 12.0,
///     to_protrusion: 8.0,
///     spacer_protrusions: vec![
///         SpacerProtrusionParams { entry_protrusion: 12.0, exit_protrusion: 5.0 },
///     ],
/// }
/// ```
///
/// An edge whose from-node is close to the face midpoint gets a longer
/// `from_protrusion`; an edge further from the midpoint gets a shorter
/// one. Each spacer's entry and exit protrusions are computed
/// independently based on the edges sharing that specific rank gap.
#[derive(Clone, Debug, Default)]
pub(in crate::taffy_to_svg_elements_mapper) struct OrthoProtrusionParams {
    /// Protrusion length in pixels at the from-node endpoint.
    ///
    /// `0.0` means no protrusion (the path routes directly from the
    /// contact point).
    pub(in crate::taffy_to_svg_elements_mapper) from_protrusion: f32,

    /// Protrusion length in pixels at the to-node endpoint.
    ///
    /// `0.0` means no protrusion.
    pub(in crate::taffy_to_svg_elements_mapper) to_protrusion: f32,

    /// Per-spacer protrusion depths, indexed in the same order as the
    /// `spacers` slice passed to `build_spacer_edge_path`.
    ///
    /// When the edge has no spacers, this is empty.
    pub(in crate::taffy_to_svg_elements_mapper) spacer_protrusions: Vec<SpacerProtrusionParams>,
}

/// Builds pass-2 edge paths using orthogonal (90-degree) lines with
/// rounded arc corners between spacers.
///
/// This handles the `EdgeCurvature::Orthogonal` variant where segments
/// between nodes and spacers are drawn as horizontal/vertical lines
/// that turn at 90-degree angles, with small arcs rounding each corner.
///
/// When `OrthoProtrusionParams` specifies non-zero protrusions, a
/// short perpendicular stub is drawn exiting/entering each node face
/// before the main routing segment. This turns each L-shaped segment
/// into a Z-shaped or S-shaped segment with two 90-degree turns.
///
/// Spacer protrusions extend the path slightly past each spacer
/// boundary so that the routing legs do not run directly along node
/// faces.
#[derive(Clone, Copy, Debug)]
pub(in crate::taffy_to_svg_elements_mapper) struct EdgePathBuilderPass2Ortho;

impl EdgePathBuilderPass2Ortho {
    /// Builds an orthogonal path from `start` through spacers to `end`.
    ///
    /// The path structure mirrors
    /// `EdgePathBuilderPass2Curve::build_spacer_edge_path`
    /// but uses right-angle segments instead of bezier curves:
    ///
    /// 1. An orthogonal segment from `end` (to-face) to the last spacer's exit.
    /// 2. A straight line through each spacer (exit to entry, in reversed
    ///    order).
    /// 3. Orthogonal segments between adjacent spacers.
    /// 4. An orthogonal segment from the first spacer's entry to `start`
    ///    (from-face).
    ///
    /// The path is built in reverse order (from `end` to `start`) for
    /// correct SVG rendering direction.
    ///
    /// Spacer protrusions extend the path past each spacer boundary so
    /// that the routing legs clear node faces. Every direction change
    /// in the resulting path is rounded with a small arc.
    ///
    /// # Example values
    ///
    /// * `start_x = 100.0, start_y = 50.0` -- from-node contact point
    /// * `end_x = 400.0, end_y = 250.0` -- to-node contact point
    /// * `spacers = &[SpacerCoordinates { entry_x: 200.0, entry_y: 130.0,
    ///   exit_x: 200.0, exit_y: 135.0 }]`
    #[allow(clippy::too_many_arguments)]
    pub(in crate::taffy_to_svg_elements_mapper) fn build_spacer_edge_path(
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        from_face: NodeFace,
        to_face: NodeFace,
        spacers: &[SpacerCoordinates],
        protrusion: &OrthoProtrusionParams,
    ) -> BezPath {
        // === Collect waypoints === //
        //
        // Build an ordered list of waypoints that the path must visit,
        // each annotated with a direction. The path is constructed in
        // reverse (end -> start) for SVG rendering, so waypoints are
        // collected in that order.
        //
        // A waypoint is a coordinate + direction. Between consecutive
        // waypoints, a multi-leg orthogonal segment with rounded
        // corners is drawn.

        let mut waypoints: Vec<Waypoint> = Vec::new();

        // --- To-node contact point and protrusion --- //
        let to_dir = Self::face_outward_direction(to_face);

        waypoints.push(Waypoint {
            x: end_x,
            y: end_y,
            dir: to_dir,
        });

        if protrusion.to_protrusion > 1e-3 {
            let (eff_end_x, eff_end_y) =
                Self::protrusion_offset(end_x, end_y, to_face, protrusion.to_protrusion);
            waypoints.push(Waypoint {
                x: eff_end_x,
                y: eff_end_y,
                dir: to_dir,
            });
        }

        // --- Spacer waypoints (in reverse order) --- //
        let spacer_count = spacers.len();
        for (rev_index, spacer) in spacers.iter().rev().enumerate() {
            let (sdx, sdy) = Self::spacer_passthrough_direction(spacer);

            // Original (forward) index of this spacer.
            let fwd_index = spacer_count - 1 - rev_index;

            let spacer_prot = protrusion
                .spacer_protrusions
                .get(fwd_index)
                .copied()
                .unwrap_or_default();

            // Spacer exit side (with protrusion extending past exit).
            //
            // The protrusion extends the path past the spacer exit in
            // the passthrough direction, so that the routing leg that
            // connects to this spacer does not run along a node face.
            let exit_prot_x = spacer.exit_x + sdx * spacer_prot.exit_protrusion;
            let exit_prot_y = spacer.exit_y + sdy * spacer_prot.exit_protrusion;

            waypoints.push(Waypoint {
                x: exit_prot_x,
                y: exit_prot_y,
                // Direction entering this waypoint from the previous
                // segment (reversed passthrough, since we're building
                // the path in reverse).
                dir: (-sdx, -sdy),
            });

            // Spacer exit (actual coordinate).
            waypoints.push(Waypoint {
                x: spacer.exit_x,
                y: spacer.exit_y,
                dir: (-sdx, -sdy),
            });

            // Spacer entry (actual coordinate) -- straight through.
            waypoints.push(Waypoint {
                x: spacer.entry_x,
                y: spacer.entry_y,
                dir: (-sdx, -sdy),
            });

            // Spacer entry side (with protrusion extending past entry).
            //
            // Only add the entry-side protrusion if this is NOT the
            // last spacer in the reversed iteration. If it IS the last
            // (i.e. the first spacer in original order), the from-node
            // protrusion handles the extension on that side.
            //
            // Also, between consecutive spacers in the same rank gap,
            // both the exit protrusion of the next spacer and the entry
            // protrusion of this spacer serve to keep the routing leg
            // clear of node faces.
            let entry_prot_x = spacer.entry_x - sdx * spacer_prot.entry_protrusion;
            let entry_prot_y = spacer.entry_y - sdy * spacer_prot.entry_protrusion;

            waypoints.push(Waypoint {
                x: entry_prot_x,
                y: entry_prot_y,
                dir: (-sdx, -sdy),
            });
        }

        // --- From-node protrusion and contact point --- //
        let from_dir = Self::face_outward_direction(from_face);

        if protrusion.from_protrusion > 1e-3 {
            let (eff_start_x, eff_start_y) =
                Self::protrusion_offset(start_x, start_y, from_face, protrusion.from_protrusion);
            waypoints.push(Waypoint {
                x: eff_start_x,
                y: eff_start_y,
                dir: from_dir,
            });
        }

        waypoints.push(Waypoint {
            x: start_x,
            y: start_y,
            dir: from_dir,
        });

        // === Build path from waypoints === //
        Self::path_from_waypoints(&waypoints)
    }

    /// Builds an orthogonal path between two non-spacer endpoints
    /// (no spacers involved).
    ///
    /// Used when `spacers` is empty and `EdgeCurvature::Orthogonal` is
    /// selected.
    ///
    /// # Example values
    ///
    /// * `start_x = 100.0, start_y = 50.0`
    /// * `end_x = 300.0, end_y = 150.0`
    /// * `from_face = NodeFace::Bottom, to_face = NodeFace::Top`
    pub(in crate::taffy_to_svg_elements_mapper) fn build_ortho_edge_path(
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        from_face: NodeFace,
        to_face: NodeFace,
        protrusion: &OrthoProtrusionParams,
    ) -> BezPath {
        let mut waypoints: Vec<Waypoint> = Vec::new();

        let to_dir = Self::face_outward_direction(to_face);
        let from_dir = Self::face_outward_direction(from_face);

        // --- To-node contact point --- //
        waypoints.push(Waypoint {
            x: end_x,
            y: end_y,
            dir: to_dir,
        });

        // --- To-node protrusion tip --- //
        if protrusion.to_protrusion > 1e-3 {
            let (eff_end_x, eff_end_y) =
                Self::protrusion_offset(end_x, end_y, to_face, protrusion.to_protrusion);
            waypoints.push(Waypoint {
                x: eff_end_x,
                y: eff_end_y,
                dir: to_dir,
            });
        }

        // --- From-node protrusion tip --- //
        if protrusion.from_protrusion > 1e-3 {
            let (eff_start_x, eff_start_y) =
                Self::protrusion_offset(start_x, start_y, from_face, protrusion.from_protrusion);
            waypoints.push(Waypoint {
                x: eff_start_x,
                y: eff_start_y,
                dir: from_dir,
            });
        }

        // --- From-node contact point --- //
        waypoints.push(Waypoint {
            x: start_x,
            y: start_y,
            dir: from_dir,
        });

        Self::path_from_waypoints(&waypoints)
    }

    /// Builds a `BezPath` from an ordered list of waypoints.
    ///
    /// Between consecutive waypoints, the path is routed with
    /// orthogonal legs. Every direction change (including between
    /// collinear segments that meet a perpendicular routing leg and
    /// at protrusion junctions) is rounded with a small arc.
    ///
    /// Two consecutive waypoints are connected by either:
    /// - A straight line, if they are collinear in the departure direction.
    /// - An L-shaped segment (two legs with one rounded corner), if one turn is
    ///   needed.
    /// - A Z/S-shaped segment (three legs with two rounded corners), if two
    ///   turns are needed (e.g. different departure and arrival directions that
    ///   are both perpendicular to the displacement).
    fn path_from_waypoints(waypoints: &[Waypoint]) -> BezPath {
        let mut path = BezPath::new();

        if waypoints.is_empty() {
            return path;
        }

        // Move to the first waypoint.
        path.move_to(Point::new(waypoints[0].x as f64, waypoints[0].y as f64));

        if waypoints.len() < 2 {
            return path;
        }

        // Connect consecutive waypoint pairs.
        for i in 0..waypoints.len() - 1 {
            let wp_from = &waypoints[i];
            let wp_to = &waypoints[i + 1];

            Self::connect_waypoints(&mut path, wp_from, wp_to);
        }

        path
    }

    /// Connects two consecutive waypoints with orthogonal legs and
    /// rounded corners at every direction change.
    ///
    /// The departure direction at `wp_from` and the arrival direction
    /// at `wp_to` determine how many legs are needed:
    ///
    /// - **Collinear**: straight `line_to`.
    /// - **One turn**: L-shaped with one rounded corner.
    /// - **Two turns**: Z/S-shaped with two rounded corners.
    fn connect_waypoints(path: &mut BezPath, wp_from: &Waypoint, wp_to: &Waypoint) {
        let px = wp_from.x;
        let py = wp_from.y;
        let qx = wp_to.x;
        let qy = wp_to.y;

        let dx = qx - px;
        let dy = qy - py;
        let dist = (dx * dx + dy * dy).sqrt();

        // Degenerate: points are coincident -- skip.
        if dist < 1e-3 {
            return;
        }

        let (p_dx, p_dy) = wp_from.dir;
        let (q_dx, q_dy) = wp_to.dir;

        let p_is_vertical = p_dy.abs() > p_dx.abs();
        let q_is_vertical = q_dy.abs() > q_dx.abs();

        // Check if the displacement between waypoints is along the
        // departure direction (collinear).
        let disp_ux = dx / dist;
        let disp_uy = dy / dist;
        let dot_p = disp_ux * p_dx + disp_uy * p_dy;

        if dot_p.abs() > 0.95 {
            // Nearly collinear with the departure direction -- just
            // draw a straight line.
            path.line_to(Point::new(qx as f64, qy as f64));
            return;
        }

        // Determine if departure and arrival directions are parallel
        // (both vertical or both horizontal). If so, we need a
        // Z/S-shaped path with two turns. If perpendicular, an
        // L-shaped path with one turn suffices.
        if p_is_vertical == q_is_vertical {
            // === Z/S-shape: two turns === //
            //
            // Both directions are the same axis (both vertical or both
            // horizontal). Route with three legs and two corners.
            //
            // The bend is placed at the `wp_to` coordinate (the
            // from-node / spacer-exit side, since waypoints are
            // collected in reverse order). This means the protrusion
            // length directly controls the distance from the
            // from-node face to the bend, keeping the routing
            // segment on the from-node side of the gap.
            //
            // For vertical departure and arrival: go vertically to
            // qy, turn horizontally to qx.
            //
            // For horizontal departure and arrival: go horizontally
            // to qx, turn vertically to qy.
            if p_is_vertical {
                // Offset the bend from qy back toward py by
                // ARC_RADIUS so that leg 3 has enough length for the
                // second rounded corner arc.
                let sign = if py < qy { -1.0 } else { 1.0 };
                let bend_y = qy + sign * ARC_RADIUS;
                let corner1_x = px;
                let corner1_y = bend_y;
                let corner2_x = qx;
                let corner2_y = bend_y;
                Self::three_leg_segment_append(
                    path, px, py, corner1_x, corner1_y, corner2_x, corner2_y, qx, qy,
                );
            } else {
                // Offset the bend from qx back toward px by
                // ARC_RADIUS so that leg 3 has enough length for the
                // second rounded corner arc.
                let sign = if px < qx { -1.0 } else { 1.0 };
                let bend_x = qx + sign * ARC_RADIUS;
                let corner1_x = bend_x;
                let corner1_y = py;
                let corner2_x = bend_x;
                let corner2_y = qy;
                Self::three_leg_segment_append(
                    path, px, py, corner1_x, corner1_y, corner2_x, corner2_y, qx, qy,
                );
            }
        } else {
            // === L-shape: one turn === //
            //
            // Departure and arrival are on perpendicular axes. One
            // L-shaped segment with a single rounded corner.
            if p_is_vertical {
                // Vertical first, then horizontal.
                // Corner at (px, qy).
                let corner_x = px;
                let corner_y = qy;
                Self::two_leg_segment_append(path, px, py, corner_x, corner_y, qx, qy);
            } else {
                // Horizontal first, then vertical.
                // Corner at (qx, py).
                let corner_x = qx;
                let corner_y = py;
                Self::two_leg_segment_append(path, px, py, corner_x, corner_y, qx, qy);
            }
        }
    }

    /// Returns the outward unit direction vector for a node face.
    ///
    /// # Example values
    ///
    /// * `NodeFace::Top` returns `(0.0, -1.0)`.
    /// * `NodeFace::Bottom` returns `(0.0, 1.0)`.
    fn face_outward_direction(face: NodeFace) -> (f32, f32) {
        match face {
            NodeFace::Top => (0.0, -1.0),
            NodeFace::Bottom => (0.0, 1.0),
            NodeFace::Left => (-1.0, 0.0),
            NodeFace::Right => (1.0, 0.0),
        }
    }

    /// Computes the point offset from `(x, y)` along the outward normal
    /// of `face` by `protrusion_len` pixels.
    ///
    /// This is the tip of the protrusion stub: the point where the main
    /// orthogonal routing begins or ends.
    ///
    /// # Example values
    ///
    /// * `(100.0, 50.0)` with `NodeFace::Bottom` and `protrusion_len = 10.0`
    ///   returns `(100.0, 60.0)`.
    /// * `(100.0, 50.0)` with `NodeFace::Top` and `protrusion_len = 10.0`
    ///   returns `(100.0, 40.0)`.
    fn protrusion_offset(x: f32, y: f32, face: NodeFace, protrusion_len: f32) -> (f32, f32) {
        match face {
            NodeFace::Top => (x, y - protrusion_len),
            NodeFace::Bottom => (x, y + protrusion_len),
            NodeFace::Left => (x - protrusion_len, y),
            NodeFace::Right => (x + protrusion_len, y),
        }
    }

    /// Appends a two-leg orthogonal path from `(ax, ay)` through corner
    /// `(cx, cy)` to `(bx, by)`, with a rounded arc at the corner.
    ///
    /// If either leg has zero length, a straight line is drawn instead.
    fn two_leg_segment_append(
        path: &mut BezPath,
        ax: f32,
        ay: f32,
        cx: f32,
        cy: f32,
        bx: f32,
        by: f32,
    ) {
        let leg1_dx = cx - ax;
        let leg1_dy = cy - ay;
        let leg1_len = (leg1_dx * leg1_dx + leg1_dy * leg1_dy).sqrt();

        let leg2_dx = bx - cx;
        let leg2_dy = by - cy;
        let leg2_len = (leg2_dx * leg2_dx + leg2_dy * leg2_dy).sqrt();

        // If either leg is degenerate, just draw a straight line.
        if leg1_len < 1e-3 || leg2_len < 1e-3 {
            path.line_to(Point::new(bx as f64, by as f64));
            return;
        }

        // Clamp arc radius so it doesn't exceed half of either leg.
        let radius = ARC_RADIUS.min(leg1_len / 2.0).min(leg2_len / 2.0);

        // Unit vectors along each leg.
        let u1x = leg1_dx / leg1_len;
        let u1y = leg1_dy / leg1_len;
        let u2x = leg2_dx / leg2_len;
        let u2y = leg2_dy / leg2_len;

        // Point where the arc starts (radius back from corner along leg 1).
        let arc_start_x = cx - u1x * radius;
        let arc_start_y = cy - u1y * radius;

        // Point where the arc ends (radius forward from corner along leg 2).
        let arc_end_x = cx + u2x * radius;
        let arc_end_y = cy + u2y * radius;

        // Draw line from current point to arc start.
        path.line_to(Point::new(arc_start_x as f64, arc_start_y as f64));

        // Draw quarter-circle arc as a cubic bezier.
        let ctrl1_x = arc_start_x + u1x * radius * KAPPA;
        let ctrl1_y = arc_start_y + u1y * radius * KAPPA;
        let ctrl2_x = arc_end_x - u2x * radius * KAPPA;
        let ctrl2_y = arc_end_y - u2y * radius * KAPPA;

        path.curve_to(
            Point::new(ctrl1_x as f64, ctrl1_y as f64),
            Point::new(ctrl2_x as f64, ctrl2_y as f64),
            Point::new(arc_end_x as f64, arc_end_y as f64),
        );

        // Draw line from arc end to final point.
        path.line_to(Point::new(bx as f64, by as f64));
    }

    /// Appends a three-leg orthogonal path from `(ax, ay)` through two
    /// corners `(c1x, c1y)` and `(c2x, c2y)` to `(bx, by)`, with a
    /// rounded arc at each corner.
    ///
    /// Used for Z-shaped or S-shaped routing when both the departure
    /// and arrival directions are on the same axis (both vertical or
    /// both horizontal).
    #[allow(clippy::too_many_arguments)]
    fn three_leg_segment_append(
        path: &mut BezPath,
        ax: f32,
        ay: f32,
        c1x: f32,
        c1y: f32,
        c2x: f32,
        c2y: f32,
        bx: f32,
        by: f32,
    ) {
        // Leg 1: a -> c1
        let leg1_dx = c1x - ax;
        let leg1_dy = c1y - ay;
        let leg1_len = (leg1_dx * leg1_dx + leg1_dy * leg1_dy).sqrt();

        // Leg 2: c1 -> c2
        let leg2_dx = c2x - c1x;
        let leg2_dy = c2y - c1y;
        let leg2_len = (leg2_dx * leg2_dx + leg2_dy * leg2_dy).sqrt();

        // Leg 3: c2 -> b
        let leg3_dx = bx - c2x;
        let leg3_dy = by - c2y;
        let leg3_len = (leg3_dx * leg3_dx + leg3_dy * leg3_dy).sqrt();

        // If any leg is degenerate, fall back to simpler strategies.
        if leg1_len < 1e-3 && leg3_len < 1e-3 {
            // All corners are collapsed; draw a straight line.
            path.line_to(Point::new(bx as f64, by as f64));
            return;
        }
        if leg1_len < 1e-3 || leg2_len < 1e-3 {
            // First corner is degenerate; use two-leg for c1 -> b.
            Self::two_leg_segment_append(path, ax, ay, c2x, c2y, bx, by);
            return;
        }
        if leg3_len < 1e-3 {
            // Second corner is degenerate; use two-leg for a -> c1.
            Self::two_leg_segment_append(path, ax, ay, c1x, c1y, bx, by);
            return;
        }

        // === First corner arc (a -> c1 -> c2) === //
        let radius1 = ARC_RADIUS.min(leg1_len / 2.0).min(leg2_len / 2.0);
        let u1x = leg1_dx / leg1_len;
        let u1y = leg1_dy / leg1_len;
        let u2x = leg2_dx / leg2_len;
        let u2y = leg2_dy / leg2_len;

        let arc1_start_x = c1x - u1x * radius1;
        let arc1_start_y = c1y - u1y * radius1;
        let arc1_end_x = c1x + u2x * radius1;
        let arc1_end_y = c1y + u2y * radius1;

        // Line from current point to first arc start.
        path.line_to(Point::new(arc1_start_x as f64, arc1_start_y as f64));

        // First arc.
        let ctrl1a_x = arc1_start_x + u1x * radius1 * KAPPA;
        let ctrl1a_y = arc1_start_y + u1y * radius1 * KAPPA;
        let ctrl1b_x = arc1_end_x - u2x * radius1 * KAPPA;
        let ctrl1b_y = arc1_end_y - u2y * radius1 * KAPPA;

        path.curve_to(
            Point::new(ctrl1a_x as f64, ctrl1a_y as f64),
            Point::new(ctrl1b_x as f64, ctrl1b_y as f64),
            Point::new(arc1_end_x as f64, arc1_end_y as f64),
        );

        // === Second corner arc (c1 -> c2 -> b) === //
        let radius2 = ARC_RADIUS.min(leg2_len / 2.0).min(leg3_len / 2.0);
        // Reuse u2 for leg2 direction (already computed).
        let u3x = leg3_dx / leg3_len;
        let u3y = leg3_dy / leg3_len;

        let arc2_start_x = c2x - u2x * radius2;
        let arc2_start_y = c2y - u2y * radius2;
        let arc2_end_x = c2x + u3x * radius2;
        let arc2_end_y = c2y + u3y * radius2;

        // Line from first arc end to second arc start.
        path.line_to(Point::new(arc2_start_x as f64, arc2_start_y as f64));

        // Second arc.
        let ctrl2a_x = arc2_start_x + u2x * radius2 * KAPPA;
        let ctrl2a_y = arc2_start_y + u2y * radius2 * KAPPA;
        let ctrl2b_x = arc2_end_x - u3x * radius2 * KAPPA;
        let ctrl2b_y = arc2_end_y - u3y * radius2 * KAPPA;

        path.curve_to(
            Point::new(ctrl2a_x as f64, ctrl2a_y as f64),
            Point::new(ctrl2b_x as f64, ctrl2b_y as f64),
            Point::new(arc2_end_x as f64, arc2_end_y as f64),
        );

        // Line from second arc end to final point.
        path.line_to(Point::new(bx as f64, by as f64));
    }

    /// Computes the unit passthrough direction for a spacer.
    ///
    /// Identical to the curve builder's version -- direction from entry
    /// to exit.
    ///
    /// # Example values
    ///
    /// * Vertical spacer: entry `(150, 200)`, exit `(150, 205)` returns `(0.0,
    ///   1.0)`.
    fn spacer_passthrough_direction(spacer: &SpacerCoordinates) -> (f32, f32) {
        let dx = spacer.exit_x - spacer.entry_x;
        let dy = spacer.exit_y - spacer.entry_y;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1e-6 {
            (0.0, 1.0)
        } else {
            (dx / len, dy / len)
        }
    }
}

/// A waypoint in the orthogonal path: a coordinate and a direction.
///
/// The direction indicates the axis along which the path should
/// depart from or arrive at this point. Between consecutive waypoints,
/// the `connect_waypoints` function routes the path with orthogonal
/// legs and rounded corners.
///
/// # Example values
///
/// ```text
/// Waypoint { x: 150.0, y: 200.0, dir: (0.0, 1.0) }
/// ```
#[derive(Clone, Copy, Debug)]
struct Waypoint {
    /// X coordinate in pixels.
    x: f32,
    /// Y coordinate in pixels.
    y: f32,
    /// Unit direction vector indicating the departure/arrival axis.
    ///
    /// For node face protrusions, this is the face outward normal.
    /// For spacer boundaries, this is the spacer passthrough direction
    /// (or its negation, depending on path direction).
    dir: (f32, f32),
}
