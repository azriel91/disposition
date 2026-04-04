use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

/// Direction that edges are laid out in.
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RankDir {
    /// Connect nodes horizontally.
    Horizontal,
    /// Connect nodes vertically.
    #[default]
    Vertical,
}

impl RankDir {
    /// Returns `true` if the rank direction is the default (vertical).
    pub fn is_default(&self) -> bool {
        matches!(self, RankDir::Vertical)
    }

    /// Returns `true` if the rank direction is horizontal.
    pub fn is_horizontal(&self) -> bool {
        matches!(self, RankDir::Horizontal)
    }

    /// Returns `true` if the rank direction is vertical.
    pub fn is_vertical(&self) -> bool {
        matches!(self, RankDir::Vertical)
    }
}

impl FromStr for RankDir {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "horizontal" => Ok(RankDir::Horizontal),
            "vertical" => Ok(RankDir::Vertical),
            _ => Err(()),
        }
    }
}

impl Display for RankDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RankDir::Horizontal => write!(f, "horizontal"),
            RankDir::Vertical => write!(f, "vertical"),
        }
    }
}
