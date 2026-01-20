use serde::{Deserialize, Serialize};

use crate::shape::NodeShapeRect;

/// The shape of a node in the diagram.
///
/// A node can be rendered as different shapes. Currently only `Rect` is
/// supported, but more shapes like `Ellipse` or `Pill` may be added in the
/// future.
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
///   t_localhost:
///     rect:
///       top_left: 8.0
///       top_right: 8.0
///       bottom_left: 8.0
///       bottom_right: 8.0
///   # Leaf node with no corner radius
///   t_aws_iam_ecs_policy:
///     rect:
///       top_left: 0.0
///       top_right: 0.0
///       bottom_left: 0.0
///       bottom_right: 0.0
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeShape {
    /// Rectangular shape with optional corner radii.
    Rect(NodeShapeRect),
}

impl Default for NodeShape {
    fn default() -> Self {
        NodeShape::Rect(NodeShapeRect::default())
    }
}

impl From<NodeShapeRect> for NodeShape {
    fn from(rect: NodeShapeRect) -> Self {
        NodeShape::Rect(rect)
    }
}
