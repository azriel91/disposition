use disposition_svg_model::SvgNodeInfo;
use disposition_taffy_model::TEXT_LINE_HEIGHT;
use kurbo::{BezPath, Point};

use super::edge_model::{EdgeType, NodeFace};

/// Per-endpoint face offset in pixels, applied perpendicular to the
/// face normal (i.e. along the face).
///
/// Positive values shift right / down; negative values shift left / up.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct EdgeFaceOffset {
    /// Pixel offset applied to the "from" node's contact point.
    pub(super) from_offset: f32,
    /// Pixel offset applied to the "to" node's contact point.
    pub(super) to_offset: f32,
}

/// Absolute coordinates of a spacer node's entry and exit edges,
/// slicing the spacer in half so that the edge path is perfectly
/// horizontal or vertical while passing through.
///
/// Coordinates are ordered from the from-node towards the to-node:
/// the path enters at `(entry_x, entry_y)` and exits at
/// `(exit_x, exit_y)`.
///
/// # Example values
///
/// Vertical passthrough (ranks stacked top-to-bottom):
///
/// ```text
/// SpacerCoordinates { entry_x: 150.0, entry_y: 200.0,
///                     exit_x:  150.0, exit_y:  205.0 }
/// ```
///
/// Horizontal passthrough (ranks stacked left-to-right):
///
/// ```text
/// SpacerCoordinates { entry_x: 200.0, entry_y: 150.0,
///                     exit_x:  205.0, exit_y:  150.0 }
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SpacerCoordinates {
    /// X coordinate where the path enters the spacer.
    pub(super) entry_x: f32,
    /// Y coordinate where the path enters the spacer.
    pub(super) entry_y: f32,
    /// X coordinate where the path exits the spacer.
    pub(super) exit_x: f32,
    /// Y coordinate where the path exits the spacer.
    pub(super) exit_y: f32,
}

/// Represents the connection geometry for an edge endpoint on a node.
/// Either the standard rectangular face, or a circle perimeter point.
#[derive(Clone, Copy, Debug)]
enum NodeEdgeGeometry {
    /// Standard rectangular face connection.
    Rect,
    /// Circle connection: the edge should connect to the perimeter of the
    /// circle at the point closest to the other endpoint.
    Circle {
        /// Absolute x of the circle center.
        cx: f32,
        /// Absolute y of the circle center.
        cy: f32,
        /// Radius of the circle.
        radius: f32,
    },
}

// Constants for edge layout

/// Percentage of the node's width to offset the edge's x coordinate
/// from the midpoint of the node.
const SELF_LOOP_X_OFFSET_RATIO: f32 = 0.2;
/// Percentage of the node's height to extend the edge vertically.
const SELF_LOOP_Y_EXTENSION_RATIO: f32 = 0.2;
/// Percentage of the node's width to curve the edge horizontally
/// outward.
const SELF_LOOP_X_EXTENSION_RATIO: f32 = 0.2;
/// Percentage of the node's width/height to offset the edge when
/// connecting to another edge.
const BIDIRECTIONAL_OFFSET_RATIO: f32 = 0.1;
/// Percentage of the node's width/height to curve the edge outward.
const CURVE_CONTROL_RATIO: f32 = 0.3;

/// Builds SVG bezier curve paths for edges connecting two nodes.
#[derive(Clone, Copy, Debug)]
pub(super) struct EdgePathBuilder;

impl EdgePathBuilder {
    /// Builds the SVG path `d` attribute for an edge between two nodes.
    ///
    /// The path is a curved bezier curve that connects the appropriate faces
    /// of the source and target nodes based on their relative positions.
    ///
    /// This is a convenience wrapper around `build_with_offsets` with zero
    /// offsets.
    pub(super) fn build(
        from_info: &SvgNodeInfo,
        to_info: &SvgNodeInfo,
        edge_type: EdgeType,
    ) -> BezPath {
        Self::build_with_offsets(from_info, to_info, edge_type, EdgeFaceOffset::default())
    }

