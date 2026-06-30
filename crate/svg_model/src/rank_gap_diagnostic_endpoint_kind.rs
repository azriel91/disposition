use serde::{Deserialize, Serialize};

/// Which endpoint or spacer side a rank-gap diagnostic entry represents.
///
/// Diagnostic mirror of the internal `RankGapEndpointKind` used by the
/// orthogonal protrusion calculator. It identifies which field of the
/// edge's `OrthoProtrusionParams` the entry's computed protrusion depth
/// is written to.
///
/// # Example values
///
/// ```rust,ignore
/// RankGapDiagnosticEndpointKind::FromEndpoint
/// RankGapDiagnosticEndpointKind::SpacerEntry { spacer_index: 0 }
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RankGapDiagnosticEndpointKind {
    /// The "from" node endpoint.
    FromEndpoint,
    /// The "to" node endpoint.
    ToEndpoint,
    /// The entry side of a spacer at the given index (0-based, in the
    /// same order as the edge's `spacer_protrusions`).
    SpacerEntry {
        /// Index into `spacer_protrusions`.
        spacer_index: usize,
    },
    /// The exit side of a spacer at the given index.
    SpacerExit {
        /// Index into `spacer_protrusions`.
        spacer_index: usize,
    },
}
