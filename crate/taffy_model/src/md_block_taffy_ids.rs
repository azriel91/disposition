/// Taffy node IDs for one block-level markdown element and its leaf tokens.
///
/// # Examples
///
/// ```text
/// MdBlockTaffyIds {
///     block_col_node_id: NodeId(5),
///     token_node_ids: vec![NodeId(6), NodeId(7)],
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct MdBlockTaffyIds {
    /// The flex-column container node for this block.
    ///
    /// Contains one flex-row-wrap `line_row_node` per logical line, split at
    /// `LineBreak` token boundaries. For blocks with no `LineBreak` tokens this
    /// holds a single `line_row_node`.
    pub block_col_node_id: taffy::NodeId,
    /// Ordered leaf node IDs for each token or image in this block.
    ///
    /// Each ID corresponds to either a `TaffyNodeCtx::MdToken` leaf or a
    /// `TaffyNodeCtx::MdImage` leaf.
    pub token_node_ids: Vec<taffy::NodeId>,
}
