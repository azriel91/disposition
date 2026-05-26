/// Taffy node IDs for one block-level markdown element and its leaf tokens.
///
/// # Examples
///
/// ```text
/// MdBlockTaffyIds {
///     block_row_node_id: NodeId(5),
///     token_node_ids: vec![NodeId(6), NodeId(7)],
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct MdBlockTaffyIds {
    /// The flex-row-wrap container node for this block.
    pub block_row_node_id: taffy::NodeId,
    /// Ordered leaf node IDs for each token or image in this block.
    ///
    /// Each ID corresponds to either a `TaffyNodeCtx::MdToken` leaf or a
    /// `TaffyNodeCtx::MdImage` leaf.
    pub token_node_ids: Vec<taffy::NodeId>,
}
