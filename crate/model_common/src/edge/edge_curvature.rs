use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

/// Controls how edge paths are drawn between nodes.
///
/// # Examples
///
/// ```rust
/// use disposition_model_common::edge::EdgeCurvature;
///
/// let curved = EdgeCurvature::Curved;
/// let ortho = EdgeCurvature::Orthogonal;
/// let direct_straight = EdgeCurvature::DirectStraight;
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeCurvature {
    /// Edges use smooth bezier curves between nodes and spacers.
    Curved,
    /// Edges use orthogonal (90-degree) lines between nodes and spacers.
    ///
    /// Corners where the path changes from horizontal to vertical (or
    /// vice versa) are rounded with a small arc.
    #[default]
    Orthogonal,
    /// Edges are straight lines drawn directly from the `from` node to the `to`
    /// node, bypassing edge spacers.
    ///
    /// Edge spacers for these edges collapse to zero size so they reserve no
    /// layout space.
    DirectStraight,
    /// Edges are smooth bezier curves drawn directly from the `from` node to the
    /// `to` node, bypassing edge spacers.
    ///
    /// Edge spacers for these edges collapse to zero size so they reserve no
    /// layout space.
    DirectCurved,
}

impl EdgeCurvature {
    /// Returns `true` if the edge curvature is the default (orthogonal).
    pub fn is_default(&self) -> bool {
        matches!(self, EdgeCurvature::Orthogonal)
    }

    /// Returns `true` if edges are drawn directly between nodes, bypassing edge
    /// spacers.
    ///
    /// This is the case for [`EdgeCurvature::DirectStraight`] and
    /// [`EdgeCurvature::DirectCurved`].
    pub fn is_direct(&self) -> bool {
        matches!(
            self,
            EdgeCurvature::DirectStraight | EdgeCurvature::DirectCurved
        )
    }
}

impl FromStr for EdgeCurvature {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "curved" => Ok(EdgeCurvature::Curved),
            "orthogonal" => Ok(EdgeCurvature::Orthogonal),
            "direct_straight" => Ok(EdgeCurvature::DirectStraight),
            "direct_curved" => Ok(EdgeCurvature::DirectCurved),
            _ => Err(()),
        }
    }
}

impl Display for EdgeCurvature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeCurvature::Curved => write!(f, "curved"),
            EdgeCurvature::Orthogonal => write!(f, "orthogonal"),
            EdgeCurvature::DirectStraight => write!(f, "direct_straight"),
            EdgeCurvature::DirectCurved => write!(f, "direct_curved"),
        }
    }
}
