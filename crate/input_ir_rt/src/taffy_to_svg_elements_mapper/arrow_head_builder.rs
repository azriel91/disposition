use kurbo::{BezPath, PathEl, Point};

/// Length of the arrowhead from tip to base, in pixels.
///
/// Also consumed by `OrthoProtrusionCalculator` to keep the orthogonal Z/S
/// bend clear of the arrow head at the to-endpoint.
pub(super) const ARROW_HEAD_LENGTH: f64 = 8.0;

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
    /// edge (the last point of the SVG path, since edge paths run from the
    /// `from` node to the `to` node).
    pub(super) fn build_static_arrow_head(edge_path: &BezPath) -> BezPath {
        let (tip, direction) = Self::tip_and_direction(edge_path);

        // Normalise the direction vector.
        let len = (direction.x * direction.x + direction.y * direction.y).sqrt();
        if len < 1e-9 {
            // Degenerate – fall back to an invisible arrow.
            return BezPath::new();
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

        path
    }

    /// Returns an origin-centred arrowhead path string for an **interaction**
    /// edge.
    ///
    /// The V-shape points in the +X direction so that CSS `offset-rotate: auto`
    /// will orient it correctly along the motion path.
    pub(super) fn build_origin_arrow_head() -> BezPath {
        let mut path = BezPath::new();
        path.move_to(Point::new(-ARROW_HEAD_LENGTH, -ARROW_HEAD_HALF_WIDTH));
        path.line_to(Point::ZERO);
        path.line_to(Point::new(-ARROW_HEAD_LENGTH, ARROW_HEAD_HALF_WIDTH));
        path.close_path();

        path
    }

    // ------------------------------------------------------------------
    // Private helpers
    // ------------------------------------------------------------------

    /// Extracts the tip position and approach direction at the `to` node end of
    /// an edge path.
    ///
    /// The tip is the last point of the path (the final segment's endpoint).
    /// The direction is the path's incoming tangent at that point, which points
    /// *toward* the `to` node -- the direction the arrowhead should face.
    fn tip_and_direction(edge_path: &BezPath) -> (Point, Point) {
        let elements = edge_path.elements();

        // The point preceding the final segment, used as the tangent origin
        // when the last segment is a straight line.
        let prev_anchor = elements
            .len()
            .checked_sub(2)
            .and_then(|index| elements.get(index))
            .and_then(Self::element_anchor_point);

        // Tip is the endpoint of the final segment; the incoming tangent is the
        // vector from the last control point (or preceding anchor) to the tip.
        // For a cubic `... C P1 P2 P3`, tangent at P3 = P3 − P2. If P2 == P3
        // (degenerate), fall back to P3 − P1, then P3 − prev_anchor.
        let (tip, tangent) = match elements.last() {
            Some(PathEl::CurveTo(p1, p2, p3)) => {
                let tip = *p3;
                let t = Point::new(tip.x - p2.x, tip.y - p2.y);
                let tangent = if t.x.abs() > 1e-9 || t.y.abs() > 1e-9 {
                    t
                } else {
                    let t2 = Point::new(tip.x - p1.x, tip.y - p1.y);
                    if t2.x.abs() > 1e-9 || t2.y.abs() > 1e-9 {
                        t2
                    } else {
                        prev_anchor
                            .map(|prev| Point::new(tip.x - prev.x, tip.y - prev.y))
                            .unwrap_or(Point::new(1.0, 0.0))
                    }
                };
                (tip, tangent)
            }
            Some(PathEl::LineTo(p)) => {
                let tip = *p;
                let tangent = prev_anchor
                    .map(|prev| Point::new(tip.x - prev.x, tip.y - prev.y))
                    .unwrap_or(Point::new(1.0, 0.0));
                (tip, tangent)
            }
            Some(PathEl::QuadTo(p1, p2)) => {
                let tip = *p2;
                let t = Point::new(tip.x - p1.x, tip.y - p1.y);
                let tangent = if t.x.abs() > 1e-9 || t.y.abs() > 1e-9 {
                    t
                } else {
                    prev_anchor
                        .map(|prev| Point::new(tip.x - prev.x, tip.y - prev.y))
                        .unwrap_or(Point::new(1.0, 0.0))
                };
                (tip, tangent)
            }
            Some(PathEl::MoveTo(p)) => return (*p, Point::new(1.0, 0.0)),
            _ => return (Point::ORIGIN, Point::new(1.0, 0.0)),
        };

        // The incoming tangent already points *toward* the to-node, which is
        // the direction the arrowhead should face.
        (tip, tangent)
    }

    /// Returns the anchor (end) point of a path element, or `None` for
    /// `ClosePath` which has no anchor point of its own.
    fn element_anchor_point(element: &PathEl) -> Option<Point> {
        match element {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => Some(*p),
            PathEl::CurveTo(_, _, p) => Some(*p),
            PathEl::QuadTo(_, p) => Some(*p),
            PathEl::ClosePath => None,
        }
    }
}
