use kurbo::{BezPath, Point};

use crate::taffy_to_svg_elements_mapper::{
    edge_model::NodeFace, edge_path_builder_pass_1::SpacerCoordinates,
    edge_path_builder_pass_2::FaceOrDirection,
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

/// Protrusion lengths for the from-node and to-node endpoints of an
/// orthogonal edge path.
///
/// A protrusion is a short stub that exits the node face perpendicular
/// to the face line before the main orthogonal routing begins. This
/// separates parallel edges that share the same node face.
///
/// # Example values
///
/// ```text
/// OrthoProtrusionParams { from_protrusion: 12.0, to_protrusion: 8.0 }
/// ```
///
/// An edge whose from-node is close to the face midpoint gets a longer
/// `from_protrusion`; an edge further from the midpoint gets a shorter
/// one.
#[derive(Clone, Copy, Debug, Default)]
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
        protrusion: OrthoProtrusionParams,
    ) -> BezPath {
        let mut path = BezPath::new();

        // === To-node protrusion === //
        //
        // The path is built in reverse (end -> start). The to-node
        // protrusion is the first thing drawn: a short stub exiting the
        // to-node face perpendicular to the face line.
        let (eff_end_x, eff_end_y) = Self::protrusion_apply_start(
            &mut path,
            end_x,
            end_y,
            to_face,
            protrusion.to_protrusion,
        );

        let spacer_count = spacers.len();

        // Iterate spacers in reverse (last spacer first in the reversed path).
        for (rev_index, spacer) in spacers.iter().rev().enumerate() {
            // === Orthogonal segment into spacer exit === //
            let seg_start_x;
            let seg_start_y;
            let seg_start_dir: FaceOrDirection;
            if rev_index == 0 {
                seg_start_x = eff_end_x;
                seg_start_y = eff_end_y;
                seg_start_dir = FaceOrDirection::Face(to_face);
            } else {
                let prev_spacer = &spacers[spacer_count - rev_index];
                seg_start_x = prev_spacer.entry_x;
                seg_start_y = prev_spacer.entry_y;
                let (pdx, pdy) = Self::spacer_passthrough_direction(prev_spacer);
                seg_start_dir = FaceOrDirection::Direction((-pdx, -pdy));
            }

            let (sdx, sdy) = Self::spacer_passthrough_direction(spacer);
            Self::ortho_segment_append(
                &mut path,
                seg_start_x,
                seg_start_y,
                spacer.exit_x,
                spacer.exit_y,
                seg_start_dir,
                FaceOrDirection::Direction((-sdx, -sdy)),
            );

            // === Straight line through spacer (exit -> entry) === //
            path.line_to(Point::new(spacer.entry_x as f64, spacer.entry_y as f64));
        }

        // === Final orthogonal segment from first spacer's entry to start === //
        let first_spacer = &spacers[0];
        let (fdx, fdy) = Self::spacer_passthrough_direction(first_spacer);

        // Compute the effective start point (after from-node protrusion).
        let (eff_start_x, eff_start_y) =
            Self::protrusion_offset(start_x, start_y, from_face, protrusion.from_protrusion);

        Self::ortho_segment_append(
            &mut path,
            first_spacer.entry_x,
            first_spacer.entry_y,
            eff_start_x,
            eff_start_y,
            FaceOrDirection::Direction((-fdx, -fdy)),
            FaceOrDirection::Face(from_face),
        );

        // === From-node protrusion === //
        //
        // Draw from the effective start back to the actual start
        // (protrusion stub entering the from-node).
        Self::protrusion_apply_end(&mut path, start_x, start_y, protrusion.from_protrusion);

        path
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
        protrusion: OrthoProtrusionParams,
    ) -> BezPath {
        let mut path = BezPath::new();

        // === To-node protrusion === //
        let (eff_end_x, eff_end_y) = Self::protrusion_apply_start(
            &mut path,
            end_x,
            end_y,
            to_face,
            protrusion.to_protrusion,
        );

        // Compute the effective start point (after from-node protrusion).
        let (eff_start_x, eff_start_y) =
            Self::protrusion_offset(start_x, start_y, from_face, protrusion.from_protrusion);

        // Path is built in reverse (end -> start) for SVG rendering.
        Self::ortho_segment_append(
            &mut path,
            eff_end_x,
            eff_end_y,
            eff_start_x,
            eff_start_y,
            FaceOrDirection::Face(to_face),
            FaceOrDirection::Face(from_face),
        );

        // === From-node protrusion === //
        Self::protrusion_apply_end(&mut path, start_x, start_y, protrusion.from_protrusion);

        path
    }

    /// Applies a protrusion at the start of the path (the to-node
    /// endpoint, since the path is built in reverse).
    ///
    /// Moves to the node contact point, draws a short perpendicular
    /// stub outward from the face, and returns the effective endpoint
    /// (the tip of the protrusion) that the main routing should start
    /// from.
    ///
    /// When `protrusion_len` is zero or near-zero, the path simply
    /// starts at the contact point and no stub is drawn.
    fn protrusion_apply_start(
        path: &mut BezPath,
        x: f32,
        y: f32,
        face: NodeFace,
        protrusion_len: f32,
    ) -> (f32, f32) {
        path.move_to(Point::new(x as f64, y as f64));

        if protrusion_len < 1e-3 {
            return (x, y);
        }

        let (eff_x, eff_y) = Self::protrusion_offset(x, y, face, protrusion_len);
        path.line_to(Point::new(eff_x as f64, eff_y as f64));
        (eff_x, eff_y)
    }

    /// Applies a protrusion at the end of the path (the from-node
    /// endpoint, since the path is built in reverse).
    ///
    /// Draws a line from the current path position to the actual node
    /// contact point, closing the protrusion stub.
    ///
    /// When `protrusion_len` is zero or near-zero, no extra line is
    /// drawn (the main routing already ends at the contact point).
    fn protrusion_apply_end(path: &mut BezPath, x: f32, y: f32, protrusion_len: f32) {
        if protrusion_len < 1e-3 {
            return;
        }

        path.line_to(Point::new(x as f64, y as f64));
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

    /// Appends an orthogonal segment from `(px, py)` to `(qx, qy)`.
    ///
    /// The segment consists of up to two straight legs joined by a
    /// rounded 90-degree arc corner. The departure direction at `p` and
    /// arrival direction at `q` are determined by `p_dir` and `q_dir`.
    ///
    /// If the two points are already axis-aligned in the departure
    /// direction, a single straight line is emitted.
    fn ortho_segment_append(
        path: &mut BezPath,
        px: f32,
        py: f32,
        qx: f32,
        qy: f32,
        p_dir: FaceOrDirection,
        q_dir: FaceOrDirection,
    ) {
        let (p_dx, p_dy) = Self::direction_vector(p_dir);
        let (_q_dx, _q_dy) = Self::direction_vector(q_dir);

        let p_is_vertical = p_dy.abs() > p_dx.abs();

        if p_is_vertical {
            // First leg is vertical, second leg is horizontal.
            // Corner is at (px, qy) -- go vertically to qy, then
            // horizontally to qx.
            let corner_x = px;
            let corner_y = qy;

            Self::two_leg_segment_append(path, px, py, corner_x, corner_y, qx, qy);
        } else {
            // First leg is horizontal, second leg is vertical.
            // Corner is at (qx, py) -- go horizontally to qx, then
            // vertically to qy.
            let corner_x = qx;
            let corner_y = py;

            Self::two_leg_segment_append(path, px, py, corner_x, corner_y, qx, qy);
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

    /// Extracts a unit direction vector from a `FaceOrDirection`.
    ///
    /// For `Face` variants, returns the outward normal direction.
    /// For `Direction` variants, returns the stored unit vector.
    fn direction_vector(face_or_dir: FaceOrDirection) -> (f32, f32) {
        match face_or_dir {
            FaceOrDirection::Face(NodeFace::Top) => (0.0, -1.0),
            FaceOrDirection::Face(NodeFace::Bottom) => (0.0, 1.0),
            FaceOrDirection::Face(NodeFace::Left) => (-1.0, 0.0),
            FaceOrDirection::Face(NodeFace::Right) => (1.0, 0.0),
            FaceOrDirection::Direction((dx, dy)) => (dx, dy),
        }
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
