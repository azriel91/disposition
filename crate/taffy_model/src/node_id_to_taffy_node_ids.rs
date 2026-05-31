use disposition_ir_model::node::NodeId;
use disposition_model_common::Map;

use crate::NodeToTaffyNodeIds;

/// Map of each IR diagram node to its related `taffy` node IDs.
pub type NodeIdToTaffyNodeIds<'id> = Map<NodeId<'id>, NodeToTaffyNodeIds>;
