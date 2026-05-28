use disposition_ir_model::{node::NodeId, IrDiagram};
use disposition_model_common::Map;
use disposition_svg_model::SvgProcessInfo;
use disposition_taffy_model::{EntityHighlightedSpans, MdImageSpan, TaffyNodeCtx};
use taffy::TaffyTree;

use disposition_ir_model::node::NodeShape;

use super::ProcessStepsHeight;

#[derive(Clone, Copy, Debug)]
pub(super) struct SvgProcessInfoBuildContext<'ctx, 'id> {
    /// Diagram intermediate representation with nodes, edges, layout, and
    /// tailwind classes.
    pub(super) ir_diagram: &'ctx IrDiagram<'id>,
    /// Holds the computed layout information for each node.
    pub(super) taffy_tree: &'ctx TaffyTree<TaffyNodeCtx>,
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
    pub(super) taffy_tree: &'ctx TaffyTree<TaffyNodeCtx>,
    /// Holds the spans of text for each node.
    pub(super) entity_highlighted_spans: &'ctx EntityHighlightedSpans<'id>,
    /// Inline image spans for markdown nodes. Absent for nodes without inline
    /// images.
    pub(super) entity_image_spans: &'ctx Map<NodeId<'id>, Vec<MdImageSpan>>,
    /// Default shape to when rendering a node.
    pub(super) default_shape: &'ctx NodeShape,
    /// Heights of all process steps for each process.
    pub(super) process_steps_heights: &'ctx [ProcessStepsHeight<'id>],
    /// Map of process ID to SVG process info
    pub(super) svg_process_infos: &'ctx Map<NodeId<'id>, SvgProcessInfo<'id>>,
    /// Map from diagram node ID to its envelope taffy node ID.
    ///
    /// Used to compute absolute envelope bounds for each node so that edge
    /// face contact points land on the outer envelope boundary.
    pub(super) node_id_to_envelope_taffy_node: &'ctx Map<NodeId<'id>, taffy::NodeId>,
}
