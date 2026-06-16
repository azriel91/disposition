use disposition_input_ir_model::EdgeAnimationActive;
use disposition_ir_model::{
    entity::EntityType,
    node::{NodeInbuilt, NodeShape, NodeShapeRect},
    IrDiagram,
};
use disposition_svg_model::SvgElements;
use disposition_taffy_model::TaffyNodeMappings;

use crate::input_to_ir_diagram_mapper::tailwind_focus_mode::TailwindFocusMode;

use self::{
    arrow_head_builder::ArrowHeadBuilder,
    edge_animation_calculator::EdgeAnimationCalculator,
    edge_path_builder_pass_1::EdgePathBuilderPass1,
    edge_path_builder_pass_2::EdgePathBuilderPass2,
    edge_path_locus_calculator::EdgePathLocusCalculator,
    edge_spacer_coordinates_calculator::EdgeSpacerCoordinatesCalculator,
    node_id_to_svg_process_info::NodeIdToSvgProcessInfo,
    process_step_graph_edges_builder::ProcessStepGraphEdgesBuilder,
    process_step_heights::ProcessStepsHeight,
    process_step_heights_calculator::ProcessStepHeightsCalculator,
    spacer_coordinates_resolver::SpacerCoordinatesResolver,
    string_char_replacer::StringCharReplacer,
    svg_edge_descriptions_builder::SvgEdgeDescriptionsBuilder,
    svg_edge_infos_builder::SvgEdgeInfosBuilder,
    svg_edge_labels_builder::SvgEdgeLabelsBuilder,
    svg_node_build_context::{SvgNodeInfoBuildContext, SvgProcessInfoBuildContext},
    svg_node_info_builder::SvgNodeInfoBuilder,
    svg_node_info_by_node_id::SvgNodeInfoByNodeId,
    svg_node_rect_path_builder::SvgNodeRectPathBuilder,
    svg_node_translate_classes_builder::SvgNodeTranslateClassesBuilder,
    svg_process_info_builder::SvgProcessInfoBuilder,
};

