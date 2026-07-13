use serde::{Deserialize, Serialize};

/// Resolved rendering values for the interaction edge halo, derived from the
/// theme at mapping time.
///
/// All fields default to `0.0` when not resolved (e.g. an `IrDiagram` built
/// directly rather than through `InputToIrDiagramMapper`).
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct InteractionEdgeHaloOptions {
    /// Resolved stroke width (pixels) of the interaction edge halo, from
    /// `ThemeAttr::StrokeWidth` on `type_interaction_edge_halo`.
    ///
    /// Used to size the halo's outline rails proportionally to the halo's
    /// own width, rather than a value hardcoded independently of the theme.
    #[serde(default)]
    pub stroke_width: f32,

    /// Resolved halo fill opacity, as a `0.0..=1.0` fraction (not the 0-100
    /// percent scale used in the theme string), from `ThemeAttr::Opacity` on
    /// `type_interaction_edge_halo`.
    ///
    /// Used as the base opacity that the halo's animated active/inactive
    /// keyframe opacities (see `EdgeAnimationCalculator`) scale, so a themed
    /// halo opacity is respected rather than a value hardcoded independently
    /// of the theme.
    #[serde(default)]
    pub opacity: f32,

    /// Resolved halo outline opacity, as a `0.0..=1.0` fraction, from
    /// `ThemeAttr::Opacity` on `type_interaction_edge_halo_outline`.
    ///
    /// Used the same way as `opacity`, but for the outline rails drawn on
    /// top of the halo fill.
    #[serde(default)]
    pub outline_opacity: f32,
}
