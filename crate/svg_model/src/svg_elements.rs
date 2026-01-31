use serde::{Deserialize, Serialize};

use crate::{SvgEdgeInfo, SvgNodeInfo, SvgProcessInfo};

/// All the necessary information to output SVG nodes and edges and styling.
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct SvgElements<'id> {
    /// Width of the SVG canvas.
    pub svg_width: f32,
    /// Height of the SVG canvas.
    pub svg_height: f32,
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
    /// Process information for all processes in the diagram.
    ///
    /// Used for calculating y-translations when processes expand.
    /// Stored separately so nodes can reference previous processes' heights.
    pub process_infos: Vec<SvgProcessInfo<'id>>,
    /// Additional tailwind classes generated during element mapping.
    ///
    /// These are the translate classes and other dynamically generated classes
    /// that need to be included in CSS generation.
    pub additional_tailwind_classes: Vec<String>,
}

impl<'id> SvgElements<'id> {
    /// Creates a new `SvgElements`.
    pub fn new(
        svg_width: f32,
        svg_height: f32,
        svg_node_infos: Vec<SvgNodeInfo<'id>>,
        svg_edge_infos: Vec<SvgEdgeInfo<'id>>,
        process_infos: Vec<SvgProcessInfo<'id>>,
        additional_tailwind_classes: Vec<String>,
    ) -> Self {
        Self {
            svg_width,
            svg_height,
            svg_node_infos,
            svg_edge_infos,
            process_infos,
            additional_tailwind_classes,
        }
    }
}
