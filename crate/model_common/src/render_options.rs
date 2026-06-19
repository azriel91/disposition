use serde::{Deserialize, Serialize};

use crate::{edge::EdgeCurvature, RankDir};

pub use self::process_render_collapse::ProcessRenderCollapse;

mod process_render_collapse;

/// Options that control how the diagram is rendered.
///
/// # Examples
///
/// ```rust
/// use disposition_model_common::RenderOptions;
///
/// let render_options = RenderOptions::default();
/// assert_eq!(
///     render_options.dependencies_edge_curvature,
///     Default::default()
/// );
/// assert_eq!(render_options.rank_dir, Default::default());
/// assert_eq!(render_options.process_render_collapse, Default::default());
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct RenderOptions {
    /// Controls how dependency edge paths are drawn between nodes.
    ///
    /// * `EdgeCurvature::Curved`: edges use smooth bezier curves.
    /// * `EdgeCurvature::Orthogonal`: edges use orthogonal 90-degree lines.
    /// * `EdgeCurvature::DirectStraight`: edges are straight lines that bypass
    ///   edge spacers.
    /// * `EdgeCurvature::DirectCurved`: edges are curved lines that bypass edge
    ///   spacers.
    #[serde(default, skip_serializing_if = "EdgeCurvature::is_default")]
    pub dependencies_edge_curvature: EdgeCurvature,

    /// Controls how interaction edge paths are drawn between nodes.
    ///
    /// * `EdgeCurvature::Curved`: edges use smooth bezier curves.
    /// * `EdgeCurvature::Orthogonal`: edges use orthogonal 90-degree lines.
    /// * `EdgeCurvature::DirectStraight`: edges are straight lines that bypass
    ///   edge spacers.
    /// * `EdgeCurvature::DirectCurved`: edges are curved lines that bypass edge
    ///   spacers.
    #[serde(default, skip_serializing_if = "EdgeCurvature::is_default")]
    pub interactions_edge_curvature: EdgeCurvature,

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
}

impl RenderOptions {
    /// Returns `true` if all fields are at their default values.
    pub fn is_default(&self) -> bool {
        self.dependencies_edge_curvature.is_default()
            && self.interactions_edge_curvature.is_default()
            && self.rank_dir.is_default()
            && self.process_render_collapse.is_default()
    }
}
