use disposition_ir_model::node::NodeRank;
use disposition_model_common::Map;

/// Maps each rank level to a spacer `taffy` node ID for a single edge
/// that crosses multiple ranks.
///
/// When an edge connects nodes at different ranks, spacer nodes are
/// inserted at each intermediate rank to guide the edge path around
/// other nodes.
///
/// # Examples
///
/// For an edge from rank 0 to rank 3, spacer nodes might exist at
/// ranks 1 and 2:
///
/// ```text
/// EdgeSpacerTaffyNodes {
///     rank_to_spacer_taffy_node_id: { 1: NodeId(5), 2: NodeId(8) }
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct EdgeSpacerTaffyNodes {
    /// Map from each intermediate rank to the spacer taffy node ID at
    /// that rank.
    pub rank_to_spacer_taffy_node_id: Map<NodeRank, taffy::NodeId>,
}

impl EdgeSpacerTaffyNodes {
    /// Creates a new empty `EdgeSpacerTaffyNodes`.
    pub fn new() -> Self {
        Self {
            rank_to_spacer_taffy_node_id: Map::new(),
        }
    }
}

impl Default for EdgeSpacerTaffyNodes {
    fn default() -> Self {
        Self::new()
    }
}
