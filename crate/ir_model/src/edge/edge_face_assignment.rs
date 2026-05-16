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
/// A self-loop edge (where `from == to`) exits the bottom face of the node;
/// `to_face` is `None` since only one label slot is needed:
///
/// ```rust,ignore
/// EdgeFaceAssignment { from_face: Some(NodeFace::Bottom), to_face: None }
/// ```
///
/// A contained edge where `from` is an ancestor of `to` in a top-to-bottom
/// diagram uses the same faces as a forward edge:
///
/// ```rust,ignore
/// EdgeFaceAssignment { from_face: Some(NodeFace::Bottom), to_face: Some(NodeFace::Top) }
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct EdgeFaceAssignment {
    /// The face of the `from` node that this edge exits, if any.
    ///
    /// `None` when the node is absent from `NodeNestingInfos`.
    pub from_face: Option<NodeFace>,

    /// The face of the `to` node that this edge enters, if any.
    ///
    /// `None` for self-loop edges (where `from == to`), since only a single
    /// label slot on the `from_face` is used. Also `None` when the node is
    /// absent from `NodeNestingInfos`.
    pub to_face: Option<NodeFace>,
}
