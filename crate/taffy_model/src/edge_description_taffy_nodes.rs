/// The taffy node IDs created for a single described edge at a single LCA
/// level.
///
/// Two nodes are created per edge description:
///
/// 1. A `container_taffy_node_id` -- a flex container interleaved between rank
///    containers at the LCA level.
/// 2. A `description_taffy_node_id` -- a leaf node inside the container whose
///    size is measured from the description text.
///
/// The container uses `TaffyNodeCtx::None` (like rank containers), while the
/// leaf uses `TaffyNodeCtx::EdgeDescription(EdgeDescriptionCtx)`.
///
/// # Examples
///
/// ```text
/// EdgeDescriptionTaffyNodes {
///     container_taffy_node_id: NodeId(10),
///     description_taffy_node_id: NodeId(11),
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EdgeDescriptionTaffyNodes {
    /// The flex container interleaved between rank containers.
    pub container_taffy_node_id: taffy::NodeId,
    /// The leaf node inside the container whose size is measured from the
    /// description text.
    pub description_taffy_node_id: taffy::NodeId,
}
