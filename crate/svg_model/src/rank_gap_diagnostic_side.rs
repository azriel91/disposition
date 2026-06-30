use serde::{Deserialize, Serialize};

/// Which physical boundary of a rank gap an endpoint protrudes from.
///
/// Diagnostic mirror of the internal `GapSide` used by the orthogonal
/// protrusion calculator. Entries on the `Low` side protrude from the
/// `rank_low` boundary of the gap; entries on the `High` side protrude
/// from the `rank_high` boundary.
///
/// # Example values
///
/// Valid values: `Low`, `High`
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RankGapDiagnosticSide {
    /// Protrudes from the `rank_low` boundary of the gap.
    Low,
    /// Protrudes from the `rank_high` boundary of the gap.
    High,
}