    /// Builds the SVG path with per-face contact point offsets.
    ///
    /// `face_offset.from_offset` shifts the "from" contact point along
    /// the face (perpendicular to its outward normal). Likewise for
    /// `face_offset.to_offset`.
    pub(super) fn build_with_offsets(
        from_info: &SvgNodeInfo,
        to_info: &SvgNodeInfo,
        edge_type: EdgeType,
        face_offset: EdgeFaceOffset,
    ) -> BezPath {
        // Handle self-loop case
        if from_info.node_id == to_info.node_id {
            return Self::build_self_loop_path(
                from_info,
                edge_type,
                SELF_LOOP_X_OFFSET_RATIO,
                SELF_LOOP_Y_EXTENSION_RATIO,
                SELF_LOOP_X_EXTENSION_RATIO,
            );
        }

        // Determine circle geometry for from/to nodes
        let from_geom = Self::node_edge_geometry(from_info);
        let to_geom = Self::node_edge_geometry(to_info);

        // Determine which faces to use based on relative positions
        let (from_face, to_face) = Self::select_edge_faces(from_info, to_info);

        // Check if from is contained inside to
        let from_contained_in_to = Self::is_node_contained_in(from_info, to_info);
        if from_contained_in_to {
            return Self::build_contained_edge_path(from_info, to_info, CURVE_CONTROL_RATIO);
        }

        // Get base connection points
        let (mut start_x, mut start_y) = Self::get_face_center(from_info, from_face);
        let (mut end_x, mut end_y) = Self::get_face_center(to_info, to_face);

        // Apply face contact offsets (spread edges along the face).
        Self::face_offset_apply(
            &mut start_x,
            &mut start_y,
            from_face,
            face_offset.from_offset,
        );
        Self::face_offset_apply(&mut end_x, &mut end_y, to_face, face_offset.to_offset);

        // Apply bidirectional offset
        if edge_type == EdgeType::PairRequest || edge_type == EdgeType::PairResponse {
            let offset_direction = if edge_type == EdgeType::PairResponse {
                1.0
            } else {
                -1.0
            };

            // Move start point down if this is the `PairRequest` edge.
            match from_face {
                NodeFace::Right | NodeFace::Left => {
                    start_y +=
                        from_info.height_collapsed * BIDIRECTIONAL_OFFSET_RATIO * offset_direction;
                }
                NodeFace::Top | NodeFace::Bottom => {
                    start_x += from_info.width * BIDIRECTIONAL_OFFSET_RATIO * offset_direction;
                }
            }

            // Move end point down if this is the `PairResponse` edge.
            match to_face {
                NodeFace::Right | NodeFace::Left => {
                    end_y +=
                        to_info.height_collapsed * BIDIRECTIONAL_OFFSET_RATIO * offset_direction;
                }
                NodeFace::Top | NodeFace::Bottom => {
                    end_x += to_info.width * BIDIRECTIONAL_OFFSET_RATIO * offset_direction;
                }
            }
        }

        // If either node has a circle, snap the connection point to the circle
        // perimeter instead of the rectangular face center.
        if let NodeEdgeGeometry::Circle { cx, cy, radius } = from_geom {
            let (sx, sy) = Self::circle_perimeter_point(cx, cy, radius, end_x, end_y);
            start_x = sx;
            start_y = sy;
        }
        if let NodeEdgeGeometry::Circle { cx, cy, radius } = to_geom {
            let (ex, ey) = Self::circle_perimeter_point(cx, cy, radius, start_x, start_y);
            end_x = ex;
            end_y = ey;
        }

        // Build curved path
        Self::build_curved_edge_path(
            start_x,
            start_y,
            end_x,
            end_y,
            from_face,
            to_face,
            CURVE_CONTROL_RATIO,
        )
    }

