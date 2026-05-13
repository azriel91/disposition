use disposition_model_common::{edge::EdgeId, Map};
use serde::{Deserialize, Serialize};

use crate::edge::EdgeFaceAssignment;

/// Map of edge IDs to their pre-layout face assignments.
///
/// Holds one [`EdgeFaceAssignment`] per edge, computed before layout using
/// rank and sibling data. Used to build envelope taffy nodes with the right
/// number of edge label slots per face, and to derive [`NodeFaceEdges`].
///
/// # Examples
///
/// ```rust,ignore
/// EdgeFaceAssignments({
///   "edge_t_a__t_b__0": EdgeFaceAssignment { from_face: Some(Right), to_face: Some(Left) },
///   "edge_t_a__t_b__1": EdgeFaceAssignment { from_face: Some(Left),  to_face: Some(Right) },
///   "edge_t_b__t_b__0": EdgeFaceAssignment { from_face: None,        to_face: None },
/// })
/// ```
///
/// [`NodeFaceEdges`]: crate::node::NodeFaceEdges
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct EdgeFaceAssignments<'id>(Map<EdgeId<'id>, EdgeFaceAssignment>);

impl<'id> EdgeFaceAssignments<'id> {
    /// Returns a new empty `EdgeFaceAssignments` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the [`EdgeFaceAssignment`] for the given edge ID, if any.
    pub fn get(&self, edge_id: &EdgeId<'id>) -> Option<&EdgeFaceAssignment> {
        self.0.get(edge_id)
    }

    /// Inserts an assignment for the given edge ID.
    pub fn insert(&mut self, edge_id: EdgeId<'id>, assignment: EdgeFaceAssignment) {
        self.0.insert(edge_id, assignment);
    }

    /// Iterates over all `(EdgeId, EdgeFaceAssignment)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&EdgeId<'id>, &EdgeFaceAssignment)> {
        self.0.iter()
    }

    /// Converts this `EdgeFaceAssignments` into one with a `'static` lifetime.
    ///
    /// Clones any borrowed string data to produce owned values.
    pub fn into_static(self) -> EdgeFaceAssignments<'static> {
        EdgeFaceAssignments(
            self.0
                .into_iter()
                .map(|(edge_id, assignment)| {
                    (EdgeId::from(edge_id.into_inner().into_static()), assignment)
                })
                .collect(),
        )
    }
}
