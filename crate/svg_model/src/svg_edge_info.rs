use disposition_ir_model::{edge::EdgeId, node::NodeId};
use disposition_model_common::edge::EdgeGroupId;
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
    /// ID of the edge group this edge belongs to.
    pub edge_group_id: EdgeGroupId<'id>,
    /// The source node ID where this edge originates.
    pub from_node_id: NodeId<'id>,
    /// The target node ID where this edge points to.
    pub to_node_id: NodeId<'id>,
    /// The SVG path `d` attribute for rendering the edge curve.
    pub path_d: String,
}

impl<'id> SvgEdgeInfo<'id> {
    /// Creates a new `SvgEdgeInfo`.
    pub fn new(
        edge_id: EdgeId<'id>,
        edge_group_id: EdgeGroupId<'id>,
        from_node_id: NodeId<'id>,
        to_node_id: NodeId<'id>,
        path_d: String,
    ) -> Self {
        Self {
            edge_id,
            edge_group_id,
            from_node_id,
            to_node_id,
            path_d,
        }
    }
}