    /// Builds the SVG path with per-face contact point offsets and
    /// intermediate spacer passthrough segments.
    ///
    /// When `spacers` is empty this delegates to `build_with_offsets`.
    /// Otherwise the path curves from the from-node face to the first
    /// spacer, passes straight through each spacer, curves between
    /// spacers, and curves from the last spacer to the to-node face.
    ///
    /// Self-loops and contained-edge special cases ignore spacers.
    ///
    /// # Parameters
    ///
    /// * `spacers` - Intermediate spacer coordinates the edge must pass
    ///   through, e.g. `&[SpacerCoordinates { entry_x: 150.0, entry_y: 200.0,
    ///   exit_x: 150.0, exit_y: 205.0 }]`.
    pub(super) fn build_with_offsets_and_spacers(
        from_info: &SvgNodeInfo,
        to_info: &SvgNodeInfo,
        edge_type: EdgeType,
        face_offset: EdgeFaceOffset,
        spacers: &[SpacerCoordinates],
    ) -> BezPath {
        if spacers.is_empty() {
            return Self::build_with_offsets(from_info, to_info, edge_type, face_offset);
        }

        // Handle self-loop case (waypoints ignored)
        if from_info.node_id == to_info.node_id {
            return Self::build_self_loop_path(
                from_info,
                edge_type,
                SELF_LOOP_X_OFFSET_RATIO,
                SELF_LOOP_Y_EXTENSION_RATIO,
                SELF_LOOP_X_EXTENSION_RATIO,
            );
        }

        // Determine circle geometry for from/to nodes
        let from_geom = Self::node_edge_geometry(from_info);
        let to_geom = Self::node_edge_geometry(to_info);

        // Determine which faces to use based on relative positions
        let (from_face, to_face) = Self::select_edge_faces(from_info, to_info);

        // Check if from is contained inside to (waypoints ignored)
        let from_contained_in_to = Self::is_node_contained_in(from_info, to_info);
        if from_contained_in_to {
            return Self::build_contained_edge_path(from_info, to_info, CURVE_CONTROL_RATIO);
        }

        // Get base connection points
        let (mut start_x, mut start_y) = Self::get_face_center(from_info, from_face);
        let (mut end_x, mut end_y) = Self::get_face_center(to_info, to_face);

        // Apply face contact offsets (spread edges along the face).
        Self::face_offset_apply(
            &mut start_x,
            &mut start_y,
            from_face,
            face_offset.from_offset,
        );
        Self::face_offset_apply(&mut end_x, &mut end_y, to_face, face_offset.to_offset);

        // Apply bidirectional offset
        if edge_type == EdgeType::PairRequest || edge_type == EdgeType::PairResponse {
            let offset_direction = if edge_type == EdgeType::PairResponse {
                1.0
            } else {
                -1.0
            };

            // Move start point down if this is the `PairRequest` edge.
            match from_face {
                NodeFace::Right | NodeFace::Left => {
                    start_y +=
                        from_info.height_collapsed * BIDIRECTIONAL_OFFSET_RATIO * offset_direction;
                }
                NodeFace::Top | NodeFace::Bottom => {
                    start_x += from_info.width * BIDIRECTIONAL_OFFSET_RATIO * offset_direction;
                }
            }

            // Move end point down if this is the `PairResponse` edge.
            match to_face {
                NodeFace::Right | NodeFace::Left => {
                    end_y +=
                        to_info.height_collapsed * BIDIRECTIONAL_OFFSET_RATIO * offset_direction;
                }
                NodeFace::Top | NodeFace::Bottom => {
                    end_x += to_info.width * BIDIRECTIONAL_OFFSET_RATIO * offset_direction;
                }
            }
        }

        // If either node has a circle, snap the connection point to the circle
        // perimeter instead of the rectangular face center.
        if let NodeEdgeGeometry::Circle { cx, cy, radius } = from_geom {
            let (sx, sy) = Self::circle_perimeter_point(cx, cy, radius, end_x, end_y);
            start_x = sx;
            start_y = sy;
        }
        if let NodeEdgeGeometry::Circle { cx, cy, radius } = to_geom {
            let (ex, ey) = Self::circle_perimeter_point(cx, cy, radius, start_x, start_y);
            end_x = ex;
            end_y = ey;
        }

        // Build spacer path
        Self::build_spacer_edge_path(
            start_x,
            start_y,
            end_x,
            end_y,
            from_face,
            to_face,
            spacers,
            CURVE_CONTROL_RATIO,
        )
    }

    /// Returns the (from_face, to_face) that would be selected for an
    /// edge between two nodes, without building the full path.
    ///
    /// For self-loops both faces are `NodeFace::Bottom`.
    /// For contained edges both faces are `None` (no face-based offset
    /// applies).
    pub(super) fn faces_select(
        from_info: &SvgNodeInfo,
        to_info: &SvgNodeInfo,
    ) -> Option<(NodeFace, NodeFace)> {
        if from_info.node_id == to_info.node_id {
            // Self-loop: both endpoints touch the bottom face.
            return Some((NodeFace::Bottom, NodeFace::Bottom));
        }
        if Self::is_node_contained_in(from_info, to_info) {
            // Contained edges bypass face-based contact points.
            return None;
        }
        Some(Self::select_edge_faces(from_info, to_info))
    }

