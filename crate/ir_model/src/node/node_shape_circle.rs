use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Circle shape configuration for a node.
///
/// This struct defines the radius of a circular node shape.
///
/// # Example
///
/// ```yaml
/// node_shapes:
///   t_status_indicator:
///     circle:
///       radius: 4.0
///   t_large_indicator:
///     circle:
///       radius: 12.0
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct NodeShapeCircle {
    /// The radius of the circle.
    ///
    /// Defaults to `1.0`.
    #[serde(default = "radius_default")]
    pub radius: f32,
}

impl NodeShapeCircle {
    /// Creates a new `NodeShapeCircle` with radius set to 0.0.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_ir_model::node::NodeShapeCircle;
    ///
    /// let circle = NodeShapeCircle::new();
    ///
    /// assert_eq!(circle.radius, 1.0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `NodeShapeCircle` with the given radius.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_ir_model::node::NodeShapeCircle;
    ///
    /// let circle = NodeShapeCircle::with_radius(6.0);
    ///
    /// assert_eq!(circle.radius, 6.0);
    /// ```
    pub fn with_radius(radius: f32) -> Self {
        Self { radius }
    }

    /// Returns the radius of the circle.
    pub fn radius(&self) -> f32 {
        self.radius
    }
}

impl Default for NodeShapeCircle {
    fn default() -> Self {
        Self {
            radius: radius_default(),
        }
    }
}

impl Display for NodeShapeCircle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Circle({})", self.radius)
    }
}

fn radius_default() -> f32 {
    1.0
}
