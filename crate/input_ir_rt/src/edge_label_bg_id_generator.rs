use disposition_ir_model::edge::EdgeId;
use disposition_model_common::Id;

/// Generates the entity ID used to key the tailwind classes for an
/// interaction edge's label background path.
pub struct EdgeLabelBgIdGenerator;

impl EdgeLabelBgIdGenerator {
    /// Generates the label background entity ID for an edge.
    ///
    /// Format: `"{edge_id}__label_bg"`
    pub fn generate<'id>(edge_id: &EdgeId<'id>) -> Id<'static> {
        let label_bg_id_str = format!("{edge_id}__label_bg");
        Id::try_from(label_bg_id_str).expect("label background ID should be valid")
    }
}
