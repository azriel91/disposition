use std::fmt::Write;

use disposition_ir_model::{
    layout::NodeLayout,
    node::{NodeId, NodeInbuilt, NodeShape, NodeShapeRect},
    IrDiagram,
};
use disposition_model_common::{entity::EntityType, Map};
use disposition_svg_model::{SvgElements, SvgNodeInfo, SvgProcessInfo, SvgTextSpan};
use disposition_taffy_model::{NodeContext, NodeToTaffyNodeIds, TaffyNodeMappings};
use taffy::TaffyTree;

/// Maps the IR diagram and `TaffyNodeMappings` to SVG elements.
///
/// These include nodes with simple coordinates and edges.
#[derive(Clone, Copy, Debug)]
pub struct TaffyToSvgElementsMapper;

impl TaffyToSvgElementsMapper {
    pub fn map<'id>(
        ir_diagram: &IrDiagram<'id>,
        taffy_node_mappings: &TaffyNodeMappings<'id>,
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
            Self::process_step_heights_calculate(ir_diagram, taffy_tree, node_id_to_taffy);

        // Build process_infos map from process_steps_heights
        // We need to compute the actual values for each process node
        let mut process_infos: Map<NodeId<'id>, SvgProcessInfo<'id>> = Map::new();

        process_steps_heights
            .iter()
            .enumerate()
            .for_each(|(idx, psh)| {
                let process_node_id = &psh.process_id;

                // Look up taffy layout for the process node
                if let Some(taffy_node_ids) = node_id_to_taffy.get(process_node_id).copied() {
                    let taffy_node_id = taffy_node_ids.wrapper_taffy_node_id();
                    if let Ok(layout) = taffy_tree.layout(taffy_node_id) {
                        // Calculate y coordinate
                        let y = {
                            let mut y_acc = layout.location.y;
                            let mut current_node_id = taffy_node_id;
                            while let Some(parent_taffy_node_id) =
                                taffy_tree.parent(current_node_id)
                            {
                                let Ok(parent_layout) = taffy_tree.layout(parent_taffy_node_id)
                                else {
                                    break;
                                };
                                y_acc += parent_layout.location.y;
                                current_node_id = parent_taffy_node_id;
                            }
                            y_acc
                        };

                        let width = layout.size.width;
                        let height_expanded = layout.size.height.min(layout.content_size.height);

                        // Get the node shape (corner radii)
                        let node_shape = ir_diagram
                            .node_shapes
                            .get(process_node_id)
                            .unwrap_or(&default_shape);

                        let path_d_expanded =
                            Self::build_rect_path(width, height_expanded, node_shape);

                        let process_steps_height_predecessors_cumulative =
                            Self::process_steps_height_predecessors_cumulative(
                                &process_steps_heights,
                                idx,
                            );
                        let base_y = y - process_steps_height_predecessors_cumulative;

                        let process_info = SvgProcessInfo::new(
                            height_expanded,
                            path_d_expanded,
                            psh.process_id.clone(),
                            psh.process_step_ids.clone(),
                            idx,
                            psh.total_height,
                            base_y,
                        );

                        process_infos.insert(psh.process_id.clone(), process_info);
                    }
                }
            });

        let mut svg_node_infos: Vec<SvgNodeInfo<'id>> = Vec::new();
        let mut additional_tailwind_classes: Vec<String> = Vec::new();

