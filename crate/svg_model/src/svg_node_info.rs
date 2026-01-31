use disposition_ir_model::node::NodeId;
use serde::{Deserialize, Serialize};

/// Information to render SVG elements for a node.
///
/// This includes:
///
/// * Coordinates for the `<g>` element.
/// * Coordinates for the `<path>` element for the node's background.
/// * Tailwind classes to define colours to use.
/// * Tailwind classes to define the `<path>`'s `d` attribute and height.
/// * The node label to place in the `<text>` element.
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct SvgNodeInfo<'id> {
    /// ID of the IR node this `SvgNodeInfo` represents.
    pub node_id: NodeId<'id>,
}
