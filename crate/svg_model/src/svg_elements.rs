use disposition_ir_model::{entity::EntityTailwindClasses, node::NodeId};
use disposition_model_common::{theme::Css, Map};
use serde::{Deserialize, Serialize};

use crate::{SvgEdgeInfo, SvgEdgeLabelInfo, SvgNodeInfo, SvgProcessInfo};

/// All the necessary information to output SVG nodes and edges and styling.
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
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
    /// Information to render SVG text labels on edge endpoints.
    ///
    /// Each entry holds the label slots (from and to) for one edge. Only edges
    /// with a description and a face assignment produce non-empty slots.
    pub edge_label_infos: Vec<SvgEdgeLabelInfo<'id>>,
    /// Process information indexed by process node ID.
    ///
    /// Used for calculating y-translations when processes expand.
    /// The map preserves insertion order, which corresponds to process order
    /// in the diagram.
    pub svg_process_infos: Map<NodeId<'id>, SvgProcessInfo<'id>>,
    /// Computed Tailwind CSS classes for interactive visibility behaviour.
    ///
    /// These classes control visibility, colors, animations, and interactions
    /// based on the diagram's state.
    pub tailwind_classes: EntityTailwindClasses<'id>,
    /// Additional CSS to place in the SVG's inline `<styles>` section.
    ///
    /// Allows for custom CSS rules such as keyframe animations that
    /// cannot be expressed through Tailwind classes alone.
    pub css: Css,
}

impl<'id> SvgElements<'id> {
    /// Creates a new `SvgElements`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        svg_width: f32,
        svg_height: f32,
        svg_node_infos: Vec<SvgNodeInfo<'id>>,
        svg_edge_infos: Vec<SvgEdgeInfo<'id>>,
        edge_label_infos: Vec<SvgEdgeLabelInfo<'id>>,
        svg_process_infos: Map<NodeId<'id>, SvgProcessInfo<'id>>,
        tailwind_classes: EntityTailwindClasses<'id>,
        css: Css,
    ) -> Self {
        Self {
            svg_width,
            svg_height,
            svg_node_infos,
            svg_edge_infos,
            edge_label_infos,
            svg_process_infos,
            tailwind_classes,
            css,
        }
    }
}
