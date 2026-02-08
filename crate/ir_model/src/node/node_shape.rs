use serde::{Deserialize, Serialize};

use crate::node::NodeShapeRect;

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
///       radius_top_left: 4.0
///       radius_top_right: 4.0
///       radius_bottom_left: 4.0
///       radius_bottom_right: 4.0
///   t_localhost:
///     rect:
///       radius_top_left: 8.0
///       radius_top_right: 8.0
///       radius_bottom_left: 8.0
///       radius_bottom_right: 8.0
///   # Leaf node with no corner radius
///   t_aws_iam_ecs_policy:
///     rect:
///       radius_top_left: 0.0
///       radius_top_right: 0.0
///       radius_bottom_left: 0.0
///       radius_bottom_right: 0.0
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