    /// Applies a pixel offset along a face.
    ///
    /// For `Left` / `Right` faces the offset shifts vertically.
    /// For `Top` / `Bottom` faces the offset shifts horizontally.
    fn face_offset_apply(x: &mut f32, y: &mut f32, face: NodeFace, offset: f32) {
        match face {
            NodeFace::Left | NodeFace::Right => *y += offset,
            NodeFace::Top | NodeFace::Bottom => *x += offset,
        }
    }

    /// Returns the edge connection geometry for a node.
    ///
    /// If the node has a circle, returns `NodeEdgeGeometry::Circle` with
    /// the circle's absolute center and radius. Otherwise returns
    /// `NodeEdgeGeometry::Rect`.
    fn node_edge_geometry(node_info: &SvgNodeInfo) -> NodeEdgeGeometry {
        if let Some(ref circle) = node_info.circle {
            NodeEdgeGeometry::Circle {
                cx: node_info.x + circle.cx,
                cy: node_info.y + circle.cy,
                radius: circle.radius,
            }
        } else {
            NodeEdgeGeometry::Rect
        }
    }

    /// Returns the point on a circle's perimeter closest to a target point.
    ///
    /// Given a circle at `(cx, cy)` with `radius`, computes the point on
    /// its perimeter that lies on the line from the center to
    /// `(target_x, target_y)`.
    fn circle_perimeter_point(
        cx: f32,
        cy: f32,
        radius: f32,
        target_x: f32,
        target_y: f32,
    ) -> (f32, f32) {
        let dx = target_x - cx;
        let dy = target_y - cy;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < f32::EPSILON {
            // Target is at the center; default to rightward
            (cx + radius, cy)
        } else {
            let ratio = radius / dist;
            (cx + dx * ratio, cy + dy * ratio)
        }
    }

    /// Builds a self-loop path that goes from the bottom of a node, extends
    /// down, curves left, and returns to the bottom of the same node.
    fn build_self_loop_path(
        node_info: &SvgNodeInfo,
        edge_type: EdgeType,
        x_offset_ratio: f32,
        y_extension_ratio: f32,
        x_extension_ratio: f32,
    ) -> BezPath {
        let start_x = node_info.x + node_info.width * (0.5 + x_offset_ratio);
        let start_y = node_info.y + node_info.height_collapsed;
        let end_x = node_info.x + node_info.width * (0.5 - x_offset_ratio);
        let end_y = start_y;

        let extension_y = TEXT_LINE_HEIGHT.max(node_info.height_collapsed * y_extension_ratio);
        let extension_x = node_info.width * x_extension_ratio;

        let start = Point::new(start_x as f64, start_y as f64);

        // Control points for the self-loop curve
        let ctrl1 = Point::new(
            (start_x + extension_x * 0.5) as f64,
            (start_y + extension_y) as f64,
        );
        let mid = Point::new(
            (node_info.x + node_info.width * 0.5) as f64,
            (start_y + extension_y) as f64,
        );

        let ctrl3 = Point::new(
            (end_x - extension_x * 0.5) as f64,
            (start_y + extension_y) as f64,
        );
        let end = Point::new(end_x as f64, end_y as f64);

        // Paths have to be built in reverse to get them to render in the correct
        // direction in the SVG.
        let mut path = BezPath::new();
        match edge_type {
            EdgeType::Unpaired | EdgeType::PairRequest => {
                path.move_to(end);
                path.curve_to(end, ctrl3, mid);
                path.curve_to(mid, ctrl1, start);
            }
            EdgeType::PairResponse => {
                path.move_to(start);
                path.curve_to(start, ctrl1, mid);
                path.curve_to(mid, ctrl3, end);
            }
        }

        path
    }

    /// Builds a path for an edge where the source node is contained inside the
    /// target node.
    fn build_contained_edge_path(
        from_info: &SvgNodeInfo,
        to_info: &SvgNodeInfo,
        curve_ratio: f32,
    ) -> BezPath {
        // Start from bottom of from node
        let start_x = from_info.x + from_info.width * 0.5;
        let start_y = from_info.y + from_info.height_collapsed;

        // End at left face of to node
        let end_x = to_info.x;
        let end_y = to_info.y + to_info.height_collapsed * 0.5;

        // Control points: go down, then left, then up
        let ctrl_distance = (start_y - end_y).abs().max(from_info.width) * curve_ratio;

        let ctrl1 = Point::new(start_x as f64, (start_y + ctrl_distance) as f64);
        let ctrl2 = Point::new((end_x - ctrl_distance) as f64, end_y as f64);
        let end = Point::new(end_x as f64, end_y as f64);

        // Paths have to be built in reverse to get them to render in the correct
        // direction in the SVG.
        let mut path = BezPath::new();
        let start = Point::new(start_x as f64, start_y as f64);
        path.move_to(end);
        path.curve_to(ctrl2, ctrl1, start);

        path
    }

