use kurbo::{stroke, BezPath, Cap, Join, Stroke, StrokeOpts};

/// Width of the stroke expansion used to compute the edge locus, in pixels.
const LOCUS_STROKE_WIDTH: f64 = 8.0;

/// Accuracy tolerance for path approximation when computing the locus.
const LOCUS_TOLERANCE: f64 = 0.1;

/// Computes the locus (parallel curve / offset curve) around an edge path and
/// its arrow head, for use as a focus indicator.
///
/// The locus is the outline of a stroke expansion that wraps both the edge
/// body and the arrow head, suitable for rendering as a dashed highlight when
/// the edge is focused.
#[derive(Clone, Copy, Debug)]
pub(super) struct EdgePathLocusCalculator;

impl EdgePathLocusCalculator {
    /// Computes the locus `BezPath` for the given edge body path and arrow
    /// head path.
    ///
    /// The two paths are chained and then expanded via [`kurbo::stroke`] to
    /// produce a filled shape whose outline represents the parallel curves
    /// around the combined edge and arrow head.
    ///
    /// # Parameters
    ///
    /// * `edge_path` -- the `BezPath` for the edge body.
    /// * `arrow_head_path` -- the `BezPath` for the edge's arrow head.
    pub(super) fn calculate(edge_path: &BezPath, arrow_head_path: &BezPath) -> BezPath {
        let combined = edge_path.into_iter().chain(arrow_head_path.into_iter());

        let style = Stroke::new(LOCUS_STROKE_WIDTH)
            .with_join(Join::Round)
            .with_caps(Cap::Round);

        let opts = StrokeOpts::default();

        stroke(combined, &style, &opts, LOCUS_TOLERANCE)
    }
}
