use disposition_input_model::process::ProcessId;
use disposition_model_common::Set;
use serde::{Deserialize, Serialize};

/// Processes to include in diagram generation.
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ProcessesIncluded {
    /// All processes should be included in diagram generation.
    ///
    /// This could mean one diagram with all processes, all one diagram per
    /// process (see `DiagramSplit::OnePerProcess`).
    All,
    /// Diagram should fit within 1536x1280.
    Filter {
        /// IDs of the processes to include.
        process_ids: Set<ProcessId>,
    },
}
