use disposition_ir_model::edge::EdgeId;
use disposition_model_common::Map;

use crate::EdgeSpacerTaffyNodes;

/// Map of each edge to its spacer taffy node IDs at intermediate ranks.
pub type EdgeIdToEdgeSpacerTaffyNodes<'id> = Map<EdgeId<'id>, EdgeSpacerTaffyNodes>;
