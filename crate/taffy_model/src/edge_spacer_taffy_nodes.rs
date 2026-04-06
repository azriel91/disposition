use disposition_ir_model::node::NodeRank;
use disposition_model_common::Map;

/// Holds spacer `taffy` node IDs for a single edge.
///
/// Two kinds of spacers are tracked:
///
/// 1. **Rank-based spacers** -- inserted at intermediate rank levels when an
///    edge crosses multiple ranks at the same nesting level. These are stored
///    in `rank_to_spacer_taffy_node_id`.
///
/// 2. **Cross-container spacers** -- inserted inside nested containers when an
///    edge enters a container from outside and needs to route alongside sibling
///    nodes to reach its target. These are stored in
///    `cross_container_spacer_taffy_node_ids`.
///
/// # Examples
///
/// For an edge from rank 0 to rank 3, rank-based spacer nodes might
/// exist at ranks 1 and 2:
///
/// ```text
/// EdgeSpacerTaffyNodes {
///     rank_to_spacer_taffy_node_id: { 1: NodeId(5), 2: NodeId(8) },
///     cross_container_spacer_taffy_node_ids: [],
/// }
/// ```
///
/// For an edge from node A to deeply nested node D (inside container
/// C), a cross-container spacer alongside C's sibling B:
///
/// ```text
/// EdgeSpacerTaffyNodes {
///     rank_to_spacer_taffy_node_id: {},
///     cross_container_spacer_taffy_node_ids: [NodeId(12)],
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct EdgeSpacerTaffyNodes {
    /// Map from each intermediate rank to the spacer taffy node ID at
    /// that rank.
    ///
    /// Used for edges that cross multiple ranks at the same hierarchy
    /// level.
    pub rank_to_spacer_taffy_node_id: Map<NodeRank, taffy::NodeId>,

    /// Spacer taffy node IDs inside nested containers for edges that
    /// cross container boundaries.
    ///
    /// These spacers are not keyed by rank because multiple
    /// cross-container spacers may share the same global rank value.
    /// Their absolute positions after layout are used to determine
    /// the correct ordering along the edge path.
    pub cross_container_spacer_taffy_node_ids: Vec<taffy::NodeId>,
}

impl EdgeSpacerTaffyNodes {
    /// Creates a new empty `EdgeSpacerTaffyNodes`.
    pub fn new() -> Self {
        Self {
            rank_to_spacer_taffy_node_id: Map::new(),
            cross_container_spacer_taffy_node_ids: Vec::new(),
        }
    }
}

impl Default for EdgeSpacerTaffyNodes {
    fn default() -> Self {
        Self::new()
    }
}
