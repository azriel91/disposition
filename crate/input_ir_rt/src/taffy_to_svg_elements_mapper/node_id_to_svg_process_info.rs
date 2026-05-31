use disposition_ir_model::node::NodeId;
use disposition_model_common::Map;
use disposition_svg_model::SvgProcessInfo;

/// Map of each process node ID to its computed [`SvgProcessInfo`].
pub(super) type NodeIdToSvgProcessInfo<'id> = Map<NodeId<'id>, SvgProcessInfo<'id>>;
