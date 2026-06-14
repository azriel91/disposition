use disposition_ir_model::node::NodeId;
use disposition_model_common::Map;
use disposition_taffy_model::{
    taffy::{self, TaffyTree},
    MdNodeTaffyIds, NodeToTaffyNodeIds, TaffyNodeCtx,
};

use super::taffy_node_build_context::EdgeLabelLeafBuilt;

/// Mutable accumulators threaded through the taffy build functions.
///
/// Bundles the taffy tree being built and the output maps that are populated
/// incrementally as nodes are created. Passed by `&mut` so it can be reborrowed
/// down the recursive node-building call tree, complementing the immutable
/// [`TaffyBuildCtx`].
///
/// [`TaffyBuildCtx`]: super::taffy_build_ctx::TaffyBuildCtx
pub(crate) struct TaffyBuildState<'ctx> {
    /// The taffy tree that nodes are inserted into.
    pub(crate) taffy_tree: &'ctx mut TaffyTree<TaffyNodeCtx>,
    /// Map from each diagram node ID to the taffy node IDs that represent it.
    pub(crate) node_id_to_taffy: &'ctx mut Map<NodeId<'static>, NodeToTaffyNodeIds>,
    /// Reverse map from a taffy node ID to its diagram node ID.
    pub(crate) taffy_id_to_node: &'ctx mut Map<taffy::NodeId, NodeId<'static>>,
    /// Map from each diagram node ID to its envelope taffy node ID.
    ///
    /// Populated incrementally as each node's envelope is built.
    pub(crate) node_id_to_envelope_taffy_node: &'ctx mut Map<NodeId<'static>, taffy::NodeId>,
    /// Accumulator for edge label leaf nodes built across all envelope nodes.
    ///
    /// After all nodes are built, merged into `edge_label_taffy_nodes` in
    /// `TaffyNodeMappings`.
    pub(crate) edge_label_leaf_builts: &'ctx mut Vec<EdgeLabelLeafBuilt>,
    /// Accumulator for md node taffy IDs built across all diagram nodes.
    pub(crate) md_node_taffy_ids: &'ctx mut Map<NodeId<'static>, MdNodeTaffyIds>,
}
