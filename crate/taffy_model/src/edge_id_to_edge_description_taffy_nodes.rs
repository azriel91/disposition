use disposition_ir_model::edge::EdgeId;
use disposition_model_common::Map;

use crate::EdgeDescriptionTaffyNodes;

/// Map from each edge ID to its `edge_description_container` and leaf taffy
/// node IDs.
pub type EdgeIdToEdgeDescriptionTaffyNodes<'id> = Map<EdgeId<'id>, EdgeDescriptionTaffyNodes>;
