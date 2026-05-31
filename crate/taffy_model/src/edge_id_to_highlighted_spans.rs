use disposition_ir_model::edge::EdgeId;
use disposition_model_common::Map;

use crate::EntityHighlightedSpan;

/// Highlighted spans computed for each edge description leaf node, keyed by
/// `EdgeId`.
pub type EdgeIdToHighlightedSpans<'id> = Map<EdgeId<'id>, Vec<EntityHighlightedSpan>>;
