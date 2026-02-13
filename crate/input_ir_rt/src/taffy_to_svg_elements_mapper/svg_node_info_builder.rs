use disposition_ir_model::{node::NodeId, IrDiagram};
use disposition_model_common::{entity::EntityType, Map};
use disposition_svg_model::{SvgNodeInfo, SvgProcessInfo, SvgTextSpan};
use disposition_taffy_model::NodeContext;
use taffy::TaffyTree;

use disposition_ir_model::entity::EntityTailwindClasses;

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
        taffy_node_id: taffy::NodeId,
        taffy_node_layout: &taffy::Layout,
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
        } = svg_node_info_build_context;

        let is_process = ir_diagram
            .entity_types
            .get(node_id.as_ref())
            .map(|types| types.contains(&EntityType::ProcessDefault))
            .unwrap_or(false);

        let (x, y) =
            Self::node_absolute_xy_coordinates(taffy_tree, taffy_node_id, taffy_node_layout);
        let process_id = Self::find_process_id(node_id, ir_diagram, svg_process_infos);

        let width = taffy_node_layout.size.width;
        let height_expanded = taffy_node_layout
            .size
            .height
            .min(taffy_node_layout.content_size.height);
        let height_collapsed = {
            let mut node_height = height_expanded;

            // If this is a process, subtract the height of its process steps.
            if is_process && let Some(proc_info) = svg_process_infos.get(node_id) {
                node_height -= proc_info.total_height;
            }

            node_height
        };
        let height_to_expand_to = if is_process {
            Some(height_expanded)
        } else {
            None
        };
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
                    .map(|span| SvgTextSpan::new(span.x, span.y, Self::escape_xml(&span.text)))
                    .collect()
            })
            .unwrap_or_default();

        SvgNodeInfo::new(
            node_id.clone(),
            tab_index,
            x,
            y,
            width,
            height_collapsed,
            path_d_collapsed,
            process_id,
            text_spans,
        )
    }

    /// Calculates the absolute x and y coordinates of a node.
    ///
    /// The coordinates of the taffy node in the Taffy tree are relative to each
    /// node's parent, whereas we need them to be absolute when rendering the
    /// SVG.
    fn node_absolute_xy_coordinates(
        taffy_tree: &TaffyTree<NodeContext>,
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

    /// Escape XML special characters in text content.
    fn escape_xml(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        s.chars().for_each(|c| match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            _ => result.push(c),
        });
        result
    }
}
