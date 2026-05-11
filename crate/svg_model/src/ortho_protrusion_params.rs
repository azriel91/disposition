use serde::{Deserialize, Serialize};

use crate::SpacerProtrusionParams;

/// Protrusion lengths for the from-node and to-node endpoints of an
/// orthogonal edge path, plus per-spacer protrusion depths.
///
/// A protrusion is a short stub that exits the node face perpendicular
/// to the face line before the main orthogonal routing begins. This
/// separates parallel edges that share the same node face.
///
/// Spacer protrusions serve the same purpose at intermediate spacer
/// boundaries: they extend the path past the spacer so that the
/// routing leg between spacers does not run along a node face, and
/// multiple edges crossing the same inter-rank gap use distinct
/// depths.
///
/// # Example values
///
/// ```rust,ignore
/// OrthoProtrusionParams {
///     from_protrusion: 12.0,
///     to_protrusion: 8.0,
///     spacer_protrusions: vec![
///         SpacerProtrusionParams { entry_protrusion: 12.0, exit_protrusion: 5.0 },
///     ],
/// }
/// ```
///
/// An edge whose from-node is close to the face midpoint gets a longer
/// `from_protrusion`; an edge further from the midpoint gets a shorter
/// one. Each spacer's entry and exit protrusions are computed
/// independently based on the edges sharing that specific rank gap.
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct OrthoProtrusionParams {
    /// Protrusion length in pixels at the from-node endpoint.
    ///
    /// `0.0` means no protrusion (the path routes directly from the
    /// contact point).
    pub from_protrusion: f32,

    /// Protrusion length in pixels at the to-node endpoint.
    ///
    /// `0.0` means no protrusion.
    pub to_protrusion: f32,

    /// Per-spacer protrusion depths, indexed in the same order as the
    /// `spacers` slice passed to `build_spacer_edge_path`.
    ///
    /// When the edge has no spacers, this is empty.
    pub spacer_protrusions: Vec<SpacerProtrusionParams>,
}
