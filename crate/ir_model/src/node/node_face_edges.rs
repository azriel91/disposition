use disposition_model_common::{edge::EdgeGroupId, Id, Map};
use serde::{Deserialize, Serialize};

use crate::{
    edge::{EdgeFaceAssignments, EdgeGroups, EdgeId},
    node::{NodeFace, NodeId, NodeNestingInfos},
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
    ///
    /// Within each face the edge IDs are then ordered by the **other**
    /// endpoint's structural position (its [`NodeNestingInfo::nesting_path`],
    /// whose leading component is the divergent sibling index). Ordering each
    /// face's edges by where the opposite endpoint sits -- rather than by edge
    /// declaration order -- lays out the edge label slots (and therefore the
    /// edge contact points) so that paths to/from spatially adjacent nodes get
    /// adjacent slots, minimising edge crossings.
    ///
    /// [`NodeNestingInfo::nesting_path`]:
    /// crate::node::NodeNestingInfo::nesting_path
    pub fn from_assignments(
        edge_face_assignments: &EdgeFaceAssignments<'id>,
        edge_groups: &EdgeGroups<'id>,
        node_nesting_infos: &NodeNestingInfos<'id>,
    ) -> Self {
        let mut inner: Map<NodeId<'id>, Map<NodeFace, Vec<EdgeId<'id>>>> = Map::new();

        // Endpoints per edge ID, so each face's edge list can be ordered by the
        // *other* endpoint relative to the face-owning node.
        let mut edge_endpoints: Map<EdgeId<'id>, EdgeEndpoints<'id>> = Map::new();

        for (edge_group_id, edge_group) in edge_groups.iter() {
            for (edge_index, edge) in edge_group.iter().enumerate() {
                let edge_id = Self::edge_id_generate(edge_group_id, edge_index);

                let Some(assignment) = edge_face_assignments.get(&edge_id) else {
                    continue;
                };

                edge_endpoints.insert(
                    edge_id.clone(),
                    EdgeEndpoints {
                        from: edge.from.clone(),
                        to: edge.to.clone(),
                    },
                );

                Self::face_edge_append(
                    &mut inner,
                    edge.from.clone(),
                    assignment.from_face,
                    edge_id.clone(),
                );
                Self::face_edge_append(&mut inner, edge.to.clone(), assignment.to_face, edge_id);
            }
        }

        Self::face_edges_order_by_other_endpoint(&mut inner, &edge_endpoints, node_nesting_infos);

        NodeFaceEdges(inner)
    }

    /// Orders every face's edge list by the structural position of each edge's
    /// *other* endpoint (the endpoint that is not the face-owning node).
    ///
    /// The sort key is the other endpoint's
    /// [`nesting_path`](crate::node::NodeNestingInfo::nesting_path), compared
    /// lexicographically, with the edge ID as a deterministic tiebreaker. A
    /// missing nesting path sorts as empty (before all others).
    fn face_edges_order_by_other_endpoint(
        inner: &mut Map<NodeId<'id>, Map<NodeFace, Vec<EdgeId<'id>>>>,
        edge_endpoints: &Map<EdgeId<'id>, EdgeEndpoints<'id>>,
        node_nesting_infos: &NodeNestingInfos<'id>,
    ) {
        const EMPTY_NESTING_PATH: &[usize] = &[];

        for (node_id, face_map) in inner.iter_mut() {
            for edge_ids in face_map.values_mut() {
                if edge_ids.len() <= 1 {
                    continue;
                }

                let other_nesting_path = |edge_id: &EdgeId<'id>| -> &[usize] {
                    edge_endpoints
                        .get(edge_id)
                        .map(|edge_endpoints| edge_endpoints.other_than(node_id))
                        .and_then(|other_node_id| node_nesting_infos.get(other_node_id))
                        .map(|node_nesting_info| node_nesting_info.nesting_path.as_slice())
                        .unwrap_or(EMPTY_NESTING_PATH)
                };

                edge_ids.sort_by(|edge_id_a, edge_id_b| {
                    other_nesting_path(edge_id_a)
                        .cmp(other_nesting_path(edge_id_b))
                        .then_with(|| edge_id_a.as_str().cmp(edge_id_b.as_str()))
                });
            }
        }
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

/// The two endpoint node IDs of an edge.
///
/// Used by [`NodeFaceEdges::from_assignments`] to look up the endpoint
/// *opposite* a face-owning node when ordering that face's edge list.
struct EdgeEndpoints<'id> {
    /// The edge's `from` (source) node ID.
    from: NodeId<'id>,
    /// The edge's `to` (target) node ID.
    to: NodeId<'id>,
}

impl<'id> EdgeEndpoints<'id> {
    /// Returns the endpoint that is not `node_id`.
    ///
    /// For a self-loop (`from == to == node_id`) the `from` endpoint is
    /// returned, which equals `node_id`.
    fn other_than(&self, node_id: &NodeId<'id>) -> &NodeId<'id> {
        if &self.from == node_id {
            &self.to
        } else {
            &self.from
        }
    }
}
