use disposition_ir_model::edge::EdgeId;
use disposition_model_common::Id;

/// Generates the entity ID used to key the tailwind classes for an
/// interaction edge's description background path.
pub struct EdgeDescBgIdGenerator;

impl EdgeDescBgIdGenerator {
    /// Generates the description background entity ID for an edge.
    ///
    /// Format: `"{edge_id}__desc_bg"`
    pub fn generate<'id>(edge_id: &EdgeId<'id>) -> Id<'static> {
        let desc_bg_id_str = format!("{edge_id}__desc_bg");
        Id::try_from(desc_bg_id_str).expect("description background ID should be valid")
    }
}
