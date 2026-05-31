use disposition_ir_model::edge::EdgeId;
use disposition_model_common::Map;

use crate::EdgeLabelTaffyNodeIds;

/// Map from each edge ID to its two edge label taffy leaf node IDs.
pub type EdgeIdToEdgeLabelTaffyNodeIds<'id> = Map<EdgeId<'id>, EdgeLabelTaffyNodeIds>;
