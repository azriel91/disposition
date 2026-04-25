use kurbo::{stroke, BezPath, Cap, Join, Stroke, StrokeOpts};
use linesweeper::{BinaryOp, FillRule};

/// Width of the stroke expansion used to compute the edge locus, in pixels.
const LOCUS_STROKE_WIDTH: f64 = 10.0;

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
        let style = Stroke::new(LOCUS_STROKE_WIDTH)
            .with_join(Join::Round)
            .with_caps(Cap::Round);

        let opts = StrokeOpts::default();
        let edge_locus = stroke(edge_path, &style, &opts, LOCUS_TOLERANCE);
        let arrow_head_locus = stroke(arrow_head_path, &style, &opts, LOCUS_TOLERANCE);

        let contours = linesweeper::binary_op(
            &edge_locus,
            &arrow_head_locus,
            FillRule::NonZero,
            BinaryOp::Union,
        )
        .unwrap_or_else(|e| panic!("Failed to compute union of locus paths: {e:?}"));

        // We expect only one contour, since the arrow head should overlap with the
        // edge path body.
        contours
            .contours()
            .next()
            .map(|contour| contour.path.clone())
            .unwrap_or(edge_locus)
    }
}