    /// Selects the appropriate faces for connecting two nodes based on their
    /// relative positions, choosing the faces that produce the shortest path.
    fn select_edge_faces(from_info: &SvgNodeInfo, to_info: &SvgNodeInfo) -> (NodeFace, NodeFace) {
        let from_center_x = from_info.x + from_info.width / 2.0;
        let from_center_y = from_info.y + from_info.height_collapsed / 2.0;
        let to_center_x = to_info.x + to_info.width / 2.0;
        let to_center_y = to_info.y + to_info.height_collapsed / 2.0;

        let dx = to_center_x - from_center_x;
        let dy = to_center_y - from_center_y;

        // Check for clear horizontal or vertical alignment
        let from_right = from_info.x + from_info.width;
        let to_right = to_info.x + to_info.width;
        let from_bottom = from_info.y + from_info.height_collapsed;
        let to_bottom = to_info.y + to_info.height_collapsed;

        // Node is clearly to the right (no horizontal overlap)
        if from_right < to_info.x {
            if from_bottom < to_info.y {
                // Diagonal: from is above-left of to
                return Self::select_diagonal_faces(
                    from_info,
                    to_info,
                    NodeFace::Right,
                    NodeFace::Bottom,
                    NodeFace::Left,
                    NodeFace::Top,
                );
            } else if from_info.y > to_bottom {
                // Diagonal: from is below-left of to
                return Self::select_diagonal_faces(
                    from_info,
                    to_info,
                    NodeFace::Right,
                    NodeFace::Top,
                    NodeFace::Left,
                    NodeFace::Bottom,
                );
            }
            return (NodeFace::Right, NodeFace::Left);
        }

        // Node is clearly to the left (no horizontal overlap)
        if to_right < from_info.x {
            if from_bottom < to_info.y {
                // Diagonal: from is above-right of to
                return Self::select_diagonal_faces(
                    from_info,
                    to_info,
                    NodeFace::Left,
                    NodeFace::Bottom,
                    NodeFace::Right,
                    NodeFace::Top,
                );
            } else if from_info.y > to_bottom {
                // Diagonal: from is below-right of to
                return Self::select_diagonal_faces(
                    from_info,
                    to_info,
                    NodeFace::Left,
                    NodeFace::Top,
                    NodeFace::Right,
                    NodeFace::Bottom,
                );
            }
            return (NodeFace::Left, NodeFace::Right);
        }

        // Node is clearly below (no vertical overlap but horizontal overlap)
        if from_bottom < to_info.y {
            return (NodeFace::Bottom, NodeFace::Top);
        }

        // Node is clearly above (no vertical overlap but horizontal overlap)
        if to_bottom < from_info.y {
            return (NodeFace::Top, NodeFace::Bottom);
        }

        // Overlapping nodes - use primary direction
        if dx.abs() > dy.abs() {
            if dx > 0.0 {
                (NodeFace::Right, NodeFace::Left)
            } else {
                (NodeFace::Left, NodeFace::Right)
            }
        } else if dy > 0.0 {
            (NodeFace::Bottom, NodeFace::Top)
        } else {
            (NodeFace::Top, NodeFace::Bottom)
        }
    }

    /// Selects the best faces for diagonal connections by comparing distances.
    fn select_diagonal_faces(
        from_info: &SvgNodeInfo,
        to_info: &SvgNodeInfo,
        from_horiz: NodeFace,
        from_vert: NodeFace,
        to_horiz: NodeFace,
        to_vert: NodeFace,
    ) -> (NodeFace, NodeFace) {
        // Calculate distances for horizontal-to-vertical vs vertical-to-horizontal
        let (from_h_x, from_h_y) = Self::get_face_center(from_info, from_horiz);
        let (to_v_x, to_v_y) = Self::get_face_center(to_info, to_vert);
        let dist_h_to_v = ((to_v_x - from_h_x).powi(2) + (to_v_y - from_h_y).powi(2)).sqrt();

        let (from_v_x, from_v_y) = Self::get_face_center(from_info, from_vert);
        let (to_h_x, to_h_y) = Self::get_face_center(to_info, to_horiz);
        let dist_v_to_h = ((to_h_x - from_v_x).powi(2) + (to_h_y - from_v_y).powi(2)).sqrt();

        if dist_h_to_v <= dist_v_to_h {
            (from_horiz, to_vert)
        } else {
            (from_vert, to_horiz)
        }
    }

