use disposition_ir_model::edge::{Edge, EdgeId};
use kurbo::BezPath;

/// Represents a face/side of a rectangular node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NodeFace {
    Top,
    Bottom,
    Left,
    Right,
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
