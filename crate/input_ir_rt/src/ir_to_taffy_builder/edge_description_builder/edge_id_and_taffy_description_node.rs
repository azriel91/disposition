use std::cmp::Ordering;

use disposition_ir_model::edge::EdgeId;
use disposition_taffy_model::MdNodeTaffyIds;
use taffy::NodeId;

/// An edge's description leaf or markdown sub-tree taffy nodes paired with
/// its typed edge ID.
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

    /// The taffy leaf node (simple path) or `md_content_node` container
    /// (markdown path) for this edge.
    pub description_taffy_node_id: NodeId,

    /// Populated at `DiagramLod::Normal` with the markdown sub-tree IDs.
    pub md_node_taffy_ids: Option<MdNodeTaffyIds>,

    /// `sibling_index_from.cmp(&sibling_index_to)` at the edge's LCA depth,
    /// carried through to
    /// [`disposition_taffy_model::EdgeDescriptionTaffyNodes`]
    /// for the routing waypoint calculation.
    pub sibling_index_from_cmp_to: Ordering,
}
