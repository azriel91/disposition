use serde::{Deserialize, Serialize};

/// Controls how edge paths are drawn between nodes.
///
/// # Examples
///
/// ```rust
/// use disposition_model_common::edge::EdgeCurvature;
///
/// let curved = EdgeCurvature::Curved;
/// let ortho = EdgeCurvature::Orthogonal;
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub enum EdgeCurvature {
    /// Edges use smooth bezier curves between nodes and spacers.
    #[default]
    Curved,
    /// Edges use orthogonal (90-degree) lines between nodes and spacers.
    ///
    /// Corners where the path changes from horizontal to vertical (or
    /// vice versa) are rounded with a small arc.
    Orthogonal,
}
