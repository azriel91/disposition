use disposition_ir_model::{
    edge::{EdgeFaceAssignments, EdgeGroups, EdgeId},
    node::NodeId,
};
use disposition_model_common::{Id, Map};
use disposition_taffy_model::EdgeLabelTaffyNodeIds;

use super::taffy_node_build_context::EdgeLabelLeafBuilt;

/// Assembles the edge label taffy node map from leaf nodes collected during
/// envelope construction.
pub(crate) struct EdgeLabelBuilder;

impl EdgeLabelBuilder {
    /// Builds the `edge_label_taffy_nodes` map by merging per-node label
    /// leaves collected during envelope construction.
    ///
    /// For each [`EdgeLabelLeafBuilt`], the raw edge endpoints are looked up
    /// via `edge_groups` and compared against the leaf's `node_id` to
    /// determine whether the leaf is the `from` or `to` slot for that edge.
    /// Self-loop edges (where `from == to`) use only a `from_label` slot;
    /// their `to_face` is `None`, so the `to_label` slot is never populated.
    pub(crate) fn build(
        edge_label_leaves: Vec<EdgeLabelLeafBuilt>,
        edge_face_assignments: &EdgeFaceAssignments<'static>,
        edge_groups: &EdgeGroups<'static>,
    ) -> Map<EdgeId<'static>, EdgeLabelTaffyNodeIds> {
        let edge_id_to_node_ids = Self::edge_id_to_node_ids_build(edge_groups);

        let mut edge_label_taffy_nodes: Map<EdgeId<'static>, EdgeLabelTaffyNodeIds> = Map::new();
        for built in edge_label_leaves {
            let EdgeLabelLeafBuilt {
                edge_id,
                node_id,
                face: _,
                taffy_node_id,
                md_node_taffy_ids,
            } = built;

            let Some((from_node_id, to_node_id)) = edge_id_to_node_ids.get(&edge_id) else {
                continue;
            };

            // Only create an entry when there is a face assignment to populate.
            let Some(assignment) = edge_face_assignments.get(&edge_id) else {
                continue;
            };

            let is_from_slot = &node_id == from_node_id && assignment.from_face.is_some();
            let is_to_slot = &node_id == to_node_id && assignment.to_face.is_some();

            let entry = edge_label_taffy_nodes
                .entry(edge_id)
                .or_insert(EdgeLabelTaffyNodeIds {
                    from_label_taffy_node_id: None,
                    to_label_taffy_node_id: None,
                    from_label_md_node_taffy_ids: None,
                    to_label_md_node_taffy_ids: None,
                });

            if is_from_slot {
                entry.from_label_taffy_node_id = Some(taffy_node_id);
                entry.from_label_md_node_taffy_ids = md_node_taffy_ids;
            } else if is_to_slot {
                entry.to_label_taffy_node_id = Some(taffy_node_id);
                entry.to_label_md_node_taffy_ids = md_node_taffy_ids;
            }
        }

        edge_label_taffy_nodes
    }

    /// Builds a lookup from each edge ID to the node IDs of its endpoints.
    ///
    /// The edge ID format mirrors `NodeFaceEdges::edge_id_generate`:
    /// `"{edge_group_id}__{edge_index}"`.
    pub(crate) fn edge_id_to_node_ids_build(
        edge_groups: &EdgeGroups<'static>,
    ) -> Map<EdgeId<'static>, (NodeId<'static>, NodeId<'static>)> {
        edge_groups
            .iter()
            .flat_map(|(edge_group_id, edge_group)| {
                edge_group
                    .iter()
                    .enumerate()
                    .map(|(edge_index, edge)| {
                        let edge_id_str = format!("{edge_group_id}__{edge_index}");
                        let edge_id: EdgeId<'static> = Id::try_from(edge_id_str)
                            .expect("edge group ID and index should produce a valid edge ID")
                            .into();
                        (edge_id, (edge.from.clone(), edge.to.clone()))
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}
