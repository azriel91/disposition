use kurbo::{BezPath, PathEl, Point};

/// Length of the arrowhead from tip to base, in pixels.
const ARROW_HEAD_LENGTH: f64 = 8.0;

/// Half-width of the arrowhead (distance from centre line to each wing tip),
/// in pixels.
const ARROW_HEAD_HALF_WIDTH: f64 = 4.0;

/// Builds SVG path strings for arrowheads on edges.
///
/// Two kinds of arrowhead are produced:
///
/// * **Static (dependency edges)** – a closed V-shape positioned and rotated at
///   the `to` node end of the edge path.
/// * **Animated (interaction edges)** – a closed V-shape centred at the origin
///   pointing right, intended to be animated along the edge using CSS
///   `offset-path`.
#[derive(Clone, Copy, Debug)]
pub(super) struct ArrowHeadBuilder;

impl ArrowHeadBuilder {
    /// Returns a positioned arrowhead path string for a **dependency** edge.
    ///
    /// The arrowhead is a closed V whose tip sits at the `to` node end of the
    /// edge (the first point of the SVG path, since edge paths are built in
    /// reverse order).
    pub(super) fn build_static_arrow_head(edge_path: &BezPath) -> String {
        let (tip, direction) = Self::tip_and_direction(edge_path);

        // Normalise the direction vector.
        let len = (direction.x * direction.x + direction.y * direction.y).sqrt();
        if len < 1e-9 {
            // Degenerate – fall back to an invisible arrow.
            return String::new();
        }
        let dx = direction.x / len;
        let dy = direction.y / len;

        // Perpendicular (rotated 90° counter-clockwise).
        let px = -dy;
        let py = dx;

        // Wing points: step back along the direction and offset sideways.
        let wing1 = Point::new(
            tip.x - ARROW_HEAD_LENGTH * dx - ARROW_HEAD_HALF_WIDTH * px,
            tip.y - ARROW_HEAD_LENGTH * dy - ARROW_HEAD_HALF_WIDTH * py,
        );
        let wing2 = Point::new(
            tip.x - ARROW_HEAD_LENGTH * dx + ARROW_HEAD_HALF_WIDTH * px,
            tip.y - ARROW_HEAD_LENGTH * dy + ARROW_HEAD_HALF_WIDTH * py,
        );

        let mut path = BezPath::new();
        path.move_to(wing1);
        path.line_to(tip);
        path.line_to(wing2);
        path.close_path();

        path.to_svg()
    }

    /// Returns an origin-centred arrowhead path string for an **interaction**
    /// edge.
    ///
    /// The V-shape points in the +X direction so that CSS `offset-rotate: auto`
    /// will orient it correctly along the motion path.
    pub(super) fn build_origin_arrow_head() -> String {
        let mut path = BezPath::new();
        path.move_to(Point::new(-ARROW_HEAD_LENGTH, -ARROW_HEAD_HALF_WIDTH));
        path.line_to(Point::ZERO);
        path.line_to(Point::new(-ARROW_HEAD_LENGTH, ARROW_HEAD_HALF_WIDTH));
        path.close_path();

        path.to_svg()
    }

    // ------------------------------------------------------------------
    // Private helpers
    // ------------------------------------------------------------------

    /// Extracts the tip position and approach direction at the `to` node end of
    /// an edge path.
    ///
    /// The tip is the first point of the path (`MoveTo`).  The direction points
    /// *toward* the `to` node (i.e. opposite to the path's tangent at its
    /// start), which is the direction the arrowhead should face.
    fn tip_and_direction(edge_path: &BezPath) -> (Point, Point) {
        let elements = edge_path.elements();

        let tip = match elements.first() {
            Some(PathEl::MoveTo(p)) => *p,
            _ => return (Point::ORIGIN, Point::new(1.0, 0.0)),
        };

        // Tangent at the start of the first segment after the MoveTo.
        // For a cubic `M P0 C P1 P2 P3`, tangent at P0 = P1 − P0.
        // If P1 == P0 (degenerate), fall back to P2 − P0, then P3 − P0.
        let tangent = match elements.get(1) {
            Some(PathEl::CurveTo(p1, p2, p3)) => {
                let t = Point::new(p1.x - tip.x, p1.y - tip.y);
                if t.x.abs() > 1e-9 || t.y.abs() > 1e-9 {
                    t
                } else {
                    let t2 = Point::new(p2.x - tip.x, p2.y - tip.y);
                    if t2.x.abs() > 1e-9 || t2.y.abs() > 1e-9 {
                        t2
                    } else {
                        Point::new(p3.x - tip.x, p3.y - tip.y)
                    }
                }
            }
            Some(PathEl::LineTo(p)) => Point::new(p.x - tip.x, p.y - tip.y),
            Some(PathEl::QuadTo(p1, p2)) => {
                let t = Point::new(p1.x - tip.x, p1.y - tip.y);
                if t.x.abs() > 1e-9 || t.y.abs() > 1e-9 {
                    t
                } else {
                    Point::new(p2.x - tip.x, p2.y - tip.y)
                }
            }
            _ => Point::new(1.0, 0.0),
        };

        // The arrowhead should point *toward* the to-node, which is opposite
        // to the path's outgoing tangent at the start.
        let direction = Point::new(-tangent.x, -tangent.y);

        (tip, direction)
    }
}
