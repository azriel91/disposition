use disposition_ir_model::{node::NodeId, IrDiagram};
use disposition_model_common::Map;
use disposition_svg_model::SvgProcessInfo;
use disposition_taffy_model::{EntityHighlightedSpans, NodeContext};
use taffy::TaffyTree;

use disposition_ir_model::node::NodeShape;

use super::ProcessStepsHeight;

#[derive(Clone, Copy, Debug)]
pub(super) struct SvgProcessInfoBuildContext<'ctx, 'id> {
    /// Diagram intermediate representation with nodes, edges, layout, and
    /// tailwind classes.
    pub(super) ir_diagram: &'ctx IrDiagram<'id>,
    /// Holds the computed layout information for each node.
    pub(super) taffy_tree: &'ctx TaffyTree<NodeContext>,
    /// Default shape to when rendering a node.
    pub(super) default_shape: &'ctx NodeShape,
    /// Heights of all process steps for each process.
    pub(super) process_steps_heights: &'ctx [ProcessStepsHeight<'id>],
}

#[derive(Clone, Copy, Debug)]
pub(super) struct SvgNodeInfoBuildContext<'ctx, 'id> {
    /// Diagram intermediate representation with nodes, edges, layout, and
    /// tailwind classes.
    pub(super) ir_diagram: &'ctx IrDiagram<'id>,
    /// Holds the computed layout information for each node.
    pub(super) taffy_tree: &'ctx TaffyTree<NodeContext>,
    /// Holds the spans of text for each node.
    pub(super) entity_highlighted_spans: &'ctx EntityHighlightedSpans<'id>,
    /// Default shape to when rendering a node.
    pub(super) default_shape: &'ctx NodeShape,
    /// Heights of all process steps for each process.
    pub(super) process_steps_heights: &'ctx [ProcessStepsHeight<'id>],
    /// Map of process ID to SVG process info
    pub(super) svg_process_infos: &'ctx Map<NodeId<'id>, SvgProcessInfo<'id>>,
}
