use disposition_ir_model::node::{NodeId, NodeInbuilt};
use disposition_model_common::Map;
use taffy::TaffyTree;

use crate::{EntityHighlightedSpans, NodeContext, NodeToTaffyNodeIds};

/// The taffy tree and mappings from each IR node ID to its `taffy` node ID.
#[derive(Clone, Debug)]
pub struct TaffyNodeMappings {
    /// The taffy tree that contains the layout information for each node.
    pub taffy_tree: TaffyTree<NodeContext>,
    /// Map of each inbuilt node (root, thing container, etc.) to its `taffy`
    /// node ID.
    pub node_inbuilt_to_taffy: Map<NodeInbuilt, taffy::NodeId>,
    /// Map of each IR diagram node to related `taffy` node IDs.
    pub node_id_to_taffy: Map<NodeId<'static>, NodeToTaffyNodeIds>,
    /// Map of each `taffy` node ID to its corresponding IR node ID.
    pub taffy_id_to_node: Map<taffy::NodeId, NodeId<'static>>,
    /// Syntax highlighted spans of entity descriptions.
    ///
    /// Currently this does not contain any styling information, because diagram
    /// generation increases from 20 ms to 1000 ms (debug mode). This was
    /// removed in `a331529`.
    pub entity_highlighted_spans: EntityHighlightedSpans,
}

impl PartialEq for TaffyNodeMappings {
    fn eq(&self, other: &Self) -> bool {
        let self_root = self.node_inbuilt_to_taffy.get(&NodeInbuilt::Root).copied();
        let other_root = other.node_inbuilt_to_taffy.get(&NodeInbuilt::Root).copied();
        self_root == other_root
            && self_root
                .zip(other_root)
                .map(|(self_root, other_root)| {
                    self.taffy_tree.children(self_root).ok()
                        == other.taffy_tree.children(other_root).ok()
                })
                .unwrap_or_default()
            && self.node_inbuilt_to_taffy == other.node_inbuilt_to_taffy
            && self.node_id_to_taffy == other.node_id_to_taffy
            && self.taffy_id_to_node == other.taffy_id_to_node
            && self.entity_highlighted_spans == other.entity_highlighted_spans
    }
}
