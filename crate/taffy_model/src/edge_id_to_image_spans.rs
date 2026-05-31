use disposition_ir_model::edge::EdgeId;
use disposition_model_common::Map;

use crate::MdImageSpan;

/// Inline image spans for edge descriptions that used the markdown path, keyed
/// by `EdgeId`.
pub type EdgeIdToImageSpans<'id> = Map<EdgeId<'id>, Vec<MdImageSpan>>;
