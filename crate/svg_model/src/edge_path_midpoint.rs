use serde::{Deserialize, Serialize};

/// Mean anchor point of an edge's zero-offset (pass-1) path, in absolute
/// SVG coordinates.
///
/// Retained as a diagnostic value: it is the centroid of the path's
/// anchor points (MoveTo, LineTo, and final CurveTo / QuadTo points),
/// used while sorting face contacts during edge routing.
///
/// # Example values
///
/// ```rust,ignore
/// EdgePathMidpoint { x: 150.0, y: 80.0 }
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct EdgePathMidpoint {
    /// Mean x coordinate of the path's anchor points.
    pub x: f64,
    /// Mean y coordinate of the path's anchor points.
    pub y: f64,
}
