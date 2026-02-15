use disposition_input_ir_model::EdgeAnimationActive;
use disposition_ir_model::{
    node::{NodeId, NodeInbuilt, NodeShape, NodeShapeRect},
    IrDiagram,
};
use disposition_model_common::Map;
use disposition_svg_model::{SvgElements, SvgProcessInfo};
use disposition_taffy_model::TaffyNodeMappings;

use self::{
    arrow_head_builder::ArrowHeadBuilder,
    edge_animation_calculator::EdgeAnimationCalculator,
    edge_path_builder::EdgePathBuilder,
    process_step_heights::ProcessStepsHeight,
    process_step_heights_calculator::ProcessStepHeightsCalculator,
    string_char_replacer::StringCharReplacer,
    svg_edge_infos_builder::SvgEdgeInfosBuilder,
    svg_node_build_context::{SvgNodeInfoBuildContext, SvgProcessInfoBuildContext},
    svg_node_info_builder::SvgNodeInfoBuilder,
    svg_node_rect_path_builder::SvgNodeRectPathBuilder,
    svg_node_translate_classes_builder::SvgNodeTranslateClassesBuilder,
    svg_process_info_builder::SvgProcessInfoBuilder,
};

mod arrow_head_builder;
mod edge_animation_calculator;
mod edge_model;
mod edge_path_builder;
mod process_step_heights;
mod process_step_heights_calculator;
mod string_char_replacer;
mod svg_edge_infos_builder;
mod svg_node_build_context;
mod svg_node_info_builder;
mod svg_node_rect_path_builder;
mod svg_node_translate_classes_builder;
mod svg_process_info_builder;

/// Maps the IR diagram and `TaffyNodeMappings` to SVG elements.
///
/// These include nodes with simple coordinates and edges.
#[derive(Clone, Copy, Debug)]
pub struct TaffyToSvgElementsMapper;

impl TaffyToSvgElementsMapper {
    pub fn map<'id>(
        ir_diagram: &IrDiagram<'id>,
        taffy_node_mappings: &TaffyNodeMappings<'id>,
        edge_animation_active: EdgeAnimationActive,
    ) -> SvgElements<'id> {
        let TaffyNodeMappings {
            taffy_tree,
            node_inbuilt_to_taffy,
            node_id_to_taffy,
            taffy_id_to_node: _,
            entity_highlighted_spans,
        } = taffy_node_mappings;

        // Get root layout for SVG dimensions
        let root_taffy_node_id = node_inbuilt_to_taffy
            .get(&NodeInbuilt::Root)
            .copied()
            .expect("Expected root taffy node to exist.");
        let root_layout = taffy_tree
            .layout(root_taffy_node_id)
            .expect("Expected root layout to exist.");
        let svg_width = root_layout.size.width;
        let svg_height = root_layout.size.height;

        // Default shape for nodes without explicit shape configuration
        let default_shape = NodeShape::Rect(NodeShapeRect::new());

        // First, collect process information for y-coordinate calculations
        let process_steps_heights =
            ProcessStepHeightsCalculator::calculate(ir_diagram, taffy_tree, node_id_to_taffy);

        // Build process_infos map from process_steps_heights
        // We need to compute the actual values for each process node
        let svg_process_info_build_context = SvgProcessInfoBuildContext {
            ir_diagram,
            taffy_tree,
            default_shape: &default_shape,
            process_steps_heights: &process_steps_heights,
        };
        let svg_process_infos = process_steps_heights.iter().enumerate().fold(
            Map::<NodeId<'id>, SvgProcessInfo<'id>>::new(),
            |mut svg_process_infos, (process_idx, process_steps_height)| {
                let process_node_id = &process_steps_height.process_id;

                // Look up taffy layout for the process node
                if let Some(taffy_node_ids) = node_id_to_taffy.get(process_node_id).copied() {
                    let taffy_node_id = taffy_node_ids.wrapper_taffy_node_id();
                    if let Ok(layout) = taffy_tree.layout(taffy_node_id) {
                        let svg_process_info = SvgProcessInfoBuilder::build(
                            svg_process_info_build_context,
                            process_idx,
                            process_steps_height,
                            process_node_id,
                            taffy_node_id,
                            layout,
                        );

                        svg_process_infos
                            .insert(process_steps_height.process_id.clone(), svg_process_info);
                    }
                }

                svg_process_infos
            },
        );

        // Clone `tailwind_classes` from `ir_diagram`, and append to each entity
        // additional tailwind classes e.g. for translating process nodes when
        // collapsing / expanding them.
        let tailwind_classes = ir_diagram.tailwind_classes.clone();

        // Build an `SvgNodeInfo` for each node in the order specified by
        // `node_ordering`.
        let svg_node_info_build_context = SvgNodeInfoBuildContext {
            ir_diagram,
            taffy_tree,
            entity_highlighted_spans,
            default_shape: &default_shape,
            process_steps_heights: &process_steps_heights,
            svg_process_infos: &svg_process_infos,
        };
        let (svg_node_infos, mut tailwind_classes) = ir_diagram.node_ordering.iter().fold(
            (Vec::new(), tailwind_classes),
            |(mut svg_node_infos, mut entity_tailwind_classes), (node_id, &tab_index)| {
                if let Some(taffy_node_ids) = node_id_to_taffy.get(node_id).copied() {
                    let taffy_node_id = taffy_node_ids.wrapper_taffy_node_id();

                    if let Ok(taffy_node_layout) = taffy_tree.layout(taffy_node_id) {
                        let svg_node_info = SvgNodeInfoBuilder::build(
                            svg_node_info_build_context,
                            taffy_node_id,
                            taffy_node_layout,
                            &mut entity_tailwind_classes,
                            node_id,
                            tab_index,
                        );

                        svg_node_infos.push(svg_node_info);
                    }
                }

                (svg_node_infos, entity_tailwind_classes)
            },
        );

        // Build a lookup map from NodeId to SvgNodeInfo for edge building
        let svg_node_info_map: Map<&NodeId<'id>, &_> = svg_node_infos
            .iter()
            .map(|info| (&info.node_id, info))
            .collect();

        // Clone css from ir_diagram; edge animation CSS will be appended.
        let mut css = ir_diagram.css.clone();

        // Build edge information and compute animation data for interaction
        // edges.
        let svg_edge_infos = SvgEdgeInfosBuilder::build(
            &ir_diagram.edge_groups,
            &ir_diagram.entity_types,
            &svg_node_info_map,
            &mut tailwind_classes,
            &mut css,
            edge_animation_active,
            &ir_diagram.process_step_entities,
        );

        SvgElements::new(
            svg_width,
            svg_height,
            svg_node_infos,
            svg_edge_infos,
            svg_process_infos,
            tailwind_classes,
            css,
        )
    }
}
