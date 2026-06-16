use disposition_ir_model::node::{NodeFace, NodeId, NodeInbuilt, NodeRank};

/// The structural role of a `taffy` node that does not directly correspond to a
/// diagram entity, edge spacer, or edge label.
///
/// These are the wrapper / container nodes inserted while building the taffy
/// tree (envelopes, rank containers, rank stacking containers). They are
/// tracked so the taffy tree can be printed with a meaningful label for each
/// node (e.g. `t_outer_envelope`) instead of taffy's generic debug label (e.g.
/// `GRID`).
///
/// Nodes that already carry a label elsewhere are intentionally excluded:
///
/// * Diagram nodes and inbuilt containers -- via
///   [`TaffyNodeToNodeId`](crate::TaffyNodeToNodeId).
/// * Edge spacers, edge labels, edge descriptions, markdown tokens / images --
///   via [`TaffyNodeCtx`](crate::TaffyNodeCtx).
#[derive(Clone, Debug, PartialEq)]
pub enum TaffyNodeKind<'id> {
    /// The 3x3 grid envelope that wraps a diagram node, reserving edge-label
    /// slots on each face.
    ///
    /// Labelled `{node_id}_envelope`, e.g. `t_outer_envelope`.
    Envelope {
        /// The diagram node this envelope wraps.
        node_id: NodeId<'id>,
    },
    /// One of the four per-face wrappers inside a node's envelope that hold the
    /// edge-label slots for that face.
    ///
    /// Labelled `{node_id}_edge_wrapper_{face}`, e.g. `t_a0_edge_wrapper_top`.
    EnvelopeFaceWrapper {
        /// The diagram node whose envelope this face wrapper belongs to.
        node_id: NodeId<'id>,
        /// The face of the node this wrapper holds edge-label slots for.
        face: NodeFace,
    },
    /// The container that stacks a nested container node's per-rank child
    /// containers along the rank axis.
    ///
    /// Labelled `{node_id}_rank_stacking_container`, e.g.
    /// `t_outer_rank_stacking_container`.
    RankStackingContainer {
        /// The container diagram node whose ranks are stacked.
        node_id: NodeId<'id>,
    },
    /// A per-rank child container within a nested container diagram node.
    ///
    /// Labelled `{node_id}_rank_container_{rank}`, e.g.
    /// `t_outer_rank_container_1`.
    RankContainer {
        /// The container diagram node this rank container belongs to.
        node_id: NodeId<'id>,
        /// The rank of the nodes held by this container.
        rank: NodeRank,
    },
    /// A per-rank container for first-level nodes of an entity type -- the
    /// children of an inbuilt things / processes / tags container.
    ///
    /// Labelled `{node_inbuilt}_rank_container_{rank}`, e.g.
    /// `_things_container_rank_0`.
    FirstLevelRankContainer {
        /// The inbuilt container these first-level rank containers sit under.
        node_inbuilt: NodeInbuilt,
        /// The rank of the nodes held by this container.
        rank: NodeRank,
    },
}

impl TaffyNodeKind<'_> {
    /// Returns the display label for this node kind, used when printing the
    /// taffy tree.
    pub fn label(&self) -> String {
        match self {
            Self::Envelope { node_id } => format!("{node_id}_envelope"),
            Self::EnvelopeFaceWrapper { node_id, face } => {
                format!("{node_id}_edge_wrapper_{face}")
            }
            Self::RankStackingContainer { node_id } => {
                format!("{node_id}_rank_stacking_container")
            }
            Self::RankContainer { node_id, rank } => {
                format!("{node_id}_rank_container_{rank}")
            }
            Self::FirstLevelRankContainer { node_inbuilt, rank } => {
                format!("{node_inbuilt}_rank_container_{rank}")
            }
        }
    }
}
