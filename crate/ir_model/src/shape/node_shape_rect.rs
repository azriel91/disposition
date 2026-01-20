use serde::{Deserialize, Serialize};

/// Corner radius configuration for a rectangular node shape.
///
/// This struct defines the radius for each corner of a rectangular node,
/// allowing for different corner radii. A radius of 0.0 means a sharp corner.
///
/// # Example
///
/// ```yaml
/// node_shapes:
///   t_aws:
///     rect:
///       top_left: 4.0
///       top_right: 4.0
///       bottom_left: 4.0
///       bottom_right: 4.0
///   # Different radii for each corner
///   t_localhost:
///     rect:
///       top_left: 8.0
///       top_right: 4.0
///       bottom_left: 4.0
///       bottom_right: 8.0
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct NodeShapeRect {
    /// The radius for the top-left corner of the rectangle.
    ///
    /// A value of 0.0 means a sharp corner.
    #[serde(default)]
    pub top_left: f32,

    /// The radius for the top-right corner of the rectangle.
    ///
    /// A value of 0.0 means a sharp corner.
    #[serde(default)]
    pub top_right: f32,

    /// The radius for the bottom-left corner of the rectangle.
    ///
    /// A value of 0.0 means a sharp corner.
    #[serde(default)]
    pub bottom_left: f32,

    /// The radius for the bottom-right corner of the rectangle.
    ///
    /// A value of 0.0 means a sharp corner.
    #[serde(default)]
    pub bottom_right: f32,
}

impl NodeShapeRect {
    /// Creates a new `NodeShapeRect` with all corners set to 0.0 (sharp
    /// corners).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_ir_model::shape::NodeShapeRect;
    ///
    /// let rect = NodeShapeRect::new();
    ///
    /// assert_eq!(rect.top_left, 0.0);
    /// assert_eq!(rect.top_right, 0.0);
    /// assert_eq!(rect.bottom_left, 0.0);
    /// assert_eq!(rect.bottom_right, 0.0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `NodeShapeRect` with the same radius for all corners.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_ir_model::shape::NodeShapeRect;
    ///
    /// let rect = NodeShapeRect::with_uniform_radius(4.0);
    ///
    /// assert_eq!(rect.top_left, 4.0);
    /// assert_eq!(rect.top_right, 4.0);
    /// assert_eq!(rect.bottom_left, 4.0);
    /// assert_eq!(rect.bottom_right, 4.0);
    /// ```
    pub fn with_uniform_radius(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        }
    }

    /// Returns the top-left corner radius.
    pub fn top_left(&self) -> f32 {
        self.top_left
    }

    /// Returns the top-right corner radius.
    pub fn top_right(&self) -> f32 {
        self.top_right
    }

    /// Returns the bottom-left corner radius.
    pub fn bottom_left(&self) -> f32 {
        self.bottom_left
    }

    /// Returns the bottom-right corner radius.
    pub fn bottom_right(&self) -> f32 {
        self.bottom_right
    }

    /// Returns true if all corner radii are zero (sharp corners).
    pub fn is_sharp(&self) -> bool {
        self.top_left == 0.0
            && self.top_right == 0.0
            && self.bottom_left == 0.0
            && self.bottom_right == 0.0
    }

    /// Returns true if all corner radii are equal.
    pub fn is_uniform(&self) -> bool {
        self.top_left == self.top_right
            && self.top_right == self.bottom_left
            && self.bottom_left == self.bottom_right
    }
}
