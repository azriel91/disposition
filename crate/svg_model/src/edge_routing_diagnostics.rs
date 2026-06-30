use serde::{Deserialize, Serialize};

use crate::{EdgeRoutingDiagnostic, RankGapEntryDiagnostic};

/// Diagnostic intermediate data produced while routing edges.
///
/// This captures the pass-1, offset, slot-index, rank-gap, and protrusion
/// values the edge router (`OrthoProtrusionCalculator` and
/// `SvgEdgeInfosBuilder`) computes internally and otherwise discards.
///
/// It is a sibling of `SvgElements` -- produced during the same mapping
/// stage but kept separate so the `SvgElements` output stays focused on
/// render data. Nothing in the render pipeline reads it back; it exists to
/// make the edge-routing calculations inspectable (via the CLI's
/// `edge-routing` data kind and the playground's "Edge Routing" tab) for
/// diagnosing and refining the routing algorithm.
///
/// # Example values
///
/// ```rust,ignore
/// EdgeRoutingDiagnostics {
///     edge_entries: vec![/* one EdgeRoutingDiagnostic per edge */],
///     rank_gap_entries: vec![/* one RankGapEntryDiagnostic per gap endpoint */],
/// }
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct EdgeRoutingDiagnostics<'id> {
    /// Per-edge pass-1, offset, and protrusion diagnostics, in edge-group
    /// then edge order.
    pub edge_entries: Vec<EdgeRoutingDiagnostic<'id>>,
    /// Per-endpoint rank-gap entries considered when assigning protrusion
    /// depths, grouped (in `rank_low`, `rank_high` order) by the gap they
    /// belong to.
    pub rank_gap_entries: Vec<RankGapEntryDiagnostic<'id>>,
}

impl<'id> EdgeRoutingDiagnostics<'id> {
    /// Creates a new `EdgeRoutingDiagnostics`.
    pub fn new(
        edge_entries: Vec<EdgeRoutingDiagnostic<'id>>,
        rank_gap_entries: Vec<RankGapEntryDiagnostic<'id>>,
    ) -> Self {
        Self {
            edge_entries,
            rank_gap_entries,
        }
    }
}
