use serde::{Deserialize, Serialize};

use crate::{SvgEdgeInfo, SvgNodeInfo};

/// All the necessary information to output SVG nodes and edges and styling.
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct SvgElements<'id> {
    /// Information to render SVG elements for a node.
    ///
    /// This includes:
    ///
    /// * Coordinates for the `<g>` element.
    /// * Coordinates for the `<path>` element for the node's background.
    /// * Tailwind classes to define colours to use.
    /// * Tailwind classes to define the `<path>`'s `d` attribute and height.
    /// * The node label to place in the `<text>` element.
    pub svg_node_infos: Vec<SvgNodeInfo<'id>>,
    /// Information to render SVG elements for edges.
    ///
    /// This includes:
    ///
    /// * The `<path>` element's coordinates and its `d` attribute.
    /// * Tailwind classes to define its styling and visibility.
    pub svg_edge_infos: Vec<SvgEdgeInfo<'id>>,
}
