use serde::{Deserialize, Serialize};

use crate::{edge::EdgeCurvature, RankDir};

pub use self::{
    interaction_edge_halo::InteractionEdgeHalo, process_render_collapse::ProcessRenderCollapse,
};

mod interaction_edge_halo;
mod process_render_collapse;

/// Options that control how the diagram is rendered.
///
/// # Examples
///
/// ```rust
/// use disposition_model_common::RenderOptions;
///
/// use disposition_model_common::edge::EdgeCurvature;
///
/// let render_options = RenderOptions::default();
/// assert_eq!(render_options.rank_dir, Default::default());
/// assert_eq!(render_options.process_render_collapse, Default::default());
/// assert_eq!(
///     render_options.dependency_edge_curvature,
///     EdgeCurvature::Orthogonal
/// );
/// assert_eq!(
///     render_options.interaction_edge_curvature,
///     EdgeCurvature::DirectCurved
/// );
/// assert_eq!(render_options.interaction_edge_halo, Default::default());
/// assert_eq!(render_options.interaction_edge_animation_millis_per_px, 3.0);
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct RenderOptions {
    /// Direction of edges in the diagram.
    ///
    /// * `RankDir::LeftToRight`: edges connect nodes from left to right.
    /// * `RankDir::RightToLeft`: edges connect nodes from right to left.
    /// * `RankDir::TopToBottom`: edges connect nodes from top to bottom.
    /// * `RankDir::BottomToTop`: edges connect nodes from bottom to top.
    #[serde(default, skip_serializing_if = "RankDir::is_default")]
    pub rank_dir: RankDir,

    /// Controls whether processes are rendered collapsed or expanded.
    ///
    /// * `ProcessRenderCollapse::Collapse`: processes are rendered collapsed,
    ///   expanding only when focused.
    /// * `ProcessRenderCollapse::ExpandAlways`: processes are always rendered
    ///   fully expanded.
    /// * `ProcessRenderCollapse::ExpandWhenOne`: processes are rendered
    ///   expanded when there is only a single process in the diagram, and
    ///   collapsed otherwise.
    #[serde(default, skip_serializing_if = "ProcessRenderCollapse::is_default")]
    pub process_render_collapse: ProcessRenderCollapse,

    /// Controls how dependency edge paths are drawn between nodes.
    ///
    /// * `EdgeCurvature::Curved`: edges use smooth bezier curves.
    /// * `EdgeCurvature::Orthogonal`: edges use orthogonal 90-degree lines.
    /// * `EdgeCurvature::DirectStraight`: edges are straight lines that bypass
    ///   edge spacers.
    /// * `EdgeCurvature::DirectCurved`: edges are curved lines that bypass edge
    ///   spacers.
    #[serde(default, skip_serializing_if = "EdgeCurvature::is_default")]
    pub dependency_edge_curvature: EdgeCurvature,

    /// Controls how interaction edge paths are drawn between nodes.
    ///
    /// Defaults to `EdgeCurvature::DirectCurved`.
    ///
    /// * `EdgeCurvature::Curved`: edges use smooth bezier curves.
    /// * `EdgeCurvature::Orthogonal`: edges use orthogonal 90-degree lines.
    /// * `EdgeCurvature::DirectStraight`: edges are straight lines that bypass
    ///   edge spacers.
    /// * `EdgeCurvature::DirectCurved`: edges are curved lines that bypass edge
    ///   spacers.
    #[serde(
        default = "interaction_edge_curvature_default",
        skip_serializing_if = "interaction_edge_curvature_is_default"
    )]
    pub interaction_edge_curvature: EdgeCurvature,

    /// Controls whether a semi-transparent halo is rendered behind
    /// interaction edges.
    ///
    /// Defaults to `InteractionEdgeHalo::Enabled`.
    ///
    /// * `InteractionEdgeHalo::Enabled`: a halo is rendered behind each
    ///   interaction edge, sharing its path geometry.
    /// * `InteractionEdgeHalo::Disabled`: no halo is rendered.
    #[serde(default, skip_serializing_if = "InteractionEdgeHalo::is_default")]
    pub interaction_edge_halo: InteractionEdgeHalo,

    /// Milliseconds of CSS animation duration per pixel of interaction-edge
    /// travel distance, controlling how fast interaction edges animate.
    ///
    /// Also used inversely to convert the end-of-cycle pause duration into
    /// an equivalent pixel distance, so the same value governs both.
    ///
    /// Example valid value: `3.0` (3 milliseconds per pixel -- the default).
    #[serde(
        default = "interaction_edge_animation_millis_per_px_default",
        skip_serializing_if = "interaction_edge_animation_millis_per_px_is_default"
    )]
    pub interaction_edge_animation_millis_per_px: f64,
}

impl RenderOptions {
    /// Returns `true` if all fields are at their default values.
    pub fn is_default(&self) -> bool {
        self.rank_dir.is_default()
            && self.process_render_collapse.is_default()
            && self.dependency_edge_curvature.is_default()
            && interaction_edge_curvature_is_default(&self.interaction_edge_curvature)
            && self.interaction_edge_halo.is_default()
            && interaction_edge_animation_millis_per_px_is_default(
                &self.interaction_edge_animation_millis_per_px,
            )
    }
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            rank_dir: RankDir::default(),
            process_render_collapse: ProcessRenderCollapse::default(),
            dependency_edge_curvature: EdgeCurvature::default(),
            interaction_edge_curvature: interaction_edge_curvature_default(),
            interaction_edge_halo: InteractionEdgeHalo::default(),
            interaction_edge_animation_millis_per_px:
                interaction_edge_animation_millis_per_px_default(),
        }
    }
}

/// Returns the default curvature for interaction edges: `DirectCurved`.
fn interaction_edge_curvature_default() -> EdgeCurvature {
    EdgeCurvature::DirectCurved
}

/// Returns `true` if the interaction edge curvature is the default
/// (`DirectCurved`).
fn interaction_edge_curvature_is_default(edge_curvature: &EdgeCurvature) -> bool {
    *edge_curvature == EdgeCurvature::DirectCurved
}

/// Returns the default `interaction_edge_animation_millis_per_px`: `3.0`
/// milliseconds per pixel (0.3 seconds per 100 pixels).
fn interaction_edge_animation_millis_per_px_default() -> f64 {
    3.0
}

/// Returns `true` if `interaction_edge_animation_millis_per_px` is at its
/// default value.
fn interaction_edge_animation_millis_per_px_is_default(millis_per_px: &f64) -> bool {
    *millis_per_px == interaction_edge_animation_millis_per_px_default()
}
