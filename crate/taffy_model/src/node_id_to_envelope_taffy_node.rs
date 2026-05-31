use disposition_ir_model::node::NodeId;
use disposition_model_common::Map;

/// Map from each diagram node ID to its envelope taffy node ID.
pub type NodeIdToEnvelopeTaffyNode<'id> = Map<NodeId<'id>, taffy::NodeId>;
