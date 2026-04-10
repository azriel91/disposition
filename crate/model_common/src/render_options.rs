use serde::{Deserialize, Serialize};

use crate::{edge::EdgeCurvature, RankDir};

/// Options that control how the diagram is rendered.
///
/// # Examples
///
/// ```rust
/// use disposition_model_common::RenderOptions;
///
/// let render_options = RenderOptions::default();
/// assert_eq!(render_options.edge_curvature, Default::default());
/// assert_eq!(render_options.rank_dir, Default::default());
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct RenderOptions {
    /// Controls how edge paths are drawn between nodes.
    ///
    /// * `EdgeCurvature::Curved`: edges use smooth bezier curves.
    /// * `EdgeCurvature::Orthogonal`: edges use orthogonal 90-degree lines.
    #[serde(default, skip_serializing_if = "EdgeCurvature::is_default")]
    pub edge_curvature: EdgeCurvature,

    /// Direction of edges in the diagram.
    ///
    /// * `RankDir::LeftToRight`: edges connect nodes from left to right.
    /// * `RankDir::RightToLeft`: edges connect nodes from right to left.
    /// * `RankDir::TopToBottom`: edges connect nodes from top to bottom.
    /// * `RankDir::BottomToTop`: edges connect nodes from bottom to top.
    #[serde(default, skip_serializing_if = "RankDir::is_default")]
    pub rank_dir: RankDir,
}

impl RenderOptions {
    /// Returns `true` if all fields are at their default values.
    pub fn is_default(&self) -> bool {
        self.edge_curvature.is_default() && self.rank_dir.is_default()
    }
}
