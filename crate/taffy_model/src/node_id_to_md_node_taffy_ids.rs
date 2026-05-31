use disposition_ir_model::node::NodeId;
use disposition_model_common::Map;

use crate::MdNodeTaffyIds;

/// Per-token taffy node IDs for diagram nodes that use the markdown content
/// path, keyed by diagram `NodeId`.
pub type NodeIdToMdNodeTaffyIds<'id> = Map<NodeId<'id>, MdNodeTaffyIds>;
