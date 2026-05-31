use disposition_ir_model::node::NodeId;
use disposition_model_common::Map;

use crate::MdImageSpan;

/// Inline image spans computed after taffy layout for markdown nodes, keyed by
/// diagram `NodeId`.
pub type NodeIdToImageSpans<'id> = Map<NodeId<'id>, Vec<MdImageSpan>>;
