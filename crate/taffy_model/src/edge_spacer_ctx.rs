use disposition_ir_model::edge::EdgeId;

/// Context data stored with edge spacer nodes in the taffy tree.
///
/// Edge spacer nodes are inserted at intermediate rank levels for
/// edges that cross multiple ranks. They participate in the flex
/// layout so that the edge path can be routed through their
/// positions, reducing the chance of edges being drawn over other
/// nodes.
#[derive(Clone, Debug, PartialEq)]
pub struct EdgeSpacerCtx {
    /// The edge ID this spacer is for.
    pub edge_id: EdgeId<'static>,
}
