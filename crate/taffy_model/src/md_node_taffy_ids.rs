use crate::MdBlockTaffyIds;

/// Taffy node IDs for a diagram node's complete markdown content area.
///
/// # Examples
///
/// ```text
/// MdNodeTaffyIds {
///     content_node_id: NodeId(4),
///     block_taffy_ids: vec![
///         MdBlockTaffyIds { block_col_node_id: NodeId(5), token_node_ids: vec![NodeId(6)], is_code_block: false },
///     ],
///     blockquote_node_ids: vec![],
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct MdNodeTaffyIds {
    /// The flex-column container holding all block rows.
    ///
    /// This is the node stored as `text_node_id` in `NodeToTaffyNodeIds`.
    pub content_node_id: taffy::NodeId,
    /// One entry per block-level element, in source order.
    pub block_taffy_ids: Vec<MdBlockTaffyIds>,
    /// Container node IDs for blockquote boxes, one per blockquote (including
    /// nested ones), in the order they are closed.
    ///
    /// `MdSpansComputer` sizes a bordered-box frame span to each of these.
    pub blockquote_node_ids: Vec<taffy::NodeId>,
}
