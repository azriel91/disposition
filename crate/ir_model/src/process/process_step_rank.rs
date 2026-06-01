use std::fmt;

use serde::{Deserialize, Serialize};

/// The rank of a process step, used for layout ordering within a process.
///
/// Process steps with lower rank values are positioned earlier (left/top) and
/// process steps with higher rank values are positioned later (right/bottom)
/// along the flex direction axis.
///
/// Rank is determined by process step dependencies -- a step that depends on
/// another step (the `to` node of a process step edge) receives a higher rank
/// than the step it depends on.
///
/// # Examples
///
/// Valid values: `0`, `1`, `2`, `3`
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(
    Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize,
)]
pub struct ProcessStepRank(u32);

impl ProcessStepRank {
    /// Creates a new `ProcessStepRank` with the given value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_ir_model::process::ProcessStepRank;
    ///
    /// let rank = ProcessStepRank::new(2);
    /// assert_eq!(rank.value(), 2);
    /// ```
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the rank value.
    pub fn value(self) -> u32 {
        self.0
    }
}

impl fmt::Display for ProcessStepRank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for ProcessStepRank {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
