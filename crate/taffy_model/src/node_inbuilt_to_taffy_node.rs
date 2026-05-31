use disposition_ir_model::node::NodeInbuilt;
use disposition_model_common::Map;

/// Map of each inbuilt node (root, thing container, etc.) to its `taffy` node
/// ID.
pub type NodeInbuiltToTaffyNode = Map<NodeInbuilt, taffy::NodeId>;
