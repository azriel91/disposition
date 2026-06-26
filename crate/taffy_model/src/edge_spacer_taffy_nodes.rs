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

    /// Spacer taffy node IDs inside `edge_description_container` nodes.
    ///
    /// These spacers are inserted to allow edges to be routed alongside the
    /// edge description container without being obscured by it.
    pub edge_desc_container_spacer_taffy_node_ids: Vec<taffy::NodeId>,

    /// Spacer taffy node IDs placed beside a described node's text content (its
    /// title + description), so a cross-container edge that enters the node to
    /// reach a nested child routes around the description text rather than
    /// across it.
    ///
    /// Unlike [`Self::cross_container_spacer_taffy_node_ids`], these are
    /// **excluded from the cross-container column snap** so the detour around
    /// the text stays local to the node's text band -- the edge approaches at
    /// its normal column and only bows out around the label, instead of the
    /// whole descent column being pulled onto the text's far side.
    pub text_content_spacer_taffy_node_ids: Vec<taffy::NodeId>,
}

impl EdgeSpacerTaffyNodes {
    /// Creates a new empty `EdgeSpacerTaffyNodes`.
    pub fn new() -> Self {
        Self {
            rank_to_spacer_taffy_node_id: Map::new(),
            cross_container_spacer_taffy_node_ids: Vec::new(),
            edge_desc_container_spacer_taffy_node_ids: Vec::new(),
            text_content_spacer_taffy_node_ids: Vec::new(),
        }
    }

    /// Merges another set of spacer taffy nodes into this one, field by field.
    ///
    /// Unlike replacing the whole value (e.g. via `Map::extend` on a
    /// `Map<EdgeId, EdgeSpacerTaffyNodes>`), this preserves spacers of every
    /// kind. An edge built across multiple nesting levels accumulates a
    /// rank-based (LCA-gap) spacer at one level and cross-container spacers at
    /// each ancestor level, and all of them must be retained for the edge path
    /// to route correctly.
    pub fn merge(&mut self, other: EdgeSpacerTaffyNodes) {
        let EdgeSpacerTaffyNodes {
            rank_to_spacer_taffy_node_id,
            cross_container_spacer_taffy_node_ids,
            edge_desc_container_spacer_taffy_node_ids,
            text_content_spacer_taffy_node_ids,
        } = other;

        self.rank_to_spacer_taffy_node_id
            .extend(rank_to_spacer_taffy_node_id);
        self.cross_container_spacer_taffy_node_ids
            .extend(cross_container_spacer_taffy_node_ids);
        self.edge_desc_container_spacer_taffy_node_ids
            .extend(edge_desc_container_spacer_taffy_node_ids);
        self.text_content_spacer_taffy_node_ids
            .extend(text_content_spacer_taffy_node_ids);
    }

    /// Merges `other` into `target`, combining each edge's spacers field by
    /// field via [`EdgeSpacerTaffyNodes::merge`].
    ///
    /// This must be used instead of `Map::extend` whenever spacer maps from
    /// different nesting levels or build passes are combined, because
    /// `Map::extend` would replace the whole `EdgeSpacerTaffyNodes` for an edge
    /// that already has an entry and drop spacers of a different kind.
    pub fn map_merge<'id>(
        target: &mut crate::EdgeIdToEdgeSpacerTaffyNodes<'id>,
        other: crate::EdgeIdToEdgeSpacerTaffyNodes<'id>,
    ) {
        other.into_iter().for_each(|(edge_id, spacer_taffy_nodes)| {
            target.entry(edge_id).or_default().merge(spacer_taffy_nodes);
        });
    }
}

impl Default for EdgeSpacerTaffyNodes {
    fn default() -> Self {
        Self::new()
    }
}
