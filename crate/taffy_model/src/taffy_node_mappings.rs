use disposition_ir_model::node::{NodeId, NodeInbuilt};
use disposition_model_common::Map;
use taffy::TaffyTree;

use crate::{EntityHighlightedSpans, NodeContext};

/// The taffy tree and mappings from each IR node ID to its `taffy` node ID.
#[derive(Clone, Debug)]
pub struct TaffyNodeMappings {
    /// The taffy tree that contains the layout information for each node.
    pub taffy_tree: TaffyTree<NodeContext>,
    /// Map of each inbuilt node (root, thing container, etc.) to its `taffy`
    /// node ID.
    pub node_inbuilt_to_taffy: Map<NodeInbuilt, taffy::NodeId>,
    /// Map of each IR diagram node to its `taffy` node ID.
    pub node_id_to_taffy: Map<NodeId, taffy::NodeId>,
    /// `syntect` highlighted spans of entity descriptions.
    pub entity_highlighted_spans: EntityHighlightedSpans,
}
