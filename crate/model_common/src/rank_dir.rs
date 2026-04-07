use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

/// Direction that edges are laid out in.
///
/// # Variants
///
/// * `left_to_right`: edges connect nodes from left to right.
/// * `right_to_left`: edges connect nodes from right to left.
/// * `top_to_bottom`: edges connect nodes from top to bottom.
/// * `bottom_to_top`: edges connect nodes from bottom to top.
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RankDir {
    /// Connect nodes from left to right.
    LeftToRight,
    /// Connect nodes from right to left.
    RightToLeft,
    /// Connect nodes from top to bottom.
    #[default]
    TopToBottom,
    /// Connect nodes from bottom to top.
    BottomToTop,
}

impl RankDir {
    /// Returns `true` if the rank direction is the default (`TopToBottom`).
    pub fn is_default(&self) -> bool {
        matches!(self, RankDir::TopToBottom)
    }
}

impl FromStr for RankDir {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "left_to_right" => Ok(RankDir::LeftToRight),
            "right_to_left" => Ok(RankDir::RightToLeft),
            "top_to_bottom" => Ok(RankDir::TopToBottom),
            "bottom_to_top" => Ok(RankDir::BottomToTop),
            _ => Err(()),
        }
    }
}

impl Display for RankDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RankDir::LeftToRight => write!(f, "left_to_right"),
            RankDir::RightToLeft => write!(f, "right_to_left"),
            RankDir::TopToBottom => write!(f, "top_to_bottom"),
            RankDir::BottomToTop => write!(f, "bottom_to_top"),
        }
    }
}
