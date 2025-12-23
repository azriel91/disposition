use taffy::{Style, TaffyError, TaffyTree};

use crate::NodeContext;

#[derive(Clone, Debug)]
pub struct TaffyTreeAndRoot {
    pub taffy_tree: TaffyTree<NodeContext>,
    pub root: taffy::NodeId,
}
