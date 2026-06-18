use disposition_ir_model::{
    edge::Edge,
    node::{NodeId, NodeNestingInfos},
};

use crate::ir_to_taffy_builder::edge_spacer_builder::{
    EdgeSpacerBuildDecision, EdgeSpacerBuildDecisionBuild, EdgeSpacerBuildDecisionSkip,
};

/// Decides whether edge spacers should be built for the given edge.
pub struct EdgeSpacerBuildDecider;

impl EdgeSpacerBuildDecider {
    /// Returns whether edge spacers should be built for the given edge.
    ///
    /// # Parameters
    ///
    /// * `node_nesting_infos`: The precomputed nesting info map for all nodes.
    /// * `container_node_id`: The ID of the node which is a parent of the node
    ///   that the edge is connected to.
    /// * `container_node_direct_child_ids`: The children of
    ///   `container_node_id`.
    /// * `edge`: Edge to build spacers for.
    pub fn decide<'f, 'id>(
        node_nesting_infos: &'f NodeNestingInfos<'id>,
        container_node_id: &NodeId<'id>,
        container_node_direct_child_ids: &Vec<NodeId<'id>>,
        edge: &Edge<'id>,
    ) -> EdgeSpacerBuildDecision<'f, 'id> {
        let Some(node_nesting_info_from) = node_nesting_infos.get(&edge.from) else {
            return EdgeSpacerBuildDecision::Skip(
                EdgeSpacerBuildDecisionSkip::NestingInfoFromNotFound {
                    node_id: edge.from.clone(),
                },
            );
        };
        let Some(node_nesting_info_to) = node_nesting_infos.get(&edge.to) else {
            return EdgeSpacerBuildDecision::Skip(
                EdgeSpacerBuildDecisionSkip::NestingInfoToNotFound {
                    node_id: edge.to.clone(),
                },
            );
        };

        // Whether cross-container spacers are needed depends on the target's
        // rank *inside* the container (handled by the insertion loop in
        // `build_cross_container_spacers_for_edge`, which only routes around
        // siblings at ranks strictly before the target). It is independent of
        // the root-level distance between the divergent ancestors -- an edge
        // into a deeply-ranked child still has to route around the container's
        // lower-rank siblings even when the divergent ancestors are adjacent.

        // Determine if exactly one endpoint is inside this container
        // and the other is outside.
        let container_node_contains_node_from = node_nesting_info_from
            .ancestor_chain
            .contains(container_node_id);
        let container_node_contains_node_to = node_nesting_info_to
            .ancestor_chain
            .contains(container_node_id);

        // We create a spacer node for edges where one node is inside the container and
        // one is outside.
        match (
            container_node_contains_node_from,
            container_node_contains_node_to,
        ) {
            (true, true) => {
                return EdgeSpacerBuildDecision::Skip(
                    EdgeSpacerBuildDecisionSkip::ContainerNodeContainsBothFromAndToNodes {
                        node_id_container: container_node_id.clone(),
                        node_id_from: edge.from.clone(),
                        node_id_to: edge.to.clone(),
                    },
                )
            }
            (false, false) => {
                return EdgeSpacerBuildDecision::Skip(
                    EdgeSpacerBuildDecisionSkip::ContainerNodeContainsNeitherFromAndToNodes {
                        node_id_container: container_node_id.clone(),
                        node_id_from: edge.from.clone(),
                        node_id_to: edge.to.clone(),
                    },
                )
            }
            // Continue checking if the edge needs a spacer across the container.
            (true, false) | (false, true) => {}
        }

        // Determine which endpoint is inside and which is outside.
        let node_nesting_info_of_contained_node = if container_node_contains_node_from {
            node_nesting_info_from
        } else {
            node_nesting_info_to
        };

        // Find which direct child of this container is the ancestor of the inside
        // endpoint. The ancestor chain includes the inside endpoint itself, so we look
        // for the container in the chain and take the next element.
        //
        // # Example
        //
        // ```yaml
        // node hierarchy:
        //   a:
        //     a0:
        //       a00:
        //         a000: {}
        //     a1:
        //       a10: {}
        // ```
        //
        // For `a000`, the `ancestor_chain` is `["a", "a0", "a00", "a000"]`.
        //
        // The container depth of `a0` in the chain is `1` (the index of `a0` in the
        // chain).
        let container_depth_in_chain = node_nesting_info_of_contained_node
            .ancestor_chain
            .iter()
            .position(|id| id == container_node_id);
        let container_depth = container_depth_in_chain
            .expect("We just confirmed the `container_node` is in this node's `ancestor_chain`.");

        // Skip creating the spacer node if the container itself is one of the endpoints
        // (ancestor_chain includes self, so check that the inside endpoint is not the
        // container itself).
        //
        // The `target_child_id` is the node ID of the direct child of the container
        // node, which *may* be the inside endpoint.
        let target_child_id = node_nesting_info_of_contained_node
            .ancestor_chain
            .get(container_depth + 1);
        let target_child_id = match target_child_id {
            Some(target_child_id) => target_child_id,
            None => {
                // The container node is the deepest element, i.e. the inside endpoint IS the
                // container node, so skip creating a spacer node.
                return EdgeSpacerBuildDecision::Skip(
                    EdgeSpacerBuildDecisionSkip::ContainerNodeIsFromOrToNode {
                        node_id_from: edge.from.clone(),
                        node_id_to: edge.to.clone(),
                    },
                );
            }
        };

        // Find the index of the target child among the direct children.
        let _target_child_index = container_node_direct_child_ids
                .iter()
                .position(|id| id == target_child_id)
                .expect("`target_child_id` was just looked up from the `ancestor_chain` at `container_depth + 1`.");

        EdgeSpacerBuildDecision::Build(EdgeSpacerBuildDecisionBuild { target_child_id })
    }
}
