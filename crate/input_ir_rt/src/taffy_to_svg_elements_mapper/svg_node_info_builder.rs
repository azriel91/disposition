use crate::string_xml_escaper::StringXmlEscaper;
use disposition_ir_model::{node::NodeId, IrDiagram};
use disposition_model_common::{entity::EntityType, Map};
use disposition_svg_model::{SvgNodeInfo, SvgNodeInfoCircle, SvgProcessInfo, SvgTextSpan};
use disposition_taffy_model::{NodeToTaffyNodeIds, TaffyNodeCtx};
use taffy::TaffyTree;

use disposition_ir_model::{entity::EntityTailwindClasses, node::NodeShape};

use super::{
    svg_node_build_context::SvgNodeInfoBuildContext, SvgNodeRectPathBuilder,
    SvgNodeTranslateClassesBuilder,
};

/// Builds [`SvgNodeInfo`] values from IR nodes and their Taffy layouts.
#[derive(Clone, Copy, Debug)]
pub struct SvgNodeInfoBuilder;

impl SvgNodeInfoBuilder {
    /// Returns the [`SvgNodeInfo`] for the given IR node.
    pub(super) fn build<'ctx, 'id>(
        svg_node_info_build_context: SvgNodeInfoBuildContext<'ctx, 'id>,
        taffy_node_ids: NodeToTaffyNodeIds,
        entity_tailwind_classes: &mut EntityTailwindClasses<'id>,
        node_id: &NodeId<'id>,
        tab_index: u32,
    ) -> SvgNodeInfo<'id> {
        let SvgNodeInfoBuildContext {
            ir_diagram,
            taffy_tree,
            entity_highlighted_spans,
            default_shape,
            process_steps_heights,
            svg_process_infos,
            node_id_to_envelope_taffy_node,
        } = svg_node_info_build_context;

        let is_process = ir_diagram
            .entity_types
            .get(node_id.as_ref())
            .map(|types| types.contains(&EntityType::ProcessDefault))
            .unwrap_or(false);

        let wrapper_taffy_node_id = taffy_node_ids.wrapper_taffy_node_id();
        let wrapper_taffy_node_layout = taffy_tree
            .layout(wrapper_taffy_node_id)
            .unwrap_or_else(|e| panic!("Expected taffy layout to exist for {node_id}. Error: {e}"));

        let (x, y) = Self::node_absolute_xy_coordinates(
            taffy_tree,
            wrapper_taffy_node_id,
            wrapper_taffy_node_layout,
        );
        let process_id = Self::find_process_id(node_id, ir_diagram, svg_process_infos);

        let width = wrapper_taffy_node_layout.size.width;
        let height_expanded = wrapper_taffy_node_layout.size.height;
        let height_collapsed = {
            let mut node_height = height_expanded;

            // If this is a process, subtract the height of its process steps.
            if is_process && let Some(proc_info) = svg_process_infos.get(node_id) {
                node_height -= proc_info.total_height;
            }

            node_height
        };

        // Compute envelope bounds -- the outer taffy node that wraps the
        // diagram node and includes edge label wrapper slots on each face.
        // Fall back to wrapper bounds if no envelope is recorded.
        let (envelope_x, envelope_y, envelope_width, envelope_height_collapsed) =
            Self::node_envelope_bounds(
                taffy_tree,
                node_id,
                node_id_to_envelope_taffy_node,
                x,
                y,
                width,
                height_collapsed,
            );
        let height_to_expand_to = if is_process {
            Some(height_expanded)
        } else {
            None
        };
        let node_rank = ir_diagram
            .node_ranks_nested
            .node_rank_for(node_id, &ir_diagram.node_nesting_infos)
            .unwrap_or_else(|| panic!("node_rank not found for node_id: {:?}", node_id));
        let node_shape = ir_diagram.node_shapes.get(node_id).unwrap_or(default_shape);

        let path_d_collapsed = SvgNodeRectPathBuilder::build(width, height_collapsed, node_shape);
        let translate_classes = SvgNodeTranslateClassesBuilder::build(
            process_steps_heights,
            svg_process_infos,
            x,
            y,
            &process_id,
            width,
            height_expanded,
            height_to_expand_to,
            node_shape,
            &path_d_collapsed,
        );

