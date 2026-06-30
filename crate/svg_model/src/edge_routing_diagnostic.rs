use disposition_ir_model::{
    edge::EdgeId,
    node::{NodeFace, NodeId, NodeRank},
};
use disposition_model_common::edge::EdgeGroupId;
use serde::{Deserialize, Serialize};

use crate::{EdgePathBounds, EdgePathMidpoint, OrthoProtrusionParams};

/// Diagnostic snapshot of the pass-1, offset, and protrusion values
/// computed for a single edge during edge routing.
///
/// This gathers, per edge, the intermediate values the edge router
/// computes and then discards: the pass-1 face assignments and ranks,
/// the assigned face-contact slot indices and resolved offsets, and the
/// final orthogonal protrusion parameters. It is purely diagnostic --
/// nothing in the render pipeline reads it back.
///
/// # Example values
///
/// ```rust,ignore
/// EdgeRoutingDiagnostic {
///     edge_id: EdgeId::new("edge_dep_alice_bob__0")?,
///     edge_group_id: EdgeGroupId::new("edge_dep_alice_bob")?,
///     from_node_id: NodeId::new("t_alice")?,
///     to_node_id: NodeId::new("t_bob")?,
///     from_face: Some(NodeFace::Right),
///     to_face: Some(NodeFace::Left),
///     rank_from: NodeRank::new(0),
///     rank_to: NodeRank::new(0),
///     rank_distance: 0,
///     is_cycle_edge: true,
///     is_interaction: false,
///     from_node_x: 37.0,
///     from_node_y: 93.0,
///     to_node_x: 186.0,
///     to_node_y: 26.0,
///     from_slot_index: Some(0),
///     to_slot_index: Some(0),
///     from_face_offset: 0.0,
///     to_face_offset: 0.0,
///     ortho_protrusion_params: OrthoProtrusionParams {
///         from_protrusion: 61.0,
///         to_protrusion: 17.0,
///         spacer_protrusions: Vec::new(),
///     },
///     path_midpoint: EdgePathMidpoint { x: 130.0, y: 70.0 },
///     path_bounds: EdgePathBounds { x_min: 94.0, x_max: 186.0, y_min: 36.0, y_max: 102.6 },
/// }
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct EdgeRoutingDiagnostic<'id> {
    /// ID of the edge this diagnostic represents.
    pub edge_id: EdgeId<'id>,
    /// ID of the edge group this edge belongs to.
    pub edge_group_id: EdgeGroupId<'id>,
    /// The source node ID where this edge originates.
    pub from_node_id: NodeId<'id>,
    /// The target node ID where this edge points to.
    pub to_node_id: NodeId<'id>,
    /// Which face of the "from" node this edge connects to.
    ///
    /// `None` when the edge connects a contained node (no face offset
    /// applies).
    pub from_face: Option<NodeFace>,
    /// Which face of the "to" node this edge connects to.
    ///
    /// `None` when the edge connects a contained node (no face offset
    /// applies).
    pub to_face: Option<NodeFace>,
    /// Rank of the edge's "from" node.
    pub rank_from: NodeRank,
    /// Rank of the edge's "to" node.
    pub rank_to: NodeRank,
    /// Absolute difference in `NodeRank` between the edge's "from" and
    /// "to" nodes.
    pub rank_distance: u32,
    /// Whether this edge uses cycle (same-rank clockwise) face routing.
    pub is_cycle_edge: bool,
    /// Whether this edge is an interaction edge (`true`) or a dependency
    /// edge (`false`).
    pub is_interaction: bool,
    /// X coordinate of the edge's "from" node (absolute position).
    pub from_node_x: f32,
    /// Y coordinate of the edge's "from" node (absolute position).
    pub from_node_y: f32,
    /// X coordinate of the edge's "to" node (absolute position).
    pub to_node_x: f32,
    /// Y coordinate of the edge's "to" node (absolute position).
    pub to_node_y: f32,
    /// Assigned face-contact slot index for the "from" endpoint.
    ///
    /// `None` if no face offset applies (e.g. contained edges).
    pub from_slot_index: Option<usize>,
    /// Assigned face-contact slot index for the "to" endpoint.
    ///
    /// `None` if no face offset applies (e.g. contained edges).
    pub to_slot_index: Option<usize>,
    /// Resolved pixel offset of the "from" contact from its face midpoint.
    pub from_face_offset: f32,
    /// Resolved pixel offset of the "to" contact from its face midpoint.
    pub to_face_offset: f32,
    /// Final orthogonal protrusion parameters computed for this edge.
    pub ortho_protrusion_params: OrthoProtrusionParams,
    /// Mean anchor point of the zero-offset (pass-1) path.
    pub path_midpoint: EdgePathMidpoint,
    /// Axis-aligned bounding box of the zero-offset (pass-1) path's
    /// anchor points.
    pub path_bounds: EdgePathBounds,
}
