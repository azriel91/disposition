use disposition_model_common::{edge::EdgeId, Id};
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

    /// Returns the stable connector edge ID for this edge.
    ///
    /// The ID is `edge_ps_{from}__{to}`, used to look up the connector's
    /// tailwind classes and to identify its rendered SVG element.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_ir_model::{
    ///     node::NodeId,
    ///     process::{ProcessStepGraphEdge, ProcessStepLane},
    /// };
    /// use disposition_model_common::{id, Id};
    ///
    /// let edge = ProcessStepGraphEdge::new(
    ///     NodeId::from(id!("step_a")),
    ///     NodeId::from(id!("step_b")),
    ///     ProcessStepLane::new(0),
    /// );
    ///
    /// assert_eq!(edge.edge_id().as_str(), "edge_ps_step_a__step_b");
    /// ```
    pub fn edge_id(&self) -> EdgeId<'static> {
        let id_string = format!("edge_ps_{}__{}", self.from.as_str(), self.to.as_str());
        EdgeId::from(Id::try_from(id_string).expect("process step connector edge id is valid"))
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
