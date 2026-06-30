use serde::{Deserialize, Serialize};

/// Axis-aligned bounding box of an edge's zero-offset (pass-1) path
/// anchor points, in absolute SVG coordinates.
///
/// Computed from the same anchor points as
/// [`EdgePathMidpoint`](crate::EdgePathMidpoint). Retained as a
/// diagnostic value used while sorting face contacts during edge
/// routing.
///
/// # Example values
///
/// ```rust,ignore
/// EdgePathBounds { x_min: 100.0, x_max: 200.0, y_min: 50.0, y_max: 130.0 }
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct EdgePathBounds {
    /// Minimum x coordinate among the path's anchor points.
    pub x_min: f64,
    /// Maximum x coordinate among the path's anchor points.
    pub x_max: f64,
    /// Minimum y coordinate among the path's anchor points.
    pub y_min: f64,
    /// Maximum y coordinate among the path's anchor points.
    pub y_max: f64,
}
