use disposition_ir_model::node::{NodeId, NodeNestingInfo};

use crate::ir_to_taffy_builder::EdgeLcaSiblingDistance;

/// Reason for not building the edge spacer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EdgeSpacerBuildDecisionSkip<'id> {
    /// The nesting info for the from node was not found.
    ///
    /// In general this is a bug, because the `NodeNestingInfo` is calculated in
    /// `InputToIrDiagramMapper` for every node.
    NestingInfoFromNotFound { node_id: NodeId<'id> },
    /// The nesting info for the to node was not found.
    ///
    /// In general this is a bug, because the `NodeNestingInfo` is calculated in
    /// `InputToIrDiagramMapper` for every node.
    NestingInfoToNotFound { node_id: NodeId<'id> },
    NoIntermediateLcaSiblings {
        /// ID of the node that the edge begins from.
        node_id_from: NodeId<'id>,
        /// ID of the node that the edge ends at.
        node_id_to: NodeId<'id>,
        /// The `NodeNestingInfo` for the `from` node.
        node_nesting_info_from: NodeNestingInfo<'id>,
        /// The `NodeNestingInfo` for the `to` node.
        node_nesting_info_to: NodeNestingInfo<'id>,
        /// The distance between the LCA sibling of the `from` node and the `to`
        /// node.
        edge_lca_sibling_distance: EdgeLcaSiblingDistance,
    },
    /// The container node contains both the `from` and the `to` node, so skip
    /// creating the spacer node.
    ContainerNodeContainsBothFromAndToNodes {
        node_id_container: NodeId<'id>,
        node_id_from: NodeId<'id>,
        node_id_to: NodeId<'id>,
    },
    /// The container node contains neither the `from` nor the `to` node, so
    /// skip creating the spacer node.
    ContainerNodeContainsNeitherFromAndToNodes {
        node_id_container: NodeId<'id>,
        node_id_from: NodeId<'id>,
        node_id_to: NodeId<'id>,
    },
    /// The container node is one of the endpoints, so skip creating the spacer
    /// node.
    ContainerNodeIsFromOrToNode {
        node_id_from: NodeId<'id>,
        node_id_to: NodeId<'id>,
    },
}
