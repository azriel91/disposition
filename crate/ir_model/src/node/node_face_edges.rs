use disposition_model_common::{edge::EdgeGroupId, Id, Map};
use serde::{Deserialize, Serialize};

use crate::{
    edge::{EdgeFaceAssignments, EdgeGroups, EdgeId},
    node::{NodeFace, NodeId},
};

/// Map from node ID and face to the edge IDs that exit or enter that face.
///
/// Derived from [`EdgeFaceAssignments`] and [`EdgeGroups`]. For every edge
/// with a non-`None` face assignment, the edge ID is appended to the list for
/// `from_node -> from_face` and `to_node -> to_face`.
///
/// Used by `IrToTaffyBuilder` to build the right number of edge label slots
/// on each face of each envelope node.
///
/// # Examples
///
/// ```rust,ignore
/// NodeFaceEdges({
///   "t_a": { Right: ["edge_t_a__t_b__0"], Left: ["edge_t_a__t_b__1"] },
///   "t_b": { Left:  ["edge_t_a__t_b__0"], Right: ["edge_t_a__t_b__1"] },
/// })
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct NodeFaceEdges<'id>(Map<NodeId<'id>, Map<NodeFace, Vec<EdgeId<'id>>>>);

impl<'id> NodeFaceEdges<'id> {
    /// Returns a new empty `NodeFaceEdges` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the edge IDs that exit or enter the given `face` of `node_id`.
    ///
    /// Returns an empty slice when the node has no edges on that face.
    pub fn edges_for(&self, node_id: &NodeId<'id>, face: NodeFace) -> &[EdgeId<'id>] {
        self.0
            .get(node_id)
            .and_then(|face_map| face_map.get(&face))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Returns the number of edges that exit or enter the given `face` of
    /// `node_id`.
    pub fn face_edge_count(&self, node_id: &NodeId<'id>, face: NodeFace) -> usize {
        self.edges_for(node_id, face).len()
    }

    /// Builds a `NodeFaceEdges` from pre-computed face assignments and edge
    /// groups.
    ///
    /// For every edge with a non-`None` face assignment in
    /// `edge_face_assignments`, the edge ID is appended to:
    ///
    /// * `from_node -> from_face` when `assignment.from_face` is `Some`.
    /// * `to_node -> to_face` when `assignment.to_face` is `Some`.
    ///
    /// The edge IDs are generated in the same order and with the same format
    /// as [`EdgeFaceAssignments`] (i.e. `"{edge_group_id}__{edge_index}"`),
    /// so lookups always find the correct assignment.
    pub fn from_assignments(
        edge_face_assignments: &EdgeFaceAssignments<'id>,
        edge_groups: &EdgeGroups<'id>,
    ) -> Self {
        let mut inner: Map<NodeId<'id>, Map<NodeFace, Vec<EdgeId<'id>>>> = Map::new();

        for (edge_group_id, edge_group) in edge_groups.iter() {
            for (edge_index, edge) in edge_group.iter().enumerate() {
                let edge_id = Self::edge_id_generate(edge_group_id, edge_index);

                let Some(assignment) = edge_face_assignments.get(&edge_id) else {
                    continue;
                };

                Self::face_edge_append(
                    &mut inner,
                    edge.from.clone(),
                    assignment.from_face,
                    edge_id.clone(),
                );
                Self::face_edge_append(&mut inner, edge.to.clone(), assignment.to_face, edge_id);
            }
        }

        NodeFaceEdges(inner)
    }

    /// Converts this `NodeFaceEdges` into one with a `'static` lifetime.
    pub fn into_static(self) -> NodeFaceEdges<'static> {
        NodeFaceEdges(
            self.0
                .into_iter()
                .map(|(node_id, face_map)| {
                    let face_map_static = face_map
                        .into_iter()
                        .map(|(face, edge_ids)| {
                            let edge_ids_static = edge_ids
                                .into_iter()
                                .map(|edge_id| EdgeId::from(edge_id.into_inner().into_static()))
                                .collect();
                            (face, edge_ids_static)
                        })
                        .collect();
                    (node_id.into_static(), face_map_static)
                })
                .collect(),
        )
    }

    /// Generates the edge ID for the edge at `edge_index` in `edge_group_id`.
    ///
    /// Mirrors the format used by `EdgeIdGenerator::generate`:
    /// `"{edge_group_id}__{edge_index}"`.
    fn edge_id_generate<'g>(edge_group_id: &EdgeGroupId<'g>, edge_index: usize) -> EdgeId<'g> {
        let edge_id_str = format!("{edge_group_id}__{edge_index}");
        Id::try_from(edge_id_str)
            .expect("edge ID should be valid")
            .into()
    }

    /// Appends `edge_id` to `inner[node_id][face]` when `face` is `Some`.
    fn face_edge_append(
        inner: &mut Map<NodeId<'id>, Map<NodeFace, Vec<EdgeId<'id>>>>,
        node_id: NodeId<'id>,
        face: Option<NodeFace>,
        edge_id: EdgeId<'id>,
    ) {
        let Some(face) = face else { return };
        inner
            .entry(node_id)
            .or_default()
            .entry(face)
            .or_default()
            .push(edge_id);
    }
}
