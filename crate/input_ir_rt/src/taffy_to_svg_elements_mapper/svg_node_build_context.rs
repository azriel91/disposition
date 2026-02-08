use disposition_ir_model::{node::NodeId, IrDiagram};
use disposition_model_common::Map;
use disposition_svg_model::SvgProcessInfo;
use disposition_taffy_model::{EntityHighlightedSpans, NodeContext};
use taffy::TaffyTree;

use disposition_ir_model::node::NodeShape;

use super::ProcessStepsHeight;

#[derive(Clone, Copy, Debug)]
pub(super) struct SvgProcessInfoBuildContext<'ctx, 'id> {
    pub(super) ir_diagram: &'ctx IrDiagram<'id>,
    pub(super) taffy_tree: &'ctx TaffyTree<NodeContext>,
    pub(super) default_shape: &'ctx NodeShape,
    pub(super) process_steps_heights: &'ctx [ProcessStepsHeight<'id>],
}

#[derive(Clone, Copy, Debug)]
pub(super) struct SvgNodeInfoBuildContext<'ctx, 'id> {
    pub(super) ir_diagram: &'ctx IrDiagram<'id>,
    pub(super) taffy_tree: &'ctx TaffyTree<NodeContext>,
    pub(super) entity_highlighted_spans: &'ctx EntityHighlightedSpans<'id>,
    pub(super) default_shape: &'ctx NodeShape,
    pub(super) process_steps_heights: &'ctx [ProcessStepsHeight<'id>],
    pub(super) svg_process_infos: &'ctx Map<NodeId<'id>, SvgProcessInfo<'id>>,
}
