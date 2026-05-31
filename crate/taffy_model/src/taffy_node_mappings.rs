use disposition_ir_model::node::NodeInbuilt;
use taffy::TaffyTree;

use crate::{
    EdgeIdToEdgeDescriptionTaffyNodes, EdgeIdToEdgeLabelTaffyNodeIds, EdgeIdToEdgeSpacerTaffyNodes,
    EdgeIdToHighlightedSpans, EdgeIdToImageSpans, EntityHighlightedSpans,
    NodeIdToEnvelopeTaffyNode, NodeIdToImageSpans, NodeIdToMdNodeTaffyIds, NodeIdToTaffyNodeIds,
    NodeInbuiltToTaffyNode, TaffyNodeCtx, TaffyNodeToNodeId,
};

/// The taffy tree and mappings from each IR node ID to its `taffy` node ID.
#[derive(Clone, Debug)]
pub struct TaffyNodeMappings<'id> {
    /// The taffy tree that contains the layout information for each node.
    pub taffy_tree: TaffyTree<TaffyNodeCtx>,
    /// Map of each inbuilt node (root, thing container, etc.) to its `taffy`
    /// node ID.
    pub node_inbuilt_to_taffy: NodeInbuiltToTaffyNode,
    /// Map of each IR diagram node to related `taffy` node IDs.
    pub node_id_to_taffy: NodeIdToTaffyNodeIds<'id>,
    /// Map of each `taffy` node ID to its corresponding IR node ID.
    pub taffy_id_to_node: TaffyNodeToNodeId<'id>,
    /// Map of each edge to its spacer taffy node IDs at intermediate ranks.
    ///
    /// When an edge crosses multiple ranks, spacer nodes are inserted in
    /// the flex layout at each intermediate rank. The edge path is then
    /// routed through these spacer positions to avoid overlapping other
    /// nodes.
    pub edge_spacer_taffy_nodes: EdgeIdToEdgeSpacerTaffyNodes<'id>,
    /// Syntax highlighted spans of entity descriptions.
    ///
    /// Currently this does not contain any styling information, because diagram
    /// generation increases from 20 ms to 1000 ms (debug mode). This was
    /// removed in `a331529`.
    pub entity_highlighted_spans: EntityHighlightedSpans<'id>,
    /// Map from each edge ID to its two edge label taffy leaf node IDs.
    ///
    /// Populated during envelope node construction (Phase 2). Each entry
    /// holds the label leaf on the `from` endpoint's face and the label leaf
    /// on the `to` endpoint's face. Both may be `None` for contained or
    /// self-loop edges.
    pub edge_label_taffy_nodes: EdgeIdToEdgeLabelTaffyNodeIds<'id>,
    /// Map from each edge ID to its `edge_description_container` and leaf
    /// taffy node IDs.
    ///
    /// Populated during edge description container construction. Each entry
    /// holds the container taffy node ID (a flex container interleaved between
    /// rank containers) and the description taffy node ID (a leaf whose size
    /// is measured from the description text).
    pub edge_description_taffy_nodes: EdgeIdToEdgeDescriptionTaffyNodes<'id>,
    /// Highlighted spans computed for each edge description leaf node.
    ///
    /// Keyed by `EdgeId` (separate from `entity_highlighted_spans` to avoid
    /// key collisions between edge IDs and node IDs).
    /// Populated in Phase 3 after taffy layout completes.
    pub edge_description_highlighted_spans: EdgeIdToHighlightedSpans<'id>,
    /// Map from each diagram node ID to its envelope taffy node ID.
    ///
    /// The envelope wraps the existing `diagram_node_wrapper_node` and adds
    /// flex-row/column slots for edge label leaf nodes on each face.
    /// Populated during envelope node construction (Phase 2).
    ///
    /// Kept separate from `node_id_to_taffy` (which maps to
    /// `NodeToTaffyNodeIds`) to avoid churn in all existing code that reads
    /// `node_id_to_taffy`.
    pub node_id_to_envelope_taffy_node: NodeIdToEnvelopeTaffyNode<'id>,
    /// Per-token taffy node IDs for diagram nodes that use the markdown
    /// content path (`DiagramLod::Normal` with a description).
    ///
    /// Keyed by diagram `NodeId`. Absent for nodes that use the legacy
    /// single-leaf text path.
    pub md_node_taffy_ids: NodeIdToMdNodeTaffyIds<'id>,
    /// Inline image spans computed after taffy layout for markdown nodes.
    ///
    /// Keyed by diagram `NodeId`. Absent for nodes without inline images.
    pub entity_image_spans: NodeIdToImageSpans<'id>,
    /// Inline image spans for edge descriptions that used the markdown path.
    ///
    /// Keyed by `EdgeId`. Absent for edges using the legacy single-leaf path
    /// or edges without inline images.
    pub edge_description_image_spans: EdgeIdToImageSpans<'id>,
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
            && self.edge_label_taffy_nodes == other.edge_label_taffy_nodes
            && self.edge_description_taffy_nodes == other.edge_description_taffy_nodes
            && self.edge_description_highlighted_spans == other.edge_description_highlighted_spans
            && self.node_id_to_envelope_taffy_node == other.node_id_to_envelope_taffy_node
            && self.md_node_taffy_ids == other.md_node_taffy_ids
            && self.entity_image_spans == other.entity_image_spans
            && self.edge_description_image_spans == other.edge_description_image_spans
    }
}

fn taffy_nodes_eq(
    self_taffy_tree: &TaffyTree<TaffyNodeCtx>,
    self_root: Option<taffy::NodeId>,
    other_taffy_tree: &TaffyTree<TaffyNodeCtx>,
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
