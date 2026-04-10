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
}

impl EdgeCurvature {
    /// Returns `true` if the edge curvature is the default (curved).
    pub fn is_default(&self) -> bool {
        matches!(self, EdgeCurvature::Curved)
    }
}

impl FromStr for EdgeCurvature {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "curved" => Ok(EdgeCurvature::Curved),
            "orthogonal" => Ok(EdgeCurvature::Orthogonal),
            _ => Err(()),
        }
    }
}

impl Display for EdgeCurvature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EdgeCurvature::Curved => write!(f, "curved"),
            EdgeCurvature::Orthogonal => write!(f, "orthogonal"),
        }
    }
}
