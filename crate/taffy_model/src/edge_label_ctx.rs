use disposition_ir_model::{
    edge::EdgeId,
    node::{NodeFace, NodeId},
};

/// Context data stored with edge label leaf nodes in the `TaffyTree`.
///
/// Edge label leaf nodes are inserted for each edge that attaches to a
/// diagram node face. They measure to the size of the edge description text,
/// and their layout positions are used to emit SVG edge label elements.
///
/// # Examples
///
/// ```text
/// TaffyNodeCtx::EdgeLabel(EdgeLabelCtx {
///     edge_id: "edge_t_a__t_b__0",
///     node_id: "t_a",
///     face: NodeFace::Right,
/// })
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct EdgeLabelCtx {
    /// The edge ID this label belongs to.
    pub edge_id: EdgeId<'static>,
    /// The endpoint node this label slot is attached to.
    pub node_id: NodeId<'static>,
    /// The face of the endpoint node that this label is placed on.
    pub face: NodeFace,
}
