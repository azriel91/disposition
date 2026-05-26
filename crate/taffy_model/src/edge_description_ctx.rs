use disposition_ir_model::edge::EdgeId;

/// Context data stored with edge description leaf nodes in the `TaffyTree`.
///
/// Edge description leaf nodes are placed inside an
/// `edge_description_container` taffy node, interleaved between rank
/// containers. They measure to the size of the edge description text, and
/// their layout positions are used to emit SVG edge description elements.
///
/// # Examples
///
/// ```text
/// TaffyNodeCtx::EdgeDescription(EdgeDescriptionCtx {
///     edge_id: "edge_t_a__t_b__0",
/// })
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct EdgeDescriptionCtx {
    /// The edge ID whose description this leaf node represents.
    pub edge_id: EdgeId<'static>,
}