        // Process nodes in the order specified by node_ordering
        ir_diagram
            .node_ordering
            .iter()
            .for_each(|(node_id, &tab_index)| {
                // Look up taffy layout for this node
                let Some(taffy_node_ids) = node_id_to_taffy.get(node_id).copied() else {
                    return;
                };
                let taffy_node_id = taffy_node_ids.wrapper_taffy_node_id();
                let Ok(layout) = taffy_tree.layout(taffy_node_id) else {
                    return;
                };

                let is_process = ir_diagram
                    .entity_types
                    .get(node_id.as_ref())
                    .map(|types| types.contains(&EntityType::ProcessDefault))
                    .unwrap_or(false);

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

                // Find the process ID this node belongs to (if any)
                let process_id = Self::find_process_id(node_id, ir_diagram, &process_infos);

                // TODO: if the process steps were the tallest elements in the diagram, the
                // diagram height may need to be reduced as well.
                let width = layout.size.width;
                let height_expanded = layout.size.height.min(layout.content_size.height);
                let height_collapsed = {
                    let mut node_height = height_expanded;

                    // If this is a process, subtract the height of its process steps.
                    if is_process {
                        if let Some(proc_info) = process_infos.get(node_id) {
                            node_height -= proc_info.total_height;
                        }
                    }

                    node_height
                };
                let height_to_expand_to = if is_process {
                    Some(height_expanded)
                } else {
                    None
                };

                // Get the node shape (corner radii)
                let node_shape = ir_diagram
                    .node_shapes
                    .get(node_id)
                    .unwrap_or(&default_shape);

                // Build path d attribute with collapsed height
                let path_d_collapsed = Self::build_rect_path(width, height_collapsed, node_shape);

                // Build translate classes
                let translate_classes = if let Some(ref proc_id) = process_id {
                    if let Some(proc_info) = process_infos.get(proc_id) {
                        // Calculate base_y for this specific node
                        let process_steps_height_predecessors_cumulative =
                            Self::process_steps_height_predecessors_cumulative(
                                &process_steps_heights,
                                proc_info.process_index,
                            );
                        let base_y = y - process_steps_height_predecessors_cumulative;

                        // Build path_d_expanded for this node if it's a process
                        let path_d_expanded = if height_to_expand_to.is_some() {
                            Self::build_rect_path(width, height_expanded, node_shape)
                        } else {
                            path_d_collapsed.clone()
                        };

                        Self::build_process_translate_classes(
                            x,
                            base_y,
                            &path_d_collapsed,
                            height_to_expand_to,
                            &path_d_expanded,
                            proc_info.process_index,
                            &process_steps_heights,
                        )
                    } else {
                        Self::build_translate_classes(x, y, &path_d_collapsed)
                    }
                } else {
                    Self::build_translate_classes(x, y, &path_d_collapsed)
                };

                // Collect translate classes for CSS generation
                additional_tailwind_classes.push(translate_classes);

                // Collect text spans
                let text_spans: Vec<SvgTextSpan> = entity_highlighted_spans
                    .get(node_id.as_ref())
                    .map(|spans| {
                        spans
                            .iter()
                            .map(|span| {
                                SvgTextSpan::new(span.x, span.y, Self::escape_xml(&span.text))
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                let svg_node_info = SvgNodeInfo::new(
                    node_id.clone(),
                    tab_index,
                    x,
                    y,
                    width,
                    height_collapsed,
                    path_d_collapsed,
                    process_id,
                    text_spans,
                );

                svg_node_infos.push(svg_node_info);
            });

        SvgElements::new(
            svg_width,
            svg_height,
            svg_node_infos,
            Vec::new(), // svg_edge_infos - empty for now
            process_infos,
            additional_tailwind_classes,
        )
    }

    /// Collects information about process nodes and their steps for
    /// y-coordinate calculations.
    ///
    /// Returns a vector of ProcessInfo in the order processes appear in
    /// node_ordering.
    fn process_step_heights_calculate<'id>(
        ir_diagram: &IrDiagram<'id>,
        taffy_tree: &TaffyTree<NodeContext>,
        node_id_to_taffy: &Map<NodeId<'id>, NodeToTaffyNodeIds>,
    ) -> Vec<ProcessStepsHeight<'id>> {
        let mut process_steps_height = Vec::new();

        // Iterate through node_ordering to find process nodes in order
        ir_diagram
            .node_hierarchy
            .iter()
            .filter_map(|(node_id, children)| {
                let is_process = ir_diagram
                    .entity_types
                    .get(node_id.as_ref())
                    .is_some_and(|types| types.contains(&EntityType::ProcessDefault));
                if is_process {
                    Some((
                        node_id.clone(),
                        children.keys().cloned().collect::<Vec<NodeId<'_>>>(),
                    ))
                } else {
                    None
                }
            })
            .for_each(|(process_id, process_step_ids)| {
                // Calculate total height of all steps
                let mut total_height = process_step_ids
                    .iter()
                    .filter_map(|process_step_id| node_id_to_taffy.get(process_step_id))
                    .copied()
                    .map(NodeToTaffyNodeIds::wrapper_taffy_node_id)
                    .filter_map(|taffy_node_id| taffy_tree.layout(taffy_node_id).ok())
                    .map(|layout| layout.size.height.min(layout.content_size.height))
                    .sum::<f32>();

                // Include the gap between the process name and the steps
                if let Some(NodeLayout::Flex(flex_layout)) =
                    ir_diagram.node_layouts.get(process_id.as_ref())
                {
                    total_height += flex_layout.gap * (process_step_ids.len() + 1) as f32;
                }

                process_steps_height.push(ProcessStepsHeight {
                    process_id,
                    process_step_ids,
                    total_height,
                });
            });

        process_steps_height
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
                .find(|(_, info)| info.process_step_ids.contains(node_id))
                .map(|(proc_id, _)| proc_id.clone())
        } else {
            None
        }
    }

