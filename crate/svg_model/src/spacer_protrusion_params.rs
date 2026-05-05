use serde::{Deserialize, Serialize};

/// Protrusion lengths for the entry and exit sides of a single spacer.
///
/// The entry-side protrusion extends the path past the spacer's entry
/// boundary (away from the spacer, into the gap before it). The
/// exit-side protrusion extends the path past the spacer's exit
/// boundary (away from the spacer, into the gap after it).
///
/// Protrusion depths are assigned by `OrthoProtrusionCalculator` so
/// that edges sharing the same inter-rank gap use distinct depths.
///
/// # Example values
///
/// ```rust,ignore
/// SpacerProtrusionParams { entry_protrusion: 5.0, exit_protrusion: 8.0 }
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct SpacerProtrusionParams {
    /// Protrusion length in pixels on the entry side of the spacer.
    ///
    /// `0.0` means no protrusion on the entry side.
    pub entry_protrusion: f32,

    /// Protrusion length in pixels on the exit side of the spacer.
    ///
    /// `0.0` means no protrusion on the exit side.
    pub exit_protrusion: f32,
}
