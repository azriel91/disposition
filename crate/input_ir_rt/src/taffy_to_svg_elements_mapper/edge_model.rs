use disposition_ir_model::{
    edge::{Edge, EdgeId},
    node::NodeId,
};
use kurbo::BezPath;

/// Represents a face/side of a rectangular node.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(super) enum NodeFace {
    Top,
    Bottom,
    Left,
    Right,
}

/// Identifies a specific face of a specific node.
///
/// Used as a map key to group edges that connect to the same face of
/// the same node, e.g. for spreading contact points or computing
/// offsets.
///
/// # Examples
///
/// ```text
/// NodeIdAndFace { node_id: "server", face: NodeFace::Right }
/// ```
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub(super) struct NodeIdAndFace<'id> {
    /// The node this face belongs to.
    pub(super) node_id: NodeId<'id>,
    /// Which face of the node.
    pub(super) face: NodeFace,
}

/// Mean anchor point of a `BezPath` in absolute SVG coordinates.
///
/// Computed from the MoveTo, LineTo, and final CurveTo / QuadTo points
/// of the path. Used during curvature-center sorting to determine how
/// tightly each edge curves relative to a common center.
///
/// # Examples
///
/// ```text
/// PathMidpoint { x: 150.0, y: 80.0 }
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct PathMidpoint {
    /// Mean x coordinate of the path's anchor points.
    pub(super) x: f64,
    /// Mean y coordinate of the path's anchor points.
    pub(super) y: f64,
}

/// Axis-aligned bounding box of a `BezPath`'s anchor points in absolute
/// SVG coordinates.
///
/// Computed from the same anchor points as `PathMidpoint` (MoveTo,
/// LineTo, and final CurveTo / QuadTo points). Used to determine the
/// extremal coordinates when computing the curvature center for
/// face-contact sorting.
///
/// # Examples
///
/// ```text
/// PathBounds { x_min: 100.0, x_max: 200.0, y_min: 50.0, y_max: 130.0 }
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct PathBounds {
    /// Minimum x coordinate among the path's anchor points.
    pub(super) x_min: f64,
    /// Maximum x coordinate among the path's anchor points.
    pub(super) x_max: f64,
    /// Minimum y coordinate among the path's anchor points.
    pub(super) y_min: f64,
    /// Maximum y coordinate among the path's anchor points.
    pub(super) y_max: f64,
}

/// Ordered pixel offsets for each edge contact point on a single node
/// face.
///
/// The offsets are distributed symmetrically around the face midpoint.
/// Slot 0 corresponds to the first contact in sorted order (nearest the
/// curvature center), and subsequent slots progress away from it.
///
/// # Examples
///
/// For 3 contacts with a 10 px gap: `[-10.0, 0.0, 10.0]`.
#[derive(Clone, Debug, Default)]
pub(super) struct EdgeContactPointOffsets(Vec<f32>);

impl EdgeContactPointOffsets {
    /// Creates a new `EdgeContactPointOffsets` from a vector of pixel
    /// offsets.
    pub(super) fn new(offsets: Vec<f32>) -> Self {
        Self(offsets)
    }

    /// Returns the offset at the given slot index, or `None` if out of
    /// bounds.
    pub(super) fn get(&self, slot: usize) -> Option<f32> {
        self.0.get(slot).copied()
    }

    /// Returns the largest absolute offset value, or `0.0` if the
    /// offsets are empty.
    ///
    /// # Example values
    ///
    /// For offsets `[-10.0, 0.0, 10.0]`, returns `10.0`.
    pub(super) fn max_abs(&self) -> f32 {
        self.0
            .iter()
            .map(|o| o.abs())
            .reduce(f32::max)
            .unwrap_or(0.0)
    }
}

/// Whether an edge represents an unpaired forward edge, or the request or
/// response of a pair of edges.
///
/// When two edges are paired, then their paths are offset from the midpoint of
/// the face of the node they are connected to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EdgeType {
    /// Forward direction of an unpaired edge.
    Unpaired,
    /// Request direction of a pair of edges.
    PairRequest,
    /// Response direction of a pair of edges.
    PairResponse,
}

/// Information needed to compute `SvgEdgeInfo`s including their animation.
///
/// This is collated because the sum of all path lengths in an edge group are
/// needed to compute the animation keyframe percentages for each edge.
#[derive(Clone, Debug)]
pub(super) struct EdgePathInfo<'edge, 'id> {
    pub(super) edge_id: EdgeId<'id>,
    pub(super) edge: &'edge Edge<'id>,
    pub(super) edge_type: EdgeType,
    pub(super) path: BezPath,
    pub(super) path_length: f64,
    pub(super) preceding_visible_segments_lengths: f64,
}

/// Parameters for edge `stroke-dasharray` animation generation.
///
/// These control how the decreasing visible segments in the dasharray are
/// computed and how the CSS keyframe animation is timed.
#[derive(Clone, Copy, Debug)]
pub(super) struct EdgeAnimationParams {
    /// Total length of visible segments plus inter-segment gaps.
    ///
    /// This does **not** include the trailing gap used to hide the edge.
    pub(super) visible_segments_length: f64,
    /// Constant gap width between adjacent visible segments.
    pub(super) gap_width: f64,
    /// Number of visible segments in the dasharray.
    pub(super) segment_count: usize,
    /// Geometric ratio for each successive segment (0 < ratio < 1).
    ///
    /// Each segment is `ratio` times the length of the previous one,
    /// producing a visually decreasing pattern.
    pub(super) segment_ratio: f64,
    /// Duration in seconds to pause (all edges invisible) before the
    /// animation cycle restarts.
    pub(super) pause_duration_secs: f64,
}

impl Default for EdgeAnimationParams {
    fn default() -> Self {
        Self {
            visible_segments_length: 100.0,
            gap_width: 2.0,
            segment_count: 8,
            segment_ratio: 0.6,
            pause_duration_secs: 0.5,
        }
    }
}

/// Result of computing edge animation data.
pub(super) struct EdgeAnimation {
    /// The stroke-dasharray value string, e.g. `"30.0,2.0,20.0,...,400.0"`.
    pub(super) dasharray: String,
    /// The CSS `@keyframes` rule for this edge's stroke-dashoffset animation.
    pub(super) keyframe_css: String,
    /// Unique animation name for the stroke-dashoffset keyframes rule.
    pub(super) animation_name: String,
    /// Total animation cycle duration in seconds.
    pub(super) edge_animation_duration_s: f64,
    /// The CSS `@keyframes` rule for the arrowhead offset + opacity animation.
    pub(super) arrow_head_keyframe_css: String,
    /// Unique animation name for the arrowhead keyframes rule.
    pub(super) arrow_head_animation_name: String,
}
