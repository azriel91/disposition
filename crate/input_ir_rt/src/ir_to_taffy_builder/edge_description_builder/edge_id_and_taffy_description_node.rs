use disposition_ir_model::edge::EdgeId;
use taffy::NodeId;

/// An edge's description leaf taffy node paired with its typed edge ID.
///
/// Used as the value in the per-position sorted map inside
/// [`super::EdgeDescriptionBuilder::build`] so that both the `EdgeId` (needed
/// to record the final [`disposition_taffy_model::EdgeDescriptionTaffyNodes`])
/// and the taffy node ID (needed to add the leaf as a child of the shared
/// container) remain associated after sorting.
pub struct EdgeIdAndTaffyDescriptionNode {
    /// The typed edge ID for this description.
    ///
    /// Example valid values: `EdgeId::from("edge_dep_alice_charlie__0")`.
    pub edge_id: EdgeId<'static>,

    /// The taffy leaf node that holds the
    /// [`disposition_taffy_model::TaffyNodeCtx::EdgeDescription`] context for
    /// this edge.
    pub description_taffy_node_id: NodeId,
}
