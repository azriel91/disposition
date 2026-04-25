use disposition_ir_model::{edge::EdgeId, node::NodeId};
use disposition_model_common::edge::EdgeGroupId;
use serde::{Deserialize, Serialize};

/// Information to render SVG elements for edges.
///
/// This includes:
///
/// * The `<path>` element's coordinates and its `d` attribute.
/// * Tailwind classes to define its styling and visibility.
/// * The arrowhead `<path>` element's `d` attribute.
/// * The locus `<path>` element's `d` attribute for the focus indicator.
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
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
    /// The SVG path `d` attribute for the arrowhead.
    ///
    /// For dependency edges this is a positioned closed V-shape at the `to`
    /// node end of the edge. For interaction edges this is an
    /// origin-centred V-shape that is animated along the edge path via CSS
    /// `offset-path`.
    pub arrow_head_path_d: String,
    /// The SVG path `d` attribute for the edge locus (focus indicator).
    ///
    /// This is the outline of the stroke expansion around both the edge body
    /// and the arrow head, rendered as a dashed highlight when the edge is
    /// focused. Example value: `"M10,20 C30,40 50,60 70,80"`.
    pub locus_path_d: String,
    /// Tooltip text to display when the edge is hovered.
    ///
    /// When non-empty, rendered as a `<title>` element inside the edge's `<g>`
    /// element. Example value: `"Sends a request to the API server."`.
    pub tooltip: String,
}

impl<'id> SvgEdgeInfo<'id> {
    /// Creates a new `SvgEdgeInfo`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        edge_id: EdgeId<'id>,
        edge_group_id: EdgeGroupId<'id>,
        from_node_id: NodeId<'id>,
        to_node_id: NodeId<'id>,
        path_d: String,
        arrow_head_path_d: String,
        locus_path_d: String,
        tooltip: String,
    ) -> Self {
        Self {
            edge_id,
            edge_group_id,
            from_node_id,
            to_node_id,
            path_d,
            arrow_head_path_d,
            locus_path_d,
            tooltip,
        }
    }
}
