use crate::{DiagramNodeCtx, EdgeLabelCtx, EdgeSpacerCtx};

/// Context data stored with each node in the `TaffyTree`.
///
/// This distinguishes between diagram nodes (which represent actual
/// entities in the IR diagram), edge spacer nodes (which are inserted to
/// guide edge paths between ranks), and edge label leaf nodes (which
/// measure and position edge description text).
///
/// # Examples
///
/// ```text
/// TaffyNodeCtx::DiagramNode(DiagramNodeCtx { entity_id: "app", entity_type: ThingDefault })
/// TaffyNodeCtx::EdgeSpacer(EdgeSpacerCtx {})
/// TaffyNodeCtx::EdgeLabel(EdgeLabelCtx { edge_id: "edge_t_a__t_b__0", node_id: "t_a", face: NodeFace::Right })
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum TaffyNodeCtx {
    /// A node representing an actual diagram entity (thing, process, tag,
    /// etc.).
    DiagramNode(DiagramNodeCtx),
    /// A spacer node inserted for edge routing between ranks.
    EdgeSpacer(EdgeSpacerCtx),
    /// A leaf node that measures and positions an edge description label on a
    /// specific face of a diagram node's envelope.
    EdgeLabel(EdgeLabelCtx),
}
