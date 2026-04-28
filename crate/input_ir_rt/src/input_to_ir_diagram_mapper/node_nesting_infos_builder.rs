use disposition_ir_model::node::{NodeHierarchy, NodeId, NodeNestingInfo, NodeNestingInfos};

/// Builds the [`NodeNestingInfos`] map from a [`NodeHierarchy`].
///
/// Walks the hierarchy tree recursively, recording each node's depth
/// (nesting level), index path, and ancestor chain.
pub(crate) struct NodeNestingInfosBuilder;

impl NodeNestingInfosBuilder {
    /// Computes the nesting info for all nodes in the hierarchy.
    ///
    /// Walks the hierarchy tree recursively, recording each node's depth
    /// (nesting level), index path, and ancestor chain.
    pub(crate) fn build<'id>(node_hierarchy: &NodeHierarchy<'id>) -> NodeNestingInfos<'id> {
        let mut result = NodeNestingInfos::new();
        Self::build_recursive(node_hierarchy, &[], &[], &mut result);
        result
    }

    /// Recursive helper for building the nesting info map.
    fn build_recursive<'id>(
        hierarchy: &NodeHierarchy<'id>,
        parent_path: &[usize],
        parent_ancestor_chain: &[NodeId<'id>],
        result: &mut NodeNestingInfos<'id>,
    ) {
        for (index, (node_id, child_hierarchy)) in hierarchy.iter().enumerate() {
            let mut nesting_path = parent_path.to_vec();
            nesting_path.push(index);

            let mut ancestor_chain = parent_ancestor_chain.to_vec();
            ancestor_chain.push(node_id.clone());

            result.insert(
                node_id.clone(),
                NodeNestingInfo {
                    nesting_path: nesting_path.clone(),
                    ancestor_chain: ancestor_chain.clone(),
                },
            );

            if !child_hierarchy.is_empty() {
                Self::build_recursive(child_hierarchy, &nesting_path, &ancestor_chain, result);
            }
        }
    }
}
