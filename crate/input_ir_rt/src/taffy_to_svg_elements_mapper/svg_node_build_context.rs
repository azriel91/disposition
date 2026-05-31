use disposition_ir_model::IrDiagram;
use disposition_taffy_model::{
    EntityHighlightedSpans, NodeIdToEnvelopeTaffyNode, NodeIdToImageSpans, TaffyNodeCtx,
};
use taffy::TaffyTree;

use disposition_ir_model::node::NodeShape;

use super::{NodeIdToSvgProcessInfo, ProcessStepsHeight};

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
    pub(super) entity_image_spans: &'ctx NodeIdToImageSpans<'id>,
    /// Default shape to when rendering a node.
    pub(super) default_shape: &'ctx NodeShape,
    /// Heights of all process steps for each process.
    pub(super) process_steps_heights: &'ctx [ProcessStepsHeight<'id>],
    /// Map of process ID to SVG process info
    pub(super) svg_process_infos: &'ctx NodeIdToSvgProcessInfo<'id>,
    /// Map from diagram node ID to its envelope taffy node ID.
    ///
    /// Used to compute absolute envelope bounds for each node so that edge
    /// face contact points land on the outer envelope boundary.
    pub(super) node_id_to_envelope_taffy_node: &'ctx NodeIdToEnvelopeTaffyNode<'id>,
    /// Whether processes are rendered fully expanded.
    ///
    /// When `true`, the collapsed-height logic and focus-driven expand
    /// animation classes are not emitted for process nodes.
    pub(super) process_render_expanded: bool,
}