mod arrow_head_builder;
mod edge_animation_calculator;
mod edge_face_contact_tracker;
mod edge_model;
mod edge_path_builder_pass_1;
mod edge_path_builder_pass_2;
mod edge_path_locus_calculator;
mod edge_spacer_coordinates_calculator;
mod node_id_to_svg_process_info;
mod ortho_protrusion_calculator;
mod process_step_graph_edges_builder;
mod process_step_heights;
mod process_step_heights_calculator;
mod spacer_coordinates_resolver;
mod string_char_replacer;
mod svg_edge_descriptions_builder;
mod svg_edge_infos_builder;
mod svg_edge_labels_builder;
mod svg_node_build_context;
mod svg_node_info_builder;
mod svg_node_info_by_node_id;
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
        Self::map_with_focus(
            ir_diagram,
            taffy_node_mappings,
            edge_animation_active,
            TailwindFocusMode::Interactive,
        )
    }

    /// Maps the IR diagram and `TaffyNodeMappings` to SVG elements, baking the
    /// given focus state into the focus-dependent edge animation classes.
    ///
    /// [`Self::map`] is the public entry point, which uses
    /// [`TailwindFocusMode::Interactive`].
    pub(crate) fn map_with_focus<'id>(
        ir_diagram: &IrDiagram<'id>,
        taffy_node_mappings: &TaffyNodeMappings<'id>,
        edge_animation_active: EdgeAnimationActive,
        focus_mode: TailwindFocusMode<'_, 'id>,
    ) -> SvgElements<'id> {
        let TaffyNodeMappings {
            taffy_tree,
            node_inbuilt_to_taffy,
            node_id_to_taffy,
            taffy_id_to_node: _,
            taffy_id_to_kind: _,
            edge_spacer_taffy_nodes,
            entity_highlighted_spans,
            edge_label_taffy_nodes,
            edge_description_taffy_nodes,
            edge_description_highlighted_spans,
            node_id_to_envelope_taffy_node,
            md_node_taffy_ids: _,
            entity_image_spans,
            edge_description_image_spans,
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

        // Determine whether processes should be rendered fully expanded.
        //
        // When expanded, the collapsed-height logic (and the focus-driven
        // expand animation) is not used, so the process step heights and
        // process infos are left empty.
        let process_count = ir_diagram
            .node_hierarchy
            .iter()
            .filter(|(node_id, _children)| {
                ir_diagram
                    .entity_types
                    .get(node_id.as_ref())
                    .is_some_and(|types| types.contains(&EntityType::ProcessDefault))
            })
            .count();
        let process_render_expanded = ir_diagram
            .render_options
            .process_render_collapse
            .process_render_expanded(process_count);

        // When the diagram has no processes, interaction edges are never
        // revealed by focusing a process step, so animating them only
        // `OnProcessStepFocus` would leave them static forever. Force `Always`
        // so they animate continuously (and `TailwindClassesBuilder` makes them
        // visible by default in the same no-process case).
        let edge_animation_active = if process_count == 0 {
            EdgeAnimationActive::Always
        } else {
            edge_animation_active
        };

        // First, collect process information for y-coordinate calculations.
        //
        // Not necessary when processes are rendered expanded.
        let process_steps_heights = if process_render_expanded {
            Vec::new()
        } else {
            ProcessStepHeightsCalculator::calculate(ir_diagram, taffy_tree, node_id_to_taffy)
        };

        // Build process_infos map from process_steps_heights
        // We need to compute the actual values for each process node
        let svg_process_info_build_context = SvgProcessInfoBuildContext {
            ir_diagram,
            taffy_tree,
            default_shape: &default_shape,
            process_steps_heights: &process_steps_heights,
        };
        let svg_process_infos = process_steps_heights.iter().enumerate().fold(
            NodeIdToSvgProcessInfo::<'id>::new(),
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
            entity_image_spans,
            default_shape: &default_shape,
            process_steps_heights: &process_steps_heights,
            svg_process_infos: &svg_process_infos,
            node_id_to_envelope_taffy_node,
            process_render_expanded,
            focus_mode,
        };
        let (svg_node_infos, mut tailwind_classes) = ir_diagram.node_ordering.iter().fold(
            (Vec::new(), tailwind_classes),
            |(mut svg_node_infos, mut entity_tailwind_classes), (node_id, &tab_index)| {
                if let Some(taffy_node_ids) = node_id_to_taffy.get(node_id).copied() {
                    let svg_node_info = SvgNodeInfoBuilder::build(
                        svg_node_info_build_context,
                        taffy_node_ids,
                        &mut entity_tailwind_classes,
                        node_id,
                        tab_index,
                    );

                    svg_node_infos.push(svg_node_info);
                }

                (svg_node_infos, entity_tailwind_classes)
            },
        );

        // Build a lookup map from NodeId to SvgNodeInfo for edge building
        let svg_node_info_map: SvgNodeInfoByNodeId<'_, 'id> = svg_node_infos
            .iter()
            .map(|info| (&info.node_id, info))
            .collect();

        // Clone css from ir_diagram; edge animation CSS will be appended.
        let mut css = ir_diagram.css.clone();

        // Build edge information and compute animation data for interaction
        // edges.
        let mut svg_edge_infos = SvgEdgeInfosBuilder::build(
            ir_diagram,
            &svg_node_info_map,
            taffy_tree,
            edge_spacer_taffy_nodes,
            edge_label_taffy_nodes,
            &mut tailwind_classes,
            &mut css,
            edge_animation_active,
            focus_mode,
        );

        // Append the git-graph connectors between process steps. These bypass
        // the thing/tag edge router entirely; their tailwind classes are already
        // present in `tailwind_classes` (resolved from the theme's edge_defaults
        // during IR mapping).
        svg_edge_infos.extend(ProcessStepGraphEdgesBuilder::build(
            ir_diagram,
            taffy_tree,
            node_id_to_taffy,
        ));

        let edge_label_infos = SvgEdgeLabelsBuilder::build(
            taffy_tree,
            edge_label_taffy_nodes,
            entity_highlighted_spans,
            entity_image_spans,
        );

        let edge_description_infos = SvgEdgeDescriptionsBuilder::build(
            taffy_tree,
            edge_description_taffy_nodes,
            edge_description_highlighted_spans,
            edge_description_image_spans,
        );

        SvgElements::new(
            svg_width,
            svg_height,
            svg_node_infos,
            svg_edge_infos,
            edge_label_infos,
            edge_description_infos,
            svg_process_infos,
            tailwind_classes,
            css,
        )
    }
}
