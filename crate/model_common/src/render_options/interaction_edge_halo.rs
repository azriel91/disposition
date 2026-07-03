use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

/// Controls whether a semi-transparent halo is rendered behind interaction
/// edges.
///
/// Interaction edges are animated, so a wider, translucent halo sharing the
/// edge's path geometry makes it easier to see where the animated edge
/// travels.
///
/// # Examples
///
/// ```rust
/// use disposition_model_common::InteractionEdgeHalo;
///
/// let enabled = InteractionEdgeHalo::Enabled;
/// let disabled = InteractionEdgeHalo::Disabled;
/// assert_eq!(InteractionEdgeHalo::default(), enabled);
/// assert_ne!(enabled, disabled);
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InteractionEdgeHalo {
    /// No halo is rendered behind interaction edges.
    Disabled,
    /// A semi-transparent halo path is rendered behind each interaction edge,
    /// sharing its path geometry, to make animated edges easier to follow.
    #[default]
    Enabled,
}

impl InteractionEdgeHalo {
    /// Returns `true` if this is the default (`Enabled`).
    pub fn is_default(&self) -> bool {
        matches!(self, InteractionEdgeHalo::Enabled)
    }

    /// Returns `true` if the halo should be rendered.
    pub fn is_enabled(&self) -> bool {
        matches!(self, InteractionEdgeHalo::Enabled)
    }
}

impl FromStr for InteractionEdgeHalo {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "disabled" => Ok(InteractionEdgeHalo::Disabled),
            "enabled" => Ok(InteractionEdgeHalo::Enabled),
            _ => Err(()),
        }
    }
}

impl Display for InteractionEdgeHalo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InteractionEdgeHalo::Disabled => write!(f, "disabled"),
            InteractionEdgeHalo::Enabled => write!(f, "enabled"),
        }
    }
}
