use serde::{Deserialize, Serialize};

use crate::node::NodeFace;

/// Pre-layout face assignment for one edge's two endpoints.
///
/// Computed before layout using rank and sibling data to determine which
/// face of each endpoint node the edge exits or enters. Used to build
/// envelope taffy nodes with the right number of edge label slots per face.
///
/// # Examples
///
/// A forward edge in a left-to-right diagram exits the right face of `from`
/// and enters the left face of `to`:
///
/// ```rust,ignore
/// EdgeFaceAssignment { from_face: Some(NodeFace::Right), to_face: Some(NodeFace::Left) }
/// ```
///
/// A contained edge (one endpoint is an ancestor of the other) has no face
/// assignment:
///
/// ```rust,ignore
/// EdgeFaceAssignment { from_face: None, to_face: None }
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct EdgeFaceAssignment {
    /// The face of the `from` node that this edge exits, if any.
    ///
    /// `None` for contained edges (one endpoint is an ancestor of the other).
    pub from_face: Option<NodeFace>,

    /// The face of the `to` node that this edge enters, if any.
    ///
    /// `None` for contained edges.
    pub to_face: Option<NodeFace>,
}
