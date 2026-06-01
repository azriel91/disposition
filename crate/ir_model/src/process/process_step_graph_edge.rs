use serde::{Deserialize, Serialize};

use crate::{node::NodeId, process::ProcessStepLane};

/// A connector edge between two process steps in the git-graph layout.
///
/// The connector departs the `from` step's circle, travels vertically in its
/// `lane`, then enters the `to` step's circle. The `lane` is the travel lane
/// the connector keeps between the two endpoints (often the same lane as the
/// `from` step, but a new lane for additional outgoing branches).
///
/// # Example
///
/// ```yaml
/// from: proc_app_dev_step_repository_clone
/// to: proc_app_dev_step_project_build
/// lane: 0
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProcessStepGraphEdge<'id> {
    /// The source process step node ID where this connector originates.
    pub from: NodeId<'id>,
    /// The target process step node ID where this connector points to.
    pub to: NodeId<'id>,
    /// The lane the connector travels in between its endpoints.
    pub lane: ProcessStepLane,
}

impl<'id> ProcessStepGraphEdge<'id> {
    /// Creates a new `ProcessStepGraphEdge`.
    pub fn new(from: NodeId<'id>, to: NodeId<'id>, lane: ProcessStepLane) -> Self {
        Self { from, to, lane }
    }

    /// Converts this `ProcessStepGraphEdge` into one with a `'static` lifetime.
    pub fn into_static(self) -> ProcessStepGraphEdge<'static> {
        ProcessStepGraphEdge {
            from: self.from.into_static(),
            to: self.to.into_static(),
            lane: self.lane,
        }
    }
}
