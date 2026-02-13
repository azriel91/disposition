use disposition_ir_model::node::NodeId;
use serde::{Deserialize, Serialize};

use crate::SvgTextSpan;

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
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct SvgNodeInfo<'id> {
    /// ID of the IR node this `SvgNodeInfo` represents.
    pub node_id: NodeId<'id>,
    /// Tab index for keyboard navigation.
    pub tab_index: u32,
    /// X coordinate (absolute position).
    pub x: f32,
    /// Y coordinate (absolute position, before process collapse adjustments).
    pub y: f32,
    /// Width of the node.
    pub width: f32,
    /// Height of the node in collapsed state.
    pub height_collapsed: f32,
    /// The path `d` attribute for the collapsed state.
    pub path_d_collapsed: String,
    /// Reference to the process this node belongs to (if any).
    ///
    /// For process nodes, this points to themselves.
    /// For process step nodes, this points to their parent process.
    /// For other nodes, this is `None`.
    ///
    /// The full process information can be looked up in
    /// `SvgElements.process_infos` using this ID.
    pub process_id: Option<NodeId<'id>>,
    /// Text spans to render within this node.
    pub text_spans: Vec<SvgTextSpan>,
}

impl<'id> SvgNodeInfo<'id> {
    /// Creates a new `SvgNodeInfo`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        node_id: NodeId<'id>,
        tab_index: u32,
        x: f32,
        y: f32,
        width: f32,
        height_collapsed: f32,
        path_d_collapsed: String,
        process_id: Option<NodeId<'id>>,
        text_spans: Vec<SvgTextSpan>,
    ) -> Self {
        Self {
            node_id,
            tab_index,
            x,
            y,
            width,
            height_collapsed,
            path_d_collapsed,
            process_id,
            text_spans,
        }
    }
}
