use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// A face/side of a rectangular diagram node.
///
/// Used to identify which side of a node an edge exits or enters,
/// for routing edge paths and placing edge label slots.
///
/// # Examples
///
/// Valid values: `Top`, `Bottom`, `Left`, `Right`
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeFace {
    /// The top edge of the node rectangle.
    Top,
    /// The bottom edge of the node rectangle.
    Bottom,
    /// The left edge of the node rectangle.
    Left,
    /// The right edge of the node rectangle.
    Right,
}

impl Display for NodeFace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeFace::Top => "top",
            NodeFace::Bottom => "bottom",
            NodeFace::Left => "left",
            NodeFace::Right => "right",
        }
        .fmt(f)
    }
}
