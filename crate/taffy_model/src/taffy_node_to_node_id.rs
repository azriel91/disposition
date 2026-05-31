use disposition_ir_model::node::NodeId;
use disposition_model_common::Map;

/// Map of each `taffy` node ID to its corresponding IR node ID.
pub type TaffyNodeToNodeId<'id> = Map<taffy::NodeId, NodeId<'id>>;
