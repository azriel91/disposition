use disposition_ir_model::node::NodeId;

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
