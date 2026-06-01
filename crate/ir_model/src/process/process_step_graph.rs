use disposition_model_common::Map;
use serde::{Deserialize, Serialize};

use crate::{
    node::NodeId,
    process::{ProcessStepGraphEdge, ProcessStepPlacement},
};

/// The git-graph layout for the steps of a single process.
///
/// Holds the number of lanes the graph spans, the row/lane placement of each
/// step's circle, and the connector edges (with their travel lanes). This is
/// consumed by the taffy builder to position the step circles, and by the
/// process step connector router to draw the connector paths.
///
/// # Example
///
/// ```yaml
/// lane_count: 1
/// step_placements:
///   proc_app_dev_step_repository_clone:
///     row: 0
///     lane: 0
///   proc_app_dev_step_project_build:
///     row: 1
///     lane: 0
/// edges:
///   - from: proc_app_dev_step_repository_clone
///     to: proc_app_dev_step_project_build
///     lane: 0
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProcessStepGraph<'id> {
    /// Number of lanes (cross-axis columns) the graph spans.
    pub lane_count: u32,

    /// Placement (row and lane) of each process step's circle.
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub step_placements: Map<NodeId<'id>, ProcessStepPlacement>,

    /// Connector edges between process steps and their travel lanes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<ProcessStepGraphEdge<'id>>,
}

impl<'id> ProcessStepGraph<'id> {
    /// Returns a new empty `ProcessStepGraph`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Converts this `ProcessStepGraph` into one with a `'static` lifetime.
    pub fn into_static(self) -> ProcessStepGraph<'static> {
        ProcessStepGraph {
            lane_count: self.lane_count,
            step_placements: self
                .step_placements
                .into_iter()
                .map(|(node_id, placement)| (node_id.into_static(), placement))
                .collect(),
            edges: self
                .edges
                .into_iter()
                .map(ProcessStepGraphEdge::into_static)
                .collect(),
        }
    }
}
