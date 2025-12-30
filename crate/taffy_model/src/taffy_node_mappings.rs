use taffy::TaffyTree;

use crate::NodeContext;

#[derive(Clone, Debug)]
pub struct TaffyNodeMappings {
    pub taffy_tree: TaffyTree<NodeContext>,
    pub root: taffy::NodeId,
}