    /// Gets the center point of a node's face.
    fn get_face_center(node_info: &SvgNodeInfo, face: NodeFace) -> (f32, f32) {
        match face {
            NodeFace::Top => (node_info.x + node_info.width / 2.0, node_info.y),
            NodeFace::Bottom => (
                node_info.x + node_info.width / 2.0,
                node_info.y + node_info.height_collapsed,
            ),
            NodeFace::Left => (node_info.x, node_info.y + node_info.height_collapsed / 2.0),
            NodeFace::Right => (
                node_info.x + node_info.width,
                node_info.y + node_info.height_collapsed / 2.0,
            ),
        }
    }

    /// Checks if a node is geometrically contained within another node.
    fn is_node_contained_in(inner: &SvgNodeInfo, outer: &SvgNodeInfo) -> bool {
        inner.x >= outer.x
            && inner.y >= outer.y
            && inner.x + inner.width <= outer.x + outer.width
            && inner.y + inner.height_collapsed <= outer.y + outer.height_collapsed
    }

    /// Builds a curved bezier path between two points with control points
    /// based on the faces being connected.
    fn build_curved_edge_path(
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        from_face: NodeFace,
        to_face: NodeFace,
        curve_ratio: f32,
    ) -> BezPath {
        let dx = end_x - start_x;
        let dy = end_y - start_y;
        let distance = (dx * dx + dy * dy).sqrt();
        let ctrl_distance = distance * curve_ratio;

        // Calculate control points based on face directions
        let start = Point::new(start_x as f64, start_y as f64);
        let (ctrl1_x, ctrl1_y) = Self::get_control_point_offset(from_face, ctrl_distance);
        let (ctrl2_x, ctrl2_y) = Self::get_control_point_offset(to_face, ctrl_distance);
        let ctrl1 = Point::new((start_x + ctrl1_x) as f64, (start_y + ctrl1_y) as f64);
        let ctrl2 = Point::new((end_x + ctrl2_x) as f64, (end_y + ctrl2_y) as f64);
        let end = Point::new(end_x as f64, end_y as f64);

        // Paths have to be built in reverse to get them to render in the correct
        // direction in the SVG.
        let mut path = BezPath::new();
        path.move_to(end);
        path.curve_to(ctrl2, ctrl1, start);

        path
    }

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
    /// * `curve_ratio = 0.3`
    fn build_spacer_edge_path(
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        from_face: NodeFace,
        to_face: NodeFace,
        spacers: &[SpacerCoordinates],
        curve_ratio: f32,
    ) -> BezPath {
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
            Self::build_spacer_edge_path_curve_segment(
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
        Self::build_spacer_edge_path_curve_segment(
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
    fn build_spacer_edge_path_curve_segment(
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
                let (ox, oy) = Self::get_control_point_offset(face, ctrl_distance);
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
                let (ox, oy) = Self::get_control_point_offset(face, ctrl_distance);
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

    /// Gets the control point offset direction based on the face.
    fn get_control_point_offset(face: NodeFace, distance: f32) -> (f32, f32) {
        match face {
            NodeFace::Top => (0.0, -distance),
            NodeFace::Bottom => (0.0, distance),
            NodeFace::Left => (-distance, 0.0),
            NodeFace::Right => (distance, 0.0),
        }
    }
}

/// Direction specification for a curve endpoint: either an outward
/// node face normal or an explicit unit direction vector.
///
/// Used by `build_spacer_edge_path_curve_segment` to compute control
/// points.
#[derive(Clone, Copy, Debug)]
enum FaceOrDirection {
    /// Outward normal of a node face (e.g. `NodeFace::Bottom`).
    Face(NodeFace),
    /// Explicit unit direction vector, e.g. `(0.0, 1.0)` for downward.
    Direction((f32, f32)),
}
