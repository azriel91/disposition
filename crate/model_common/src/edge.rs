pub use self::{
    edge_curvature::EdgeCurvature, edge_descs::EdgeDescs, edge_group_id::EdgeGroupId,
    edge_id::EdgeId, edge_label::EdgeLabel, edge_labels::EdgeLabels,
};

mod edge_curvature;
mod edge_descs;
mod edge_group_id;
mod edge_id;
mod edge_label;
mod edge_labels;

// === Orthogonal edge geometry === //
//
// Shared constants for orthogonal (90-degree) edge routing -- protrusion
// depths, arrow head clearance, and corner rounding. Defined here so the
// runtime and its tests reference a single source of truth.

/// Maximum fraction of the rank gap available for protrusions.
///
/// Within a rank gap, the from-side and to-side protrusion fans share this
/// single band (split proportionally to each side's endpoint count), so the
/// deepest from-tip plus the deepest to-tip never exceed `MAX_GAP_FRACTION *
/// gap`. The remaining `(1 - MAX_GAP_FRACTION) * gap` is left as the central
/// routing channel.
///
/// # Example values
///
/// `0.8` -- from and to protrusions together use up to 80% of the gap.
pub const MAX_GAP_FRACTION: f32 = 0.8;

/// Minimum protrusion length in pixels.
///
/// When an edge is not perfectly straight (i.e. the from and to
/// contact points differ on the cross-axis), the protrusion is at
/// least this many pixels so the perpendicular stub is visible.
pub const MIN_PROTRUSION_PX: f32 = 3.0;

/// Length of the arrowhead from tip to base, in pixels.
///
/// Also consumed when sizing protrusions to keep the orthogonal Z/S bend clear
/// of the arrow head at the to-endpoint.
pub const ARROW_HEAD_LENGTH: f64 = 8.0;

/// Clearance in pixels between the orthogonal Z/S bend and the base of
/// the arrow head at the to-endpoint.
///
/// # Example values
///
/// `3.0` -- the bend starts at least 3 px before the arrow head base.
pub const ARROW_HEAD_CLEARANCE_PX: f32 = 3.0;

/// Minimum protrusion length in pixels for to-endpoints.
///
/// Every edge has an arrow head drawn at its to-endpoint, occupying
/// `ARROW_HEAD_LENGTH` (8.0 px) of the path's final straight segment.
/// The to-protrusion is floored to this value (capped by the gap
/// allowance) so the Z/S bend happens at least
/// `ARROW_HEAD_CLEARANCE_PX` before the path enters the arrow head.
///
/// # Example values
///
/// `11.0` -- 8.0 px arrow head + 3.0 px clearance.
pub const TO_PROTRUSION_MIN_PX: f32 = ARROW_HEAD_LENGTH as f32 + ARROW_HEAD_CLEARANCE_PX;

/// Arc radius in pixels for orthogonal path corners.
///
/// # Example values
///
/// `4.0` -- produces a small visible rounding at each 90-degree turn.
pub const ARC_RADIUS: f32 = 4.0;
