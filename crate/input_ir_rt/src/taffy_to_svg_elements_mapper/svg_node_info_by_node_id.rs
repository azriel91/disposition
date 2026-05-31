use disposition_ir_model::node::NodeId;
use disposition_model_common::Map;
use disposition_svg_model::SvgNodeInfo;

/// Lookup map from each node ID to its [`SvgNodeInfo`], used while building
/// edges.
///
/// Both the key and value borrow from the `Vec<SvgNodeInfo>` built earlier in
/// the mapping, so they share the `'a` lifetime.
pub(super) type SvgNodeInfoByNodeId<'a, 'id> = Map<&'a NodeId<'id>, &'a SvgNodeInfo<'id>>;
