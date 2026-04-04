use kurbo::{BezPath, Point};

use crate::taffy_to_svg_elements_mapper::{
    edge_model::NodeFace,
    edge_path_builder_pass_1::{EdgePathBuilderPass1, SpacerCoordinates, CURVE_CONTROL_RATIO},
    edge_path_builder_pass_2::FaceOrDirection,
};

/// Builds pass-2 edge paths using smooth bezier curves between spacers.
///
/// This handles the `EdgeCurvature::Curved` variant where segments
/// between spacers are drawn as cubic bezier curves whose control
/// points align with the face normals or spacer passthrough directions.
#[derive(Clone, Copy, Debug)]
pub(in crate::taffy_to_svg_elements_mapper) struct EdgePathBuilderPass2Curve;

impl EdgePathBuilderPass2Curve {
    /// Builds a smooth bezier path from `start` through spacers to `end`.
    ///
    /// The path structure is:
    ///
    /// 1. A curved segment from `end` (to-face) to the last spacer's exit.
    /// 2. A straight line through each spacer (exit to entry, in reversed
    ///    order).
    /// 3. Curved segments between adjacent spacers (connecting one spacer's
    ///    entry to the next spacer's exit).
    /// 4. A curved segment from the first spacer's entry to `start`
    ///    (from-face).
    ///
    /// The path is built in reverse order (from `end` to `start`) for
    /// correct SVG rendering direction, consistent with
    /// `build_curved_edge_path`.
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
    ) -> BezPath {
        let curve_ratio = CURVE_CONTROL_RATIO;

        // === Build the ordered list of curve/line segments === //
        //
        // The path is built in reverse (end -> start) for SVG rendering.
        // In reversed order the spacers are traversed last-to-first, and
        // within each spacer we go from exit to entry.
        //
        // Segment sequence (reversed):
        //   end -> last_spacer.exit  (curve)
        //   last_spacer.exit -> last_spacer.entry  (line)
        //   last_spacer.entry -> second_last_spacer.exit  (curve)
        //   ...
        //   first_spacer.exit -> first_spacer.entry  (line)
        //   first_spacer.entry -> start  (curve)

        let mut path = BezPath::new();
        path.move_to(Point::new(end_x as f64, end_y as f64));

        let spacer_count = spacers.len();

        // Iterate spacers in reverse (last spacer first in the reversed
        // path).
        for (rev_index, spacer) in spacers.iter().rev().enumerate() {
            // === Curve into spacer exit === //
            let curve_start_x;
            let curve_start_y;
            let curve_start_face_or_dir: FaceOrDirection;
            if rev_index == 0 {
                // First curve: from `end` (to-face).
                curve_start_x = end_x;
                curve_start_y = end_y;
                curve_start_face_or_dir = FaceOrDirection::Face(to_face);
            } else {
                // Curve from previous spacer's entry point.
                // The reversed path leaves the entry point going
                // opposite to the spacer's passthrough direction
                // (passthrough is entry -> exit; we leave entry going
                // away from exit).
                let prev_spacer = &spacers[spacer_count - rev_index];
                curve_start_x = prev_spacer.entry_x;
                curve_start_y = prev_spacer.entry_y;
                let (pdx, pdy) = Self::spacer_passthrough_direction(prev_spacer);
                curve_start_face_or_dir = FaceOrDirection::Direction((-pdx, -pdy));
            }

            // The curve arrives at spacer.exit. In the reversed path the
            // straight line through the spacer goes exit -> entry, which
            // is the reverse of the passthrough direction. The curve
            // should arrive at exit aligned with that reversed direction
            // so the transition into the straight line is smooth.
            let (sdx, sdy) = Self::spacer_passthrough_direction(spacer);
            Self::curve_segment_append(
                &mut path,
                curve_start_x,
                curve_start_y,
                spacer.exit_x,
                spacer.exit_y,
                curve_start_face_or_dir,
                FaceOrDirection::Direction((-sdx, -sdy)),
                curve_ratio,
            );

            // === Straight line through spacer (exit -> entry) === //
            path.line_to(Point::new(spacer.entry_x as f64, spacer.entry_y as f64));
        }

        // === Final curve from first spacer's entry to start === //
        // The reversed path leaves the entry point going opposite to
        // the spacer's passthrough direction.
        let first_spacer = &spacers[0];
        let (fdx, fdy) = Self::spacer_passthrough_direction(first_spacer);
        Self::curve_segment_append(
            &mut path,
            first_spacer.entry_x,
            first_spacer.entry_y,
            start_x,
            start_y,
            FaceOrDirection::Direction((-fdx, -fdy)),
            FaceOrDirection::Face(from_face),
            curve_ratio,
        );

        path
    }

    /// Appends a single cubic bezier curve segment to `path`.
    ///
    /// The segment goes from `(px, py)` to `(qx, qy)`. Control points
    /// are computed from the endpoint directions:
    ///
    /// * `p_dir` -- the direction the path should leave `(px, py)`. For a
    ///   `Face`, the outward normal is used. For a `Direction`, the unit vector
    ///   is used directly.
    /// * `q_dir` -- the direction the path should arrive at `(qx, qy)`. For a
    ///   `Face`, the outward normal is used (the control point is placed on the
    ///   outward side so the bezier arrives from that direction). For a
    ///   `Direction`, the unit vector is negated to produce an inward control
    ///   point.
    #[allow(clippy::too_many_arguments)]
    fn curve_segment_append(
        path: &mut BezPath,
        px: f32,
        py: f32,
        qx: f32,
        qy: f32,
        p_dir: FaceOrDirection,
        q_dir: FaceOrDirection,
        curve_ratio: f32,
    ) {
        let dx = qx - px;
        let dy = qy - py;
        let distance = (dx * dx + dy * dy).sqrt();
        let ctrl_distance = distance * curve_ratio;

        // Control point leaving p.
        let (c1x, c1y) = match p_dir {
            FaceOrDirection::Face(face) => {
                let (ox, oy) = EdgePathBuilderPass1::get_control_point_offset(face, ctrl_distance);
                (px + ox, py + oy)
            }
            FaceOrDirection::Direction((dir_x, dir_y)) => {
                (px + dir_x * ctrl_distance, py + dir_y * ctrl_distance)
            }
        };

        // Control point arriving at q.
        let (c2x, c2y) = match q_dir {
            FaceOrDirection::Face(face) => {
                // Place the control point on the outward side of the
                // face so the bezier approaches q from outside.
                let (ox, oy) = EdgePathBuilderPass1::get_control_point_offset(face, ctrl_distance);
                (qx + ox, qy + oy)
            }
            FaceOrDirection::Direction((dir_x, dir_y)) => {
                // Negate to get an inward control point (the bezier
                // should arrive along this direction).
                (qx - dir_x * ctrl_distance, qy - dir_y * ctrl_distance)
            }
        };

        path.curve_to(
            Point::new(c1x as f64, c1y as f64),
            Point::new(c2x as f64, c2y as f64),
            Point::new(qx as f64, qy as f64),
        );
    }

    /// Computes the unit passthrough direction for a spacer.
    ///
    /// The direction vector points from the spacer's entry to its exit.
    /// For a vertical spacer (entry and exit share the same x) this is
    /// `(0.0, 1.0)` or `(0.0, -1.0)`. For a horizontal spacer it is
    /// `(1.0, 0.0)` or `(-1.0, 0.0)`.
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
