/// The taffy leaf node IDs for the two label slots of one edge.
///
/// Each edge that has a face assignment (i.e. is not a contained or self-loop
/// edge) gets one label slot on the `from` node and one on the `to` node.
/// Both slots are created as leaf nodes inside the envelope of their
/// respective endpoint nodes.
///
/// # Examples
///
/// ```text
/// EdgeLabelTaffyNodeIds {
///     from_label_taffy_node_id: Some(NodeId(1)),
///     to_label_taffy_node_id: Some(NodeId(2)),
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
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
}
