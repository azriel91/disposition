use disposition_ir_model::edge::EdgeId;
use disposition_model_common::{edge::EdgeGroupId, Id};

/// Generates `EdgeId`s for edges in an edge group.
pub struct EdgeIdGenerator;

impl EdgeIdGenerator {
    /// Generates an `EdgeId` from an edge group ID and edge index.
    ///
    /// Format: `"{edge_group_id}__{edge_index}"`
    pub fn generate<'id>(edge_group_id: &EdgeGroupId<'id>, edge_index: usize) -> EdgeId<'id> {
        let edge_id_str = format!("{edge_group_id}__{edge_index}");
        Id::try_from(edge_id_str)
            .expect("edge ID should be valid")
            .into()
    }
}
