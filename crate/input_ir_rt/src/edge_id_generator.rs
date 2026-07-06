use disposition_ir_model::edge::{EdgeGroups, EdgeId};
use disposition_model_common::{edge::EdgeGroupId, Id, Map};

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

    /// Builds a lookup from every edge instance ID to its edge group ID.
    ///
    /// Used to resolve the group-ID fallback for `edge_descs` / `edge_labels`
    /// at call sites that only have the edge instance ID at hand (e.g.
    /// envelope label-slot construction), mirroring
    /// `EdgeLabelBuilder::edge_id_to_node_ids_build`.
    pub fn edge_id_to_group_id_build(
        edge_groups: &EdgeGroups<'static>,
    ) -> Map<EdgeId<'static>, EdgeGroupId<'static>> {
        edge_groups
            .iter()
            .flat_map(|(edge_group_id, edge_group)| {
                edge_group
                    .iter()
                    .enumerate()
                    .map(move |(edge_index, _edge)| {
                        (
                            Self::generate(edge_group_id, edge_index),
                            edge_group_id.clone(),
                        )
                    })
            })
            .collect()
    }
}
