use disposition_ir_model::edge::EdgeId;
use disposition_model_common::Id;

/// Generates the entity ID used to key the tailwind classes for an
/// interaction edge's halo path.
pub struct EdgeHaloIdGenerator;

impl EdgeHaloIdGenerator {
    /// Generates the halo entity ID for an edge.
    ///
    /// Format: `"{edge_id}__halo"`
    pub fn generate<'id>(edge_id: &EdgeId<'id>) -> Id<'static> {
        let halo_id_str = format!("{edge_id}__halo");
        Id::try_from(halo_id_str).expect("halo ID should be valid")
    }
}
