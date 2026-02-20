/// `taffy` node IDs related to an IR diagram node.
///
/// The `wrapper_node_id` is the node ID of the main taffy node, that represents
/// the IR diagram node. This is the same as the `text_node_id` if there are no
/// children.
///
/// The `text_node_id` is the node ID of the text node, which contains the text
/// content of the IR diagram node.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NodeToTaffyNodeIds {
    /// This node is a leaf node, so there is only one taffy node which contains
    /// its text.
    Leaf { text_node_id: taffy::NodeId },
    /// This node is a leaf node with a circle shape.
    ///
    /// The `wrapper_node_id` is a flex-row container that holds the circle
    /// and text nodes side by side.
    LeafWithCircle {
        wrapper_node_id: taffy::NodeId,
        circle_node_id: taffy::NodeId,
        text_node_id: taffy::NodeId,
    },
    /// This node has children, so there is one wrapper node, which holds a text
    /// node and a container node for its children.
    ///
    /// Currently the container taffy node ID isn't stored.
    Wrapper {
        wrapper_node_id: taffy::NodeId,
        text_node_id: taffy::NodeId,
    },
    /// This node has children and a circle shape.
    ///
    /// The `wrapper_node_id` holds a `label_wrapper_node_id` (flex-row
    /// containing the circle and text) and a child container.
    WrapperCircle {
        wrapper_node_id: taffy::NodeId,
        label_wrapper_node_id: taffy::NodeId,
        circle_node_id: taffy::NodeId,
        text_node_id: taffy::NodeId,
    },
}

impl NodeToTaffyNodeIds {
    /// Returns the wrapper taffy node ID, which is the same as the wrapper node
    /// ID if the node is a wrapper, or the text node ID if the node is a
    /// leaf.
    pub fn wrapper_taffy_node_id(self) -> taffy::NodeId {
        match self {
            NodeToTaffyNodeIds::Leaf { text_node_id } => text_node_id,
            NodeToTaffyNodeIds::LeafWithCircle {
                wrapper_node_id, ..
            } => wrapper_node_id,
            NodeToTaffyNodeIds::Wrapper {
                wrapper_node_id,
                text_node_id: _,
            } => wrapper_node_id,
            NodeToTaffyNodeIds::WrapperCircle {
                wrapper_node_id, ..
            } => wrapper_node_id,
        }
    }

    /// Returns the text node ID, which is the same as the outer node ID if the
    /// node is a leaf, or the text node ID if the node is a wrapper.
    pub fn text_taffy_node_id(&self) -> taffy::NodeId {
        match self {
            NodeToTaffyNodeIds::Leaf { text_node_id } => *text_node_id,
            NodeToTaffyNodeIds::LeafWithCircle { text_node_id, .. } => *text_node_id,
            NodeToTaffyNodeIds::Wrapper { text_node_id, .. } => *text_node_id,
            NodeToTaffyNodeIds::WrapperCircle { text_node_id, .. } => *text_node_id,
        }
    }

    /// Returns the circle taffy node ID if this node has a circle shape,
    /// or `None` otherwise.
    pub fn circle_taffy_node_id(&self) -> Option<taffy::NodeId> {
        match self {
            NodeToTaffyNodeIds::Leaf { .. } => None,
            NodeToTaffyNodeIds::LeafWithCircle { circle_node_id, .. } => Some(*circle_node_id),
            NodeToTaffyNodeIds::Wrapper { .. } => None,
            NodeToTaffyNodeIds::WrapperCircle { circle_node_id, .. } => Some(*circle_node_id),
        }
    }
}