    /// Computes the cumulative height of steps from all processes before the
    /// given process index.
    fn process_steps_height_predecessors_cumulative(
        process_steps_height: &[ProcessStepsHeight],
        process_index: usize,
    ) -> f32 {
        process_steps_height
            .iter()
            .take(process_index)
            .map(|p| p.total_height)
            .sum()
    }

    /// Builds simple translate-x and translate-y tailwind classes for
    /// non-process/step nodes.
    fn build_translate_classes(x: f32, y: f32, path_d_collapsed: &str) -> String {
        let mut classes = String::new();
        writeln!(&mut classes, "translate-x-[{x}px]").unwrap();
        writeln!(&mut classes, "translate-y-[{y}px]").unwrap();

        let mut path_d = path_d_collapsed.to_string();
        Self::char_replace_inplace(&mut path_d, ' ', '_');
        writeln!(&mut classes, "[&>path]:[d:path('{path_d}')]").unwrap();

        classes
    }

    /// Builds the translation tailwind classes for a process or process step
    /// node.
    ///
    /// This creates:
    /// 1. A `translate-x-*` class for horizontal positioning
    /// 2. A base `translate-y-*` class for the collapsed state
    /// 3. `group-has-[#id:focus-within]:translate-y-[..]` classes for when
    ///    previous processes are focused
    /// 4. transition-transform and duration classes for smooth animation
    /// 5. `[d:path(..)]` classes for collapsed and expanded path shapes
    #[allow(clippy::too_many_arguments)]
    fn build_process_translate_classes(
        x: f32,
        base_y: f32,
        path_d_collapsed: &str,
        height_to_expand_to: Option<f32>,
        path_d_expanded: &str,
        process_index: usize,
        process_steps_height: &[ProcessStepsHeight],
    ) -> String {
        let mut classes = String::new();

        // Add translate-x for horizontal positioning
        writeln!(&mut classes, "translate-x-[{x}px]").unwrap();

        // Build path d attribute with collapsed height
        let mut path_d_collapsed_escaped = path_d_collapsed.to_string();
        Self::char_replace_inplace(&mut path_d_collapsed_escaped, ' ', '_');
        writeln!(
            &mut classes,
            "[&>path]:[d:path('{path_d_collapsed_escaped}')]"
        )
        .unwrap();

        // When this process or any of its steps are focused, expand the height
        if height_to_expand_to.is_some() {
            let ProcessStepsHeight {
                process_id,
                process_step_ids,
                total_height: _,
            } = &process_steps_height[process_index];

            // Build path d attribute with expanded height
            let mut path_d_expanded_escaped = path_d_expanded.to_string();
            Self::char_replace_inplace(&mut path_d_expanded_escaped, ' ', '_');

            writeln!(
                &mut classes,
                "group-has-[#{process_id}:focus-within]:[&>path]:[d:path('{path_d_expanded_escaped}')]"
            )
            .unwrap();

            // Add classes for when any of the process's steps are focused
            process_step_ids.iter().for_each(|process_step_id| {
                writeln!(
                    &mut classes,
                    "group-has-[#{process_step_id}:focus-within]:[&>path]:[d:path('{path_d_expanded_escaped}')]"
                )
                .unwrap();
            });
        }

        // Add transition class for smooth animation
        writeln!(&mut classes, "transition-all").unwrap();
        writeln!(&mut classes, "duration-200").unwrap();

        // Base translate-y for collapsed state
        writeln!(&mut classes, "translate-y-[{base_y}px]").unwrap();

        // For each previous process, add a class that moves this node down when that
        // process is focused
        (0..process_index).for_each(|prev_idx| {
            let process_steps_height_prev = &process_steps_height[prev_idx];
            let ProcessStepsHeight {
                process_id,
                process_step_ids,
                total_height,
            } = process_steps_height_prev;

            // When this previous process (or any of its steps) is focused,
            // we need to add back that process's steps' height
            let y_when_prev_focused = base_y + total_height;

            // Add class for when the process itself is focused
            writeln!(
                &mut classes,
                "group-has-[#{process_id}:focus-within]:translate-y-[{y_when_prev_focused}px]"
            )
            .unwrap();

            // Add classes for when any of the process's steps are focused
            process_step_ids.iter().for_each(|process_step_id| {
                writeln!(
                    &mut classes,
                    "group-has-[#{process_step_id}:focus-within]:translate-y-[{y_when_prev_focused}px]"
                )
                .unwrap();
            });
        });

        classes
    }