        if let Some(tailwind_classes) = entity_tailwind_classes
            .get_mut(AsRef::<disposition_model_common::Id<'_>>::as_ref(node_id))
        {
            tailwind_classes.push(' ');
            tailwind_classes.push_str(&translate_classes);
        } else {
            entity_tailwind_classes.insert(node_id.clone().into_inner(), translate_classes);
        }

        let text_spans: Vec<SvgTextSpan> = entity_highlighted_spans
            .get(node_id.as_ref())
            .map(|spans| {
                spans
                    .iter()
                    .map(|span| {
                        SvgTextSpan::new(span.x, span.y, StringXmlEscaper::escape(&span.text))
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Check if this node has a circle shape and compute circle info
        let circle_info = match node_shape {
            NodeShape::Circle(circle_shape) => {
                let radius = circle_shape.radius();

                // Look up the taffy node IDs for this node to find the circle taffy node
                let (circle_abs_x, circle_abs_y) = {
                    let circle_taffy_node_id = taffy_node_ids.circle_taffy_node_id().unwrap_or_else(|| panic!("Expected `circle_taffy_node_id` to exist for {node_id} as it has a `NodeShape::Circle`."));

                    let circle_taffy_node_layout =
                        taffy_tree.layout(circle_taffy_node_id).unwrap_or_else(|e| {
                            panic!("Expected layout to exist for {node_id}. Error: {e}");
                        });

                    Self::node_absolute_xy_coordinates(
                        taffy_tree,
                        circle_taffy_node_id,
                        circle_taffy_node_layout,
                    )
                };

                // Circle center relative to the node's position
                let cx = circle_abs_x - x + radius;
                let cy = circle_abs_y - y + radius;

                let path_d = SvgNodeInfoCircle::build_path_d(cx, cy, radius);

                Some(SvgNodeInfoCircle::new(path_d, cx, cy, radius))
            }
            NodeShape::Rect(_node_shape_rect) => None,
        };

        let tooltip = ir_diagram
            .entity_tooltips
            .get(node_id.as_ref())
            .cloned()
            .unwrap_or_default();

        if let Some(circle) = circle_info {
            SvgNodeInfo::with_circle(
                node_id.clone(),
                node_rank,
                tab_index,
                x,
                y,
                width,
                height_collapsed,
                envelope_x,
                envelope_y,
                envelope_width,
                envelope_height_collapsed,
                path_d_collapsed,
                process_id,
                text_spans,
                circle,
                tooltip,
            )
        } else {
            SvgNodeInfo::new(
                node_id.clone(),
                node_rank,
                tab_index,
                x,
                y,
                width,
                height_collapsed,
                envelope_x,
                envelope_y,
                envelope_width,
                envelope_height_collapsed,
                path_d_collapsed,
                process_id,
                text_spans,
                tooltip,
            )
        }
    }

    /// Calculates the absolute x and y coordinates of a node.
    ///
    /// The coordinates of the taffy node in the Taffy tree are relative to each
    /// node's parent, whereas we need them to be absolute when rendering the
    /// SVG.
    pub(super) fn node_absolute_xy_coordinates(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        taffy_node_id: taffy::NodeId,
        layout: &taffy::Layout,
    ) -> (f32, f32) {
        let (x, y) = {
            // We don't use the content_box here because these are coordinates for the
            // `<rect>` element.
            let mut x_acc = layout.location.x;
            let mut y_acc = layout.location.y;
            let mut current_node_id = taffy_node_id;
            while let Some(parent_taffy_node_id) = taffy_tree.parent(current_node_id) {
                let Ok(parent_layout) = taffy_tree.layout(parent_taffy_node_id) else {
                    break;
                };
                // `content_box_x/y` places the inner nodes to align to the bottom right of
                // the parent nodes instead of having appropriate padding around the inner
                // node.
                x_acc += parent_layout.location.x;
                y_acc += parent_layout.location.y;
                current_node_id = parent_taffy_node_id;
            }
            (x_acc, y_acc)
        };
        (x, y)
    }

    /// Returns the absolute envelope bounds for a diagram node.
    ///
    /// Looks up the envelope taffy node from `node_id_to_envelope_taffy_node`
    /// and computes its absolute coordinates using
    /// [`Self::node_absolute_xy_coordinates`]. Falls back to the wrapper node
    /// bounds (`fallback_x/y/width/height_collapsed`) when no envelope is
    /// recorded for the node.
    ///
    /// Returns `(envelope_x, envelope_y, envelope_width,
    /// envelope_height_collapsed)`.
    fn node_envelope_bounds<'id>(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        node_id: &NodeId<'id>,
        node_id_to_envelope_taffy_node: &Map<NodeId<'id>, taffy::NodeId>,
        fallback_x: f32,
        fallback_y: f32,
        fallback_width: f32,
        fallback_height_collapsed: f32,
    ) -> (f32, f32, f32, f32) {
        let Some(&envelope_taffy_node_id) = node_id_to_envelope_taffy_node.get(node_id) else {
            return (
                fallback_x,
                fallback_y,
                fallback_width,
                fallback_height_collapsed,
            );
        };

        let Ok(envelope_layout) = taffy_tree.layout(envelope_taffy_node_id) else {
            return (
                fallback_x,
                fallback_y,
                fallback_width,
                fallback_height_collapsed,
            );
        };

        let (envelope_x, envelope_y) =
            Self::node_absolute_xy_coordinates(taffy_tree, envelope_taffy_node_id, envelope_layout);
        let envelope_width = envelope_layout.size.width;
        let envelope_height_collapsed = envelope_layout
            .size
            .height
            .min(envelope_layout.content_size.height);

        (
            envelope_x,
            envelope_y,
            envelope_width,
            envelope_height_collapsed,
        )
    }

    /// Finds the process ID that a given node belongs to (if any).
    ///
    /// For process nodes, returns the node's own ID.
    /// For process step nodes, returns the parent process's ID.
    /// For other nodes, returns None.
    fn find_process_id<'id>(
        node_id: &NodeId<'id>,
        ir_diagram: &IrDiagram<'id>,
        process_infos: &Map<NodeId<'id>, SvgProcessInfo<'id>>,
    ) -> Option<NodeId<'id>> {
        let entity_types = ir_diagram.entity_types.get(node_id.as_ref());

        let is_process = entity_types
            .map(|types| types.contains(&EntityType::ProcessDefault))
            .unwrap_or(false);

        let is_process_step = entity_types
            .map(|types| types.contains(&EntityType::ProcessStepDefault))
            .unwrap_or(false);

        if is_process {
            // Process nodes reference themselves
            Some(node_id.clone())
        } else if is_process_step {
            // Find which process this step belongs to
            process_infos
                .iter()
                .find_map(|(proc_id, svg_process_info)| {
                    if svg_process_info.process_step_ids.contains(node_id) {
                        Some(proc_id.clone())
                    } else {
                        None
                    }
                })
        } else {
            None
        }
    }
}
