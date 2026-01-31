use disposition_ir_model::edge::EdgeId;
use serde::{Deserialize, Serialize};

/// Information to render SVG elements for edges.
///
/// This includes:
///
/// * The `<path>` element's coordinates and its `d` attribute.
/// * Tailwind classes to define its styling and visibility.
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct SvgEdgeInfo<'id> {
    /// ID of the edge this `SvgEdgeInfo` represents.
    pub edge_id: EdgeId<'id>,
}