    /// Builds an SVG path `d` attribute for a rectangle with optional corner
    /// radii.
    ///
    /// The path is constructed to draw a rectangle starting from just after
    /// the top-left corner (if rounded), proceeding clockwise:
    /// 1. Horizontal line to top-right corner
    /// 2. Arc for top-right corner (if radius > 0)
    /// 3. Vertical line to bottom-right corner
    /// 4. Arc for bottom-right corner (if radius > 0)
    /// 5. Horizontal line to bottom-left corner
    /// 6. Arc for bottom-left corner (if radius > 0)
    /// 7. Vertical line to top-left corner
    /// 8. Arc for top-left corner (if radius > 0)
    /// 9. Close path
    ///
    /// # Parameters
    /// - `width`: The width of the rectangle
    /// - `height`: The height of the rectangle
    /// - `node_shape`: The shape configuration containing corner radii
    pub fn build_rect_path(width: f32, height: f32, node_shape: &NodeShape) -> String {
        let NodeShape::Rect(rect) = node_shape;

        let r_tl = rect.radius_top_left;
        let r_tr = rect.radius_top_right;
        let r_bl = rect.radius_bottom_left;
        let r_br = rect.radius_bottom_right;

        let h = height;

        let mut d = String::with_capacity(128);

        // Move to start position (after top-left corner)
        write!(d, "M {r_tl} 0").unwrap();

        // Top edge: horizontal line to (width - r_tr, 0)
        write!(d, " H {}", width - r_tr).unwrap();

        // Top-right corner arc (if radius > 0)
        if r_tr > 0.0 {
            write!(d, " A {r_tr} {r_tr} 0 0 1 {width} {r_tr}").unwrap();
        }

        // Right edge: vertical line to (width, h - r_br)
        write!(d, " V {}", h - r_br).unwrap();

        // Bottom-right corner arc (if radius > 0)
        if r_br > 0.0 {
            write!(d, " A {r_br} {r_br} 0 0 1 {} {h}", width - r_br).unwrap();
        }

        // Bottom edge: horizontal line to (r_bl, h)
        write!(d, " H {r_bl}").unwrap();

        // Bottom-left corner arc (if radius > 0)
        if r_bl > 0.0 {
            write!(d, " A {r_bl} {r_bl} 0 0 1 0 {}", h - r_bl).unwrap();
        }

        // Left edge: vertical line to (0, r_tl)
        write!(d, " V {r_tl}").unwrap();

        // Top-left corner arc (if radius > 0)
        if r_tl > 0.0 {
            write!(d, " A {r_tl} {r_tl} 0 0 1 {r_tl} 0").unwrap();
        }

        // Close the path
        d.push_str(" Z");

        d
    }

    /// Replaces all occurrences of `from` byte with `to` byte in the given
    /// string, mutating it in place.
    ///
    /// # Safety
    ///
    /// This is safe because:
    ///
    /// * Both `from` and `to` must be ASCII bytes (single-byte UTF-8)
    /// * Replacing one ASCII byte with another ASCII byte preserves UTF-8
    ///   validity
    ///
    /// # Panics
    ///
    /// Panics in debug mode if either `from` or `to` is not ASCII.
    fn char_replace_inplace(s: &mut str, from: char, to: char) {
        debug_assert!(from.is_ascii(), "`from` byte must be ASCII");
        debug_assert!(to.is_ascii(), "`to` byte must be ASCII");

        // SAFETY: Replacing ASCII with ASCII preserves UTF-8 validity
        // because ASCII bytes are always single-byte UTF-8 sequences
        // and never appear as continuation bytes in multi-byte sequences.
        unsafe {
            s.as_bytes_mut().iter_mut().for_each(|byte| {
                if *byte == from as u8 {
                    *byte = to as u8;
                }
            });
        }
    }

    /// Escape XML special characters in text content
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

/// Heights for all steps within a process for y-coordinate calculations.
///
/// These are used to collapse processes to reduce the number of steps
/// displayed.
#[derive(Debug)]
struct ProcessStepsHeight<'id> {
    /// The node ID of the process.
    process_id: NodeId<'id>,
    /// List of process step node IDs belonging to this process.
    process_step_ids: Vec<NodeId<'id>>,
    /// Total height of all process steps belonging to this process.
    total_height: f32,
}
