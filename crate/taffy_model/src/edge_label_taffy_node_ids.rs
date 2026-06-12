use crate::MdNodeTaffyIds;

/// The taffy node IDs for the two label slots of one edge.
///
/// Each edge that has a face assignment (i.e. is not a contained or self-loop
/// edge) gets one label slot on the `from` node and one on the `to` node.
/// Both slots are created as nodes inside the envelope of their respective
/// endpoint nodes.
///
/// At [`DiagramLod::Normal`] each label slot wraps a markdown content sub-tree
/// (built via `MdNodeBuilder`), and its `MdNodeTaffyIds` are stored alongside
/// the slot node so the per-token spans can be computed after layout. At
/// [`DiagramLod::Simple`] each slot is a single placeholder leaf and its
/// `md_node_taffy_ids` is `None`.
///
/// [`DiagramLod::Normal`]: crate::DiagramLod::Normal
/// [`DiagramLod::Simple`]: crate::DiagramLod::Simple
///
/// # Examples
///
/// ```text
/// EdgeLabelTaffyNodeIds {
///     from_label_taffy_node_id: Some(NodeId(1)),
///     to_label_taffy_node_id: Some(NodeId(2)),
///     from_label_md_node_taffy_ids: None,
///     to_label_md_node_taffy_ids: None,
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct EdgeLabelTaffyNodeIds {
    /// Label slot on the `from` endpoint's envelope face.
    ///
    /// `None` for contained edges (one endpoint is an ancestor of the other)
    /// and self-loop edges.
    pub from_label_taffy_node_id: Option<taffy::NodeId>,
    /// Label slot on the `to` endpoint's envelope face.
    ///
    /// `None` for contained edges and self-loop edges.
    pub to_label_taffy_node_id: Option<taffy::NodeId>,
    /// Markdown sub-tree IDs for the `from` label slot.
    ///
    /// `Some` at [`DiagramLod::Normal`] when the `from` label text is
    /// non-empty; `None` otherwise.
    ///
    /// [`DiagramLod::Normal`]: crate::DiagramLod::Normal
    pub from_label_md_node_taffy_ids: Option<MdNodeTaffyIds>,
    /// Markdown sub-tree IDs for the `to` label slot.
    ///
    /// `Some` at [`DiagramLod::Normal`] when the `to` label text is
    /// non-empty; `None` otherwise.
    ///
    /// [`DiagramLod::Normal`]: crate::DiagramLod::Normal
    pub to_label_md_node_taffy_ids: Option<MdNodeTaffyIds>,
}
