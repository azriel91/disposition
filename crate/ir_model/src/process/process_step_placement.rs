use serde::{Deserialize, Serialize};

use crate::process::ProcessStepLane;

/// The position of a process step in the git-graph layout.
///
/// Each step occupies a `row` (its index along the main axis, ordered by
/// process step rank then declaration order) and a `lane` (its column along the
/// cross axis).
///
/// # Example
///
/// ```yaml
/// row: 1
/// lane: 0
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProcessStepPlacement {
    /// Row index of the step (position along the main axis).
    pub row: u32,
    /// Lane (cross-axis column) the step's circle sits in.
    pub lane: ProcessStepLane,
}

impl ProcessStepPlacement {
    /// Creates a new `ProcessStepPlacement`.
    pub fn new(row: u32, lane: ProcessStepLane) -> Self {
        Self { row, lane }
    }
}
