use serde::{Deserialize, Serialize};

/// Circle shape information for an SVG node.
///
/// This holds the SVG path `d` attribute representing the circle,
/// along with the circle's center coordinates and radius for use
/// in edge connection calculations.
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct SvgNodeInfoCircle {
    /// The SVG path `d` attribute representing the circle.
    pub path_d: String,
    /// The x coordinate of the circle's center (relative to the node's x).
    pub cx: f32,
    /// The y coordinate of the circle's center (relative to the node's y).
    pub cy: f32,
    /// The radius of the circle.
    pub radius: f32,
}

impl SvgNodeInfoCircle {
    /// Creates a new `SvgNodeInfoCircle`.
    pub fn new(path_d: String, cx: f32, cy: f32, radius: f32) -> Self {
        Self {
            path_d,
            cx,
            cy,
            radius,
        }
    }

    /// Builds the SVG path `d` attribute for a circle with the given center
    /// and radius.
    ///
    /// The circle is drawn as two arcs (semicircles) to form a complete
    /// circle using SVG path commands.
    pub fn build_path_d(cx: f32, cy: f32, radius: f32) -> String {
        // Draw circle as two arcs:
        // Move to the leftmost point, arc to the rightmost point,
        // then arc back to complete the circle.
        format!(
            "M {} {} A {radius} {radius} 0 1 1 {} {} A {radius} {radius} 0 1 1 {} {}",
            cx - radius,
            cy,
            cx + radius,
            cy,
            cx - radius,
            cy,
        )
    }
}
