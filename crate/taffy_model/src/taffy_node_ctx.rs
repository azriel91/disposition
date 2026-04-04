use crate::{DiagramNodeCtx, EdgeSpacerCtx};

/// Context data stored with each node in the `TaffyTree`.
///
/// This distinguishes between diagram nodes (which represent actual
/// entities in the IR diagram) and edge spacer nodes (which are
/// inserted to guide edge paths between ranks).
///
/// # Examples
///
/// ```text
/// TaffyNodeCtx::DiagramNode(DiagramNodeCtx { entity_id: "app", entity_type: ThingDefault })
/// TaffyNodeCtx::EdgeSpacer(EdgeSpacerCtx {})
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum TaffyNodeCtx {
    /// A node representing an actual diagram entity (thing, process, tag,
    /// etc.).
    DiagramNode(DiagramNodeCtx),
    /// A spacer node inserted for edge routing between ranks.
    EdgeSpacer(EdgeSpacerCtx),
}
