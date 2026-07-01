use disposition_ir_model::{edge::EdgeId, node::NodeRank};
use serde::{Deserialize, Serialize};

use crate::{RankGapDiagnosticEndpointKind, RankGapDiagnosticSide};

/// Diagnostic snapshot of a single rank-gap entry considered by the
/// orthogonal protrusion calculator.
///
/// Each entry represents one endpoint (or spacer side) of one edge as it
/// transits the gap between two adjacent ranks. These are the inputs the
/// calculator uses to assign protrusion depths -- exposed here so the
/// intermediate routing decisions can be inspected without re-deriving
/// them.
///
/// The `cross_axis_coord` and `jog_far_cross_axis` pair defines the
/// cross-axis span swept by this endpoint's lateral "jog" leg in the gap;
/// two legs only "read as one line" when their spans overlap.
///
/// # Example values
///
/// ```rust,ignore
/// RankGapEntryDiagnostic {
///     rank_low: NodeRank::new(0),
///     rank_high: NodeRank::new(1),
///     edge_id: EdgeId::new("edge_dep_alice_bob__0")?,
///     endpoint_kind: RankGapDiagnosticEndpointKind::FromEndpoint,
///     gap_side: RankGapDiagnosticSide::Low,
///     cross_axis_coord: 102.6,
///     jog_far_cross_axis: 36.0,
///     face_offset: 6.0,
///     rank_gap_px: 61.0,
///     envelope_clearance: 0.0,
/// }
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct RankGapEntryDiagnostic<'id> {
    /// Lower rank boundary of the gap this entry belongs to.
    pub rank_low: NodeRank,
    /// Upper rank boundary of the gap this entry belongs to.
    pub rank_high: NodeRank,
    /// ID of the edge this entry represents an endpoint of.
    pub edge_id: EdgeId<'id>,
    /// Which endpoint or spacer side of the edge this entry represents.
    pub endpoint_kind: RankGapDiagnosticEndpointKind,
    /// Which side of the rank gap this entry protrudes from.
    pub gap_side: RankGapDiagnosticSide,
    /// Cross-axis coordinate of the endpoint's node (or spacer).
    ///
    /// For `Top` / `Bottom` faces this is the node's X coordinate; for
    /// `Left` / `Right` faces this is the node's Y coordinate.
    pub cross_axis_coord: f32,
    /// Cross-axis coordinate of the next contact along the path (the far
    /// end of this endpoint's lateral jog leg).
    pub jog_far_cross_axis: f32,
    /// The face offset (slot offset) for this endpoint. Edges further
    /// from the face midpoint receive longer protrusions.
    pub face_offset: f32,
    /// Pixel distance in the rank direction available to this endpoint's
    /// protrusion within the gap.
    pub rank_gap_px: f32,
    /// Fixed clearance (in pixels) added to the band-distributed
    /// protrusion before it is written, spanning the node's own
    /// edge-label wrapper.
    pub envelope_clearance: f32,
}
