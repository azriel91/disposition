use disposition_ir_model::{
    edge::EdgeId,
    node::{NodeId, NodeInbuilt},
};
use disposition_model_common::Map;
use taffy::TaffyTree;

use crate::{EdgeSpacerTaffyNodes, EntityHighlightedSpans, NodeContext, NodeToTaffyNodeIds};

/// The taffy tree and mappings from each IR node ID to its `taffy` node ID.
#[derive(Clone, Debug)]
pub struct TaffyNodeMappings<'id> {
    /// The taffy tree that contains the layout information for each node.
    pub taffy_tree: TaffyTree<NodeContext>,
    /// Map of each inbuilt node (root, thing container, etc.) to its `taffy`
    /// node ID.
    pub node_inbuilt_to_taffy: Map<NodeInbuilt, taffy::NodeId>,
    /// Map of each IR diagram node to related `taffy` node IDs.
    pub node_id_to_taffy: Map<NodeId<'id>, NodeToTaffyNodeIds>,
    /// Map of each `taffy` node ID to its corresponding IR node ID.
    pub taffy_id_to_node: Map<taffy::NodeId, NodeId<'id>>,
    /// Map of each edge to its spacer taffy node IDs at intermediate ranks.
    ///
    /// When an edge crosses multiple ranks, spacer nodes are inserted in
    /// the flex layout at each intermediate rank. The edge path is then
    /// routed through these spacer positions to avoid overlapping other
    /// nodes.
    pub edge_spacer_taffy_nodes: Map<EdgeId<'id>, EdgeSpacerTaffyNodes>,
    /// Syntax highlighted spans of entity descriptions.
    ///
    /// Currently this does not contain any styling information, because diagram
    /// generation increases from 20 ms to 1000 ms (debug mode). This was
    /// removed in `a331529`.
    pub entity_highlighted_spans: EntityHighlightedSpans<'id>,
}

impl<'id> PartialEq for TaffyNodeMappings<'id> {
    fn eq(&self, other: &Self) -> bool {
        let self_root = self.node_inbuilt_to_taffy.get(&NodeInbuilt::Root).copied();
        let other_root = other.node_inbuilt_to_taffy.get(&NodeInbuilt::Root).copied();

        let self_taffy_tree = &self.taffy_tree;
        let other_taffy_tree = &other.taffy_tree;
        taffy_nodes_eq(self_taffy_tree, self_root, other_taffy_tree, other_root)
            && self.node_inbuilt_to_taffy == other.node_inbuilt_to_taffy
            && self.node_id_to_taffy == other.node_id_to_taffy
            && self.taffy_id_to_node == other.taffy_id_to_node
            && self.edge_spacer_taffy_nodes == other.edge_spacer_taffy_nodes
            && self.entity_highlighted_spans == other.entity_highlighted_spans
    }
}

fn taffy_nodes_eq(
    self_taffy_tree: &TaffyTree<NodeContext>,
    self_root: Option<taffy::NodeId>,
    other_taffy_tree: &TaffyTree<NodeContext>,
    other_root: Option<taffy::NodeId>,
) -> bool {
    self_root == other_root
        && self_root
            .zip(other_root)
            .map(|(self_root, other_root)| {
                let self_children = self_taffy_tree.children(self_root).ok();
                let other_children = other_taffy_tree.children(other_root).ok();

                match (self_children, other_children) {
                    (None, None) => true,
                    (None, Some(_)) | (Some(_), None) => false,
                    (Some(self_children), Some(other_children)) => {
                        self_children.len() == other_children.len()
                            && self_children.iter().copied().zip(other_children).all(
                                |(self_child, other_child)| {
                                    self_taffy_tree.layout(self_child).ok()
                                        == other_taffy_tree.layout(other_child).ok()
                                        && taffy_nodes_eq(
                                            self_taffy_tree,
                                            Some(self_child),
                                            other_taffy_tree,
                                            Some(other_child),
                                        )
                                },
                            )
                    }
                }
            })
            .unwrap_or_default()
}
