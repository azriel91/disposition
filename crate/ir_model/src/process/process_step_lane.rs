use std::fmt;

use serde::{Deserialize, Serialize};

/// The lane (cross-axis column) of a process step in the git-graph layout.
///
/// Process steps are laid out like `git log --graph`: each step occupies a row
/// (ordered by [`ProcessStepRank`]) and a lane. Lane `0` is the leftmost lane.
/// A step is shifted to a higher lane when a connector edge needs to bypass its
/// row, so the bypassing edge keeps a straight vertical line in a lower lane.
///
/// [`ProcessStepRank`]: crate::process::ProcessStepRank
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
pub struct ProcessStepLane(u32);

impl ProcessStepLane {
    /// Creates a new `ProcessStepLane` with the given value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_ir_model::process::ProcessStepLane;
    ///
    /// let lane = ProcessStepLane::new(2);
    /// assert_eq!(lane.value(), 2);
    /// ```
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the lane value.
    pub fn value(self) -> u32 {
        self.0
    }
}

impl fmt::Display for ProcessStepLane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for ProcessStepLane {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
