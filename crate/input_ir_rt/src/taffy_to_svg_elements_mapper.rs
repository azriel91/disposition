use std::fmt::Write;

use disposition_ir_model::{
    edge::{Edge, EdgeGroups, EdgeId},
    entity::{EntityTailwindClasses, EntityTypes},
    layout::NodeLayout,
    node::{NodeId, NodeInbuilt, NodeShape, NodeShapeRect},
    IrDiagram,
};
use disposition_model_common::{edge::EdgeGroupId, entity::EntityType, theme::Css, Id, Map, Set};
use disposition_svg_model::{SvgEdgeInfo, SvgElements, SvgNodeInfo, SvgProcessInfo, SvgTextSpan};
use disposition_taffy_model::{
    EntityHighlightedSpans, NodeContext, NodeToTaffyNodeIds, TaffyNodeMappings, TEXT_LINE_HEIGHT,
};
use kurbo::{BezPath, Point, Shape};
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
                        let svg_process_info = Self::build_svg_process_info(
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
                        let svg_node_info = Self::build_svg_node_info(
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
        let svg_node_info_map: Map<&NodeId<'id>, &SvgNodeInfo<'id>> = svg_node_infos
            .iter()
            .map(|info| (&info.node_id, info))
            .collect();

        // Clone css from ir_diagram; edge animation CSS will be appended.
        let mut css = ir_diagram.css.clone();

        // Build edge information and compute animation data for interaction
        // edges.
        let svg_edge_infos = Self::build_svg_edge_infos(
            &ir_diagram.edge_groups,
            &ir_diagram.entity_types,
            &svg_node_info_map,
            &mut tailwind_classes,
            &mut css,
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

    /// Returns the [`SvgProcessInfo`] for the given process IR node.
    fn build_svg_process_info<'ctx, 'id>(
        svg_process_info_build_context: SvgProcessInfoBuildContext<'ctx, 'id>,
        process_idx: usize,
        process_steps_height: &ProcessStepsHeight<'id>,
        process_node_id: &NodeId<'id>,
        taffy_node_id: taffy::NodeId,
        layout: &taffy::Layout,
    ) -> SvgProcessInfo<'id> {
        let SvgProcessInfoBuildContext {
            ir_diagram,
            taffy_tree,
            default_shape,
            process_steps_heights,
        } = svg_process_info_build_context;

        // Calculate y coordinate
        let y = {
            let mut y_acc = layout.location.y;
            let mut current_node_id = taffy_node_id;
            while let Some(parent_taffy_node_id) = taffy_tree.parent(current_node_id) {
                let Ok(parent_layout) = taffy_tree.layout(parent_taffy_node_id) else {
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
            .unwrap_or(default_shape);

        let path_d_expanded = Self::build_rect_path(width, height_expanded, node_shape);

        let process_steps_height_predecessors_cumulative =
            Self::process_steps_height_predecessors_cumulative(process_steps_heights, process_idx);
        let base_y = y - process_steps_height_predecessors_cumulative;

        SvgProcessInfo::new(
            height_expanded,
            path_d_expanded,
            process_steps_height.process_id.clone(),
            process_steps_height.process_step_ids.clone(),
            process_idx,
            process_steps_height.total_height,
            base_y,
        )
    }

    /// Returns the [`SvgNodeInfo`] for the given IR node.
    fn build_svg_node_info<'ctx, 'id>(
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

        let path_d_collapsed = Self::build_rect_path(width, height_collapsed, node_shape);
        let translate_classes = Self::build_translate_classes(
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

        if let Some(tailwind_classes) =
            entity_tailwind_classes.get_mut(AsRef::<Id<'_>>::as_ref(node_id))
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
                        children.keys().cloned().collect::<Set<NodeId<'_>>>(),
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

    /// Builds translate-x and translate-y tailwind classes for nodes.
    ///
    /// * Process nodes will have classes that collapse depending on focus on
    ///   them or their steps.
    /// * Non-process nodes will have simple translate-x and translate-y
    ///   classes.
    #[allow(clippy::too_many_arguments)]
    fn build_translate_classes<'id>(
        process_steps_heights: &[ProcessStepsHeight<'_>],
        process_infos: &Map<NodeId<'id>, SvgProcessInfo<'id>>,
        x: f32,
        y: f32,
        process_id: &Option<NodeId<'_>>,
        width: f32,
        height_expanded: f32,
        height_to_expand_to: Option<f32>,
        node_shape: &NodeShape,
        path_d_collapsed: &str,
    ) -> String {
        if let Some(ref proc_id) = *process_id
            && let Some(proc_info) = process_infos.get(proc_id)
        {
            // Calculate base_y for this specific node
            let process_steps_height_predecessors_cumulative =
                Self::process_steps_height_predecessors_cumulative(
                    process_steps_heights,
                    proc_info.process_index,
                );
            let base_y = y - process_steps_height_predecessors_cumulative;

            // Build path_d_expanded for this node if it's a process
            let path_d_expanded = if height_to_expand_to.is_some() {
                Self::build_rect_path(width, height_expanded, node_shape)
            } else {
                path_d_collapsed.to_string()
            };

            Self::build_translate_classes_for_process(
                x,
                base_y,
                path_d_collapsed,
                height_to_expand_to,
                &path_d_expanded,
                proc_info.process_index,
                process_steps_heights,
            )
        } else {
            Self::build_translate_classes_for_node(x, y, path_d_collapsed)
        }
    }

    /// Builds simple translate-x and translate-y tailwind classes for
    /// non-process/step nodes.
    fn build_translate_classes_for_node(x: f32, y: f32, path_d_collapsed: &str) -> String {
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
    fn build_translate_classes_for_process(
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

    /// Builds [`SvgEdgeInfo`] for all edges in the diagram.
    ///
    /// This iterates over all edge groups and their edges, computing the
    /// curved path for each edge based on the relative positions of the
    /// source and target nodes.
    fn build_svg_edge_infos<'id>(
        edge_groups: &EdgeGroups<'id>,
        entity_types: &EntityTypes<'id>,
        svg_node_info_map: &Map<&NodeId<'id>, &SvgNodeInfo<'id>>,
        tailwind_classes: &mut EntityTailwindClasses<'id>,
        css: &mut Css,
    ) -> Vec<SvgEdgeInfo<'id>> {
        let mut svg_edge_infos = Vec::new();

        // The keyframe percentages of an edge's animation should be proportional to the
        // length of the edge within the total length of all edges in its edge group.
        //
        // So we need to precompute the total length of all edges in each edge group,
        // and pass that in when computing the keyframe percentages.
        //
        // If we only computed the edge's keyframe percentages based on its index within
        // the edge group, the line would be animated faster because it would travel a
        // larger distance in the same amount of time, whereas it is easier to
        // understand if we got each edge to be animated at the same speed (which means
        // increasing the amount of time for that edge's animation).
        //
        // Algorithm:
        //
        // 1. Compute the total length of all edges in each edge group using the edge's
        //    `bounding_box` as an approximation, then sum them.
        // 2. The `total_animation_time` should be a constant
        //    `seconds_per_distance_units * total_length`.
        // 3. The `start_pct` ("request start") will be `preceding_edge_lengths_sum /
        //    total_length`.
        // 4. The `end_pct` ("request end") will be `(preceding_edge_lengths_sum +
        //    current_edge_length) / total_length`.
        // 5. The `duration` for each edge's animation will be the `total_animation_time
        //    * (edge_length / total_length)`.

        /// 1 second per 100 pixels
        const SECONDS_PER_PIXEL: f64 = 1.0 / 100.0;

        edge_groups.iter().for_each(|(edge_group_id, edge_group)| {
            let edge_path_infos = edge_group
                .iter()
                .enumerate()
                .filter_map(|(edge_index, edge)| {
                    // Skip edges where either node is not found
                    let Some(from_info) = svg_node_info_map.get(&edge.from) else {
                        // TODO: warn user that they probably got a Node ID wrong.
                        return None;
                    };
                    let Some(to_info) = svg_node_info_map.get(&edge.to) else {
                        // TODO: warn user that they probably got a Node ID wrong.
                        return None;
                    };

                    let edge_id = Self::generate_edge_id(edge_group_id, edge_index);

                    let edge_type = entity_types
                        .get(&*edge_id)
                        .map(|entity_types_for_edge| {
                            if [
                                EntityType::DependencyEdgeSequenceForwardDefault,
                                EntityType::DependencyEdgeCyclicForwardDefault,
                                EntityType::InteractionEdgeSequenceForwardDefault,
                                EntityType::InteractionEdgeCyclicForwardDefault,
                            ]
                            .iter()
                            .any(|entity_type_edge_forward| {
                                entity_types_for_edge.contains(entity_type_edge_forward)
                            }) {
                                EdgeType::Unpaired
                            } else if [
                                EntityType::DependencyEdgeSymmetricForwardDefault,
                                EntityType::InteractionEdgeSymmetricForwardDefault,
                            ]
                            .iter()
                            .any(|entity_type_edge_forward| {
                                entity_types_for_edge.contains(entity_type_edge_forward)
                            }) {
                                EdgeType::PairRequest
                            } else if [
                                EntityType::DependencyEdgeSymmetricReverseDefault,
                                EntityType::InteractionEdgeSymmetricReverseDefault,
                            ]
                            .iter()
                            .any(|entity_type_edge_reverse| {
                                entity_types_for_edge.contains(entity_type_edge_reverse)
                            }) {
                                EdgeType::PairResponse
                            } else {
                                EdgeType::Unpaired
                            }
                        })
                        .unwrap_or(EdgeType::Unpaired);

                    let path = Self::build_edge_path(from_info, to_info, edge_type);
                    let path_length = {
                        // not sure what this is, but I assume it means 1 pixel accuracy
                        let accuracy = 1.0;
                        path.perimeter(accuracy)
                    };

                    let edge_path_info = EdgePathInfo {
                        edge_id,
                        edge,
                        edge_type,
                        path,
                        path_length,
                    };

                    Some(edge_path_info)
                })
                .collect::<Vec<EdgePathInfo>>();

            // Rough total length of all edges in the group
            //
            // This is rough because it just uses path bounding box width and height instead
            // of actual path length.
            let edge_animation_params = EdgeAnimationParams::default();
            let visible_segments_length = edge_animation_params.visible_segments_length;
            let edge_group_path_length_total = edge_path_infos
                .iter()
                .map(|edge_path_info| edge_path_info.path_length)
                .sum::<f64>();
            let edge_group_visible_segments_length_total =
                edge_path_infos.len() as f64 * visible_segments_length;
            let edge_group_animation_duration_total_s = SECONDS_PER_PIXEL
                * edge_group_visible_segments_length_total
                + edge_animation_params.pause_duration_secs;

            let (edge_animation_infos, _preceding_visible_segments_lengths) =
                edge_path_infos.into_iter().fold(
                    (Vec::new(), 0.0),
                    |(mut edge_animation_infos, preceding_visible_segments_lengths),
                     edge_path_info| {
                        let EdgePathInfo {
                            edge_id,
                            edge,
                            edge_type,
                            path,
                            path_length,
                        } = edge_path_info;

                        edge_animation_infos.push(EdgeAnimationInfo {
                            edge_id,
                            edge,
                            edge_type,
                            path,
                            path_length,
                            preceding_visible_segments_lengths,
                        });

                        (
                            edge_animation_infos,
                            preceding_visible_segments_lengths + visible_segments_length,
                        )
                    },
                );

            edge_animation_infos
                .into_iter()
                .for_each(|edge_animation_info| {
                    // Compute animation for interaction edges.
                    let is_interaction_edge = entity_types
                        .get(AsRef::<Id<'_>>::as_ref(&edge_animation_info.edge_id))
                        .map(|edge_entity_types| {
                            edge_entity_types
                                .iter()
                                .any(EntityType::is_interaction_edge_type)
                        })
                        .unwrap_or(false);

                    if is_interaction_edge {
                        let edge_anim = Self::compute_edge_animation(
                            edge_animation_params,
                            edge_group_path_length_total,
                            edge_group_animation_duration_total_s,
                            &edge_animation_info,
                        );

                        // Append dasharray and animate tailwind classes to this
                        // edge's existing classes.
                        let edge_id_owned: Id<'id> =
                            edge_animation_info.edge_id.clone().into_inner();
                        let existing = tailwind_classes
                            .get(&edge_id_owned)
                            .cloned()
                            .unwrap_or_default();
                        let animation_classes = format!(
                            "[stroke-dasharray:{}]\nanimate-[{}_{}s_linear_infinite]",
                            edge_anim.dasharray,
                            edge_anim.animation_name,
                            format_duration(edge_anim.edge_animation_duration_s),
                        );
                        let combined = if existing.is_empty() {
                            animation_classes
                        } else {
                            format!("{existing}\n{animation_classes}")
                        };
                        tailwind_classes.insert(edge_id_owned, combined);

                        // Append CSS keyframes.
                        if !css.is_empty() {
                            css.push('\n');
                        }
                        css.push_str(&edge_anim.keyframe_css);
                    }

                    let EdgeAnimationInfo {
                        edge_id,
                        edge,
                        edge_type: _,
                        path,
                        path_length: _,
                        preceding_visible_segments_lengths: _,
                    } = edge_animation_info;

                    let path_d = path.to_svg();

                    svg_edge_infos.push(SvgEdgeInfo::new(
                        edge_id,
                        edge_group_id.clone(),
                        edge.from.clone(),
                        edge.to.clone(),
                        path_d,
                    ));
                });
        });

        svg_edge_infos
    }

    /// Generates an edge ID from the edge group ID and edge index.
    fn generate_edge_id(edge_group_id: &EdgeGroupId<'_>, edge_index: usize) -> EdgeId<'static> {
        let edge_id_str = format!("{edge_group_id}__{edge_index}");
        Id::try_from(edge_id_str)
            .expect("edge ID should be valid")
            .into()
    }

    /// Computes the stroke-dasharray, CSS keyframes, and animation name for an
    /// interaction edge.
    ///
    /// # Parameters
    ///
    /// * `edge_group_animation_duration_total_s`: Duration of the animation for
    ///   the edges for the entire edge group, which excludes the pause at the
    ///   end of the animation.
    fn compute_edge_animation(
        params: EdgeAnimationParams,
        edge_group_path_length_total: f64,
        edge_group_animation_duration_total_s: f64,
        edge_animation_info: &EdgeAnimationInfo<'_, '_>,
    ) -> EdgeAnimation {
        let EdgeAnimationInfo {
            edge_id,
            edge: _,
            edge_type,
            path: _,
            path_length,
            preceding_visible_segments_lengths,
        } = edge_animation_info;

        let is_reverse = *edge_type == EdgeType::PairResponse;

        // Generate the decreasing visible segment lengths using a geometric
        // series.
        let segments = compute_dasharray_segments(&params);

        // Use the path length so the trailing gap fully hides the
        // edge during the invisible phase of the animation.
        let trailing_gap = path_length.max(params.visible_segments_length);

        // Build the dasharray string with segments in the correct order.
        let dasharray =
            build_dasharray_string(&segments, params.gap_width, trailing_gap, is_reverse);

        // Derive a unique animation name from the edge ID by replacing
        // underscores with hyphens (tailwind translates underscores to spaces
        // inside arbitrary values).
        let animation_name = format!("{}--stroke-dashoffset", edge_id.as_str().replace('_', "-"));

        // Keyframe percentages for this edge's slot within the cycle.
        let start_pct = preceding_visible_segments_lengths / edge_group_path_length_total * 100.0;
        let end_pct = (preceding_visible_segments_lengths + params.visible_segments_length)
            / edge_group_path_length_total
            * 100.0;

        // stroke-dashoffset values:
        // - start_offset: shifts visible segments entirely before the path
        // - end_offset:   shifts visible segments entirely past the path
        let visible_length = params.visible_segments_length;
        let start_offset = -visible_length;
        let end_offset = visible_length + visible_length + trailing_gap;

        // Build the CSS @keyframes rule, omitting duplicate percentage entries
        // at 0% and 100% when they coincide with start_pct / end_pct.
        let mut keyframe_css = format!("@keyframes {} {{\n", animation_name);

        if start_pct > 0.0 {
            let _ = write!(
                keyframe_css,
                "  0% {{ stroke-dashoffset: {start_offset:.1}; }}\n"
            );
        }
        let _ = write!(
            keyframe_css,
            "  {start_pct:.1}% {{ stroke-dashoffset: {start_offset:.1}; }}\n"
        );
        let _ = write!(
            keyframe_css,
            "  {end_pct:.1}% {{ stroke-dashoffset: {end_offset:.1}; }}\n"
        );
        if end_pct < 100.0 {
            let _ = write!(
                keyframe_css,
                "  100% {{ stroke-dashoffset: {end_offset:.1}; }}\n"
            );
        }
        keyframe_css.push('}');

        EdgeAnimation {
            dasharray,
            keyframe_css,
            animation_name,
            edge_animation_duration_s: edge_group_animation_duration_total_s,
        }
    }

    /// Builds the SVG path `d` attribute for an edge between two nodes.
    ///
    /// The path is a curved bezier curve that connects the appropriate faces
    /// of the source and target nodes based on their relative positions.
    fn build_edge_path(
        from_info: &SvgNodeInfo,
        to_info: &SvgNodeInfo,
        edge_type: EdgeType,
    ) -> BezPath {
        // Constants for edge layout

        /// Percentage of the node's width to offset the edge's x coordinate
        /// from the midpoint of the node.
        const SELF_LOOP_X_OFFSET_RATIO: f32 = 0.2;
        /// Percentage of the node's height to extend the edge vertically.
        const SELF_LOOP_Y_EXTENSION_RATIO: f32 = 0.2;
        /// Percentage of the node's width to curve the edge horizontally
        /// outward.
        const SELF_LOOP_X_EXTENSION_RATIO: f32 = 0.2;
        /// Percentage of the node's width/height to offset the edge when
        /// connecting to another edge.
        const BIDIRECTIONAL_OFFSET_RATIO: f32 = 0.1;
        /// Percentage of the node's width/height to curve the edge outward.
        const CURVE_CONTROL_RATIO: f32 = 0.3;

        // Handle self-loop case
        if from_info.node_id == to_info.node_id {
            return Self::build_self_loop_path(
                from_info,
                edge_type,
                SELF_LOOP_X_OFFSET_RATIO,
                SELF_LOOP_Y_EXTENSION_RATIO,
                SELF_LOOP_X_EXTENSION_RATIO,
            );
        }

        // Determine which faces to use based on relative positions
        let (from_face, to_face) = Self::select_edge_faces(from_info, to_info);

        // Check if from is contained inside to
        let from_contained_in_to = Self::is_node_contained_in(from_info, to_info);
        if from_contained_in_to {
            return Self::build_contained_edge_path(from_info, to_info, CURVE_CONTROL_RATIO);
        }

        // Get base connection points
        let (mut start_x, mut start_y) = Self::get_face_center(from_info, from_face);
        let (mut end_x, mut end_y) = Self::get_face_center(to_info, to_face);

        // Apply bidirectional offset
        if edge_type == EdgeType::PairRequest || edge_type == EdgeType::PairResponse {
            let offset_direction = if edge_type == EdgeType::PairResponse {
                1.0
            } else {
                -1.0
            };

            // Move start point down if this is the `PairRequest` edge.
            match from_face {
                Face::Right | Face::Left => {
                    start_y +=
                        from_info.height_collapsed * BIDIRECTIONAL_OFFSET_RATIO * offset_direction;
                }
                Face::Top | Face::Bottom => {
                    start_x += from_info.width * BIDIRECTIONAL_OFFSET_RATIO * offset_direction;
                }
            }

            // Move end point down if this is the `PairResponse` edge.
            match to_face {
                Face::Right | Face::Left => {
                    end_y +=
                        to_info.height_collapsed * BIDIRECTIONAL_OFFSET_RATIO * offset_direction;
                }
                Face::Top | Face::Bottom => {
                    end_x += to_info.width * BIDIRECTIONAL_OFFSET_RATIO * offset_direction;
                }
            }
        }

        // Build curved path
        Self::build_curved_edge_path(
            start_x,
            start_y,
            end_x,
            end_y,
            from_face,
            to_face,
            CURVE_CONTROL_RATIO,
        )
    }

    /// Builds a self-loop path that goes from the bottom of a node, extends
    /// down, curves left, and returns to the bottom of the same node.
    fn build_self_loop_path(
        node_info: &SvgNodeInfo,
        edge_type: EdgeType,
        x_offset_ratio: f32,
        y_extension_ratio: f32,
        x_extension_ratio: f32,
    ) -> BezPath {
        let start_x = node_info.x + node_info.width * (0.5 + x_offset_ratio);
        let start_y = node_info.y + node_info.height_collapsed;
        let end_x = node_info.x + node_info.width * (0.5 - x_offset_ratio);
        let end_y = start_y;

        let extension_y = TEXT_LINE_HEIGHT.max(node_info.height_collapsed * y_extension_ratio);
        let extension_x = node_info.width * x_extension_ratio;

        let start = Point::new(start_x as f64, start_y as f64);

        // Control points for the self-loop curve
        let ctrl1 = Point::new(
            (start_x + extension_x * 0.5) as f64,
            (start_y + extension_y) as f64,
        );
        let mid = Point::new(
            (node_info.x + node_info.width * 0.5) as f64,
            (start_y + extension_y) as f64,
        );

        let ctrl3 = Point::new(
            (end_x - extension_x * 0.5) as f64,
            (start_y + extension_y) as f64,
        );
        let end = Point::new(end_x as f64, end_y as f64);

        // Paths have to be built in reverse to get them to render in the correct
        // direction in the SVG.
        let mut path = BezPath::new();
        match edge_type {
            EdgeType::Unpaired | EdgeType::PairRequest => {
                path.move_to(end);
                path.curve_to(end, ctrl3, mid);
                path.curve_to(mid, ctrl1, start);
            }
            EdgeType::PairResponse => {
                path.move_to(start);
                path.curve_to(start, ctrl1, mid);
                path.curve_to(mid, ctrl3, end);
            }
        }

        path
    }

    /// Builds a path for an edge where the source node is contained inside the
    /// target node.
    fn build_contained_edge_path(
        from_info: &SvgNodeInfo,
        to_info: &SvgNodeInfo,
        curve_ratio: f32,
    ) -> BezPath {
        // Start from bottom of from node
        let start_x = from_info.x + from_info.width * 0.5;
        let start_y = from_info.y + from_info.height_collapsed;

        // End at left face of to node
        let end_x = to_info.x;
        let end_y = to_info.y + to_info.height_collapsed * 0.5;

        // Control points: go down, then left, then up
        let ctrl_distance = (start_y - end_y).abs().max(from_info.width) * curve_ratio;

        let ctrl1 = Point::new(start_x as f64, (start_y + ctrl_distance) as f64);
        let ctrl2 = Point::new((end_x - ctrl_distance) as f64, end_y as f64);
        let end = Point::new(end_x as f64, end_y as f64);

        // Paths have to be built in reverse to get them to render in the correct
        // direction in the SVG.
        let mut path = BezPath::new();
        let start = Point::new(start_x as f64, start_y as f64);
        // path.move_to(start);
        // path.curve_to(ctrl1, ctrl2, end);
        path.move_to(end);
        path.curve_to(ctrl2, ctrl1, start);

        path
    }

    /// Selects the appropriate faces for connecting two nodes based on their
    /// relative positions, choosing the faces that produce the shortest path.
    fn select_edge_faces(from_info: &SvgNodeInfo, to_info: &SvgNodeInfo) -> (Face, Face) {
        let from_center_x = from_info.x + from_info.width / 2.0;
        let from_center_y = from_info.y + from_info.height_collapsed / 2.0;
        let to_center_x = to_info.x + to_info.width / 2.0;
        let to_center_y = to_info.y + to_info.height_collapsed / 2.0;

        let dx = to_center_x - from_center_x;
        let dy = to_center_y - from_center_y;

        // Check for clear horizontal or vertical alignment
        let from_right = from_info.x + from_info.width;
        let to_right = to_info.x + to_info.width;
        let from_bottom = from_info.y + from_info.height_collapsed;
        let to_bottom = to_info.y + to_info.height_collapsed;

        // Node is clearly to the right (no horizontal overlap)
        if from_right < to_info.x {
            if from_bottom < to_info.y {
                // Diagonal: from is above-left of to
                return Self::select_diagonal_faces(
                    from_info,
                    to_info,
                    Face::Right,
                    Face::Bottom,
                    Face::Left,
                    Face::Top,
                );
            } else if from_info.y > to_bottom {
                // Diagonal: from is below-left of to
                return Self::select_diagonal_faces(
                    from_info,
                    to_info,
                    Face::Right,
                    Face::Top,
                    Face::Left,
                    Face::Bottom,
                );
            }
            return (Face::Right, Face::Left);
        }

        // Node is clearly to the left (no horizontal overlap)
        if to_right < from_info.x {
            if from_bottom < to_info.y {
                // Diagonal: from is above-right of to
                return Self::select_diagonal_faces(
                    from_info,
                    to_info,
                    Face::Left,
                    Face::Bottom,
                    Face::Right,
                    Face::Top,
                );
            } else if from_info.y > to_bottom {
                // Diagonal: from is below-right of to
                return Self::select_diagonal_faces(
                    from_info,
                    to_info,
                    Face::Left,
                    Face::Top,
                    Face::Right,
                    Face::Bottom,
                );
            }
            return (Face::Left, Face::Right);
        }

        // Node is clearly below (no vertical overlap but horizontal overlap)
        if from_bottom < to_info.y {
            return (Face::Bottom, Face::Top);
        }

        // Node is clearly above (no vertical overlap but horizontal overlap)
        if to_bottom < from_info.y {
            return (Face::Top, Face::Bottom);
        }

        // Overlapping nodes - use primary direction
        if dx.abs() > dy.abs() {
            if dx > 0.0 {
                (Face::Right, Face::Left)
            } else {
                (Face::Left, Face::Right)
            }
        } else if dy > 0.0 {
            (Face::Bottom, Face::Top)
        } else {
            (Face::Top, Face::Bottom)
        }
    }

    /// Selects the best faces for diagonal connections by comparing distances.
    fn select_diagonal_faces(
        from_info: &SvgNodeInfo,
        to_info: &SvgNodeInfo,
        from_horiz: Face,
        from_vert: Face,
        to_horiz: Face,
        to_vert: Face,
    ) -> (Face, Face) {
        // Calculate distances for horizontal-to-vertical vs vertical-to-horizontal
        let (from_h_x, from_h_y) = Self::get_face_center(from_info, from_horiz);
        let (to_v_x, to_v_y) = Self::get_face_center(to_info, to_vert);
        let dist_h_to_v = ((to_v_x - from_h_x).powi(2) + (to_v_y - from_h_y).powi(2)).sqrt();

        let (from_v_x, from_v_y) = Self::get_face_center(from_info, from_vert);
        let (to_h_x, to_h_y) = Self::get_face_center(to_info, to_horiz);
        let dist_v_to_h = ((to_h_x - from_v_x).powi(2) + (to_h_y - from_v_y).powi(2)).sqrt();

        if dist_h_to_v <= dist_v_to_h {
            (from_horiz, to_vert)
        } else {
            (from_vert, to_horiz)
        }
    }

    /// Gets the center point of a node's face.
    fn get_face_center(node_info: &SvgNodeInfo, face: Face) -> (f32, f32) {
        match face {
            Face::Top => (node_info.x + node_info.width / 2.0, node_info.y),
            Face::Bottom => (
                node_info.x + node_info.width / 2.0,
                node_info.y + node_info.height_collapsed,
            ),
            Face::Left => (node_info.x, node_info.y + node_info.height_collapsed / 2.0),
            Face::Right => (
                node_info.x + node_info.width,
                node_info.y + node_info.height_collapsed / 2.0,
            ),
        }
    }

    /// Checks if a node is geometrically contained within another node.
    fn is_node_contained_in(inner: &SvgNodeInfo, outer: &SvgNodeInfo) -> bool {
        inner.x >= outer.x
            && inner.y >= outer.y
            && inner.x + inner.width <= outer.x + outer.width
            && inner.y + inner.height_collapsed <= outer.y + outer.height_collapsed
    }

    /// Builds a curved bezier path between two points with control points
    /// based on the faces being connected.
    fn build_curved_edge_path(
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        from_face: Face,
        to_face: Face,
        curve_ratio: f32,
    ) -> BezPath {
        let dx = end_x - start_x;
        let dy = end_y - start_y;
        let distance = (dx * dx + dy * dy).sqrt();
        let ctrl_distance = distance * curve_ratio;

        // Calculate control points based on face directions
        let start = Point::new(start_x as f64, start_y as f64);
        let (ctrl1_x, ctrl1_y) = Self::get_control_point_offset(from_face, ctrl_distance);
        let (ctrl2_x, ctrl2_y) = Self::get_control_point_offset(to_face, ctrl_distance);
        let ctrl1 = Point::new((start_x + ctrl1_x) as f64, (start_y + ctrl1_y) as f64);
        let ctrl2 = Point::new((end_x + ctrl2_x) as f64, (end_y + ctrl2_y) as f64);
        let end = Point::new(end_x as f64, end_y as f64);

        // Paths have to be built in reverse to get them to render in the correct
        // direction in the SVG.
        let mut path = BezPath::new();
        // path.move_to(start);
        // path.curve_to(ctrl1, ctrl2, end);
        path.move_to(end);
        path.curve_to(ctrl2, ctrl1, start);

        path
    }

    /// Gets the control point offset direction based on the face.
    fn get_control_point_offset(face: Face, distance: f32) -> (f32, f32) {
        match face {
            Face::Top => (0.0, -distance),
            Face::Bottom => (0.0, distance),
            Face::Left => (-distance, 0.0),
            Face::Right => (distance, 0.0),
        }
    }
}

/// Parameters for edge stroke-dasharray animation generation.
///
/// These control how the decreasing visible segments in the dasharray are
/// computed and how the CSS keyframe animation is timed.
#[derive(Clone, Copy, Debug)]
struct EdgeAnimationParams {
    /// Total length of visible segments plus inter-segment gaps.
    ///
    /// This does **not** include the trailing gap used to hide the edge.
    visible_segments_length: f64,
    /// Constant gap width between adjacent visible segments.
    gap_width: f64,
    /// Number of visible segments in the dasharray.
    segment_count: usize,
    /// Geometric ratio for each successive segment (0 < ratio < 1).
    ///
    /// Each segment is `ratio` times the length of the previous one,
    /// producing a visually decreasing pattern.
    segment_ratio: f64,
    /// Duration in seconds to pause (all edges invisible) before the
    /// animation cycle restarts.
    pause_duration_secs: f64,
}

impl Default for EdgeAnimationParams {
    fn default() -> Self {
        Self {
            visible_segments_length: 100.0,
            gap_width: 2.0,
            segment_count: 8,
            segment_ratio: 0.6,
            pause_duration_secs: 1.0,
        }
    }
}

/// Result of computing edge animation data.
struct EdgeAnimation {
    /// The stroke-dasharray value string, e.g. `"30.0,2.0,20.0,...,400.0"`.
    dasharray: String,
    /// The CSS `@keyframes` rule for this edge.
    keyframe_css: String,
    /// Unique animation name for the keyframes rule.
    animation_name: String,
    /// Total animation cycle duration in seconds.
    edge_animation_duration_s: f64,
}

/// Generates the visible segment lengths using a geometric series.
///
/// Given `n` segments with ratio `r`, the first segment length `a` is
/// computed so that:
///
/// ```text
/// a + a*r + a*r^2 + ... + a*r^(n-1) + (n-1)*gap = visible_segments_length
/// ```
///
/// Each successive segment is `r` times the previous, producing a visually
/// decreasing pattern (e.g. long dash, medium dash, short dash, ...).
fn compute_dasharray_segments(params: &EdgeAnimationParams) -> Vec<f64> {
    let n = params.segment_count;
    let r = params.segment_ratio;
    let g = params.gap_width;
    let total = params.visible_segments_length;

    // Space available for visible segments after subtracting inter-segment gaps.
    let available = total - (n as f64 - 1.0) * g;
    assert!(
        available > 0.0,
        "visible_segments_length ({total}) must be larger than the total gap \
         space ({} * {g} = {})",
        n - 1,
        (n as f64 - 1.0) * g,
    );

    // Sum of geometric series: a * (1 - r^n) / (1 - r)
    let weight_sum = (1.0 - r.powi(n as i32)) / (1.0 - r);
    let first = available / weight_sum;

    (0..n)
        .map(|i| (first * r.powi(i as i32)).max(0.5))
        .collect()
}

/// Builds the stroke-dasharray value string from visible segments.
///
/// For forward edges the segments are in decreasing order (largest first).
/// For reverse edges the segments are in increasing order (smallest first),
/// producing a visual "building up" effect for the response direction.
///
/// The trailing gap is appended at the end so the edge is hidden during the
/// invisible portion of the animation cycle.
fn build_dasharray_string(
    segments: &[f64],
    gap_width: f64,
    trailing_gap: f64,
    is_reverse: bool,
) -> String {
    let ordered: Vec<f64> = if is_reverse {
        segments.iter().copied().rev().collect()
    } else {
        segments.to_vec()
    };

    let mut parts = Vec::with_capacity(ordered.len() * 2 + 1);
    for (i, seg) in ordered.iter().enumerate() {
        if i > 0 {
            parts.push(format!("{gap_width:.1}"));
        }
        parts.push(format!("{seg:.1}"));
    }
    parts.push(format!("{trailing_gap:.1}"));

    parts.join(",")
}

/// Formats a duration in seconds for use in CSS, removing unnecessary trailing
/// zeros.
fn format_duration(secs: f64) -> String {
    if secs.fract() == 0.0 {
        format!("{}", secs as u64)
    } else {
        format!("{:.1}", secs)
    }
}

/// Represents a face/side of a rectangular node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Face {
    Top,
    Bottom,
    Left,
    Right,
}

/// Whether an edge represents an unpaired forward edge, or the request or
/// response of a pair of edges.
///
/// When two edges are paired, then their paths are offset from the midpoint of
/// the face of the node they are connected to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EdgeType {
    /// Forward direction of an unpaired edge.
    Unpaired,
    /// Request direction of a pair of edges.
    PairRequest,
    /// Response direction of a pair of edges.
    PairResponse,
}

/// Intermediate data for computing `SvgEdgeInfo`s.
///
/// This is collated because the sum of all path lengths in an edge group are
/// needed to compute the animation keyframe percentages for each edge.
#[derive(Clone, Debug)]
struct EdgePathInfo<'edge, 'id> {
    edge_id: EdgeId<'id>,
    edge: &'edge Edge<'id>,
    edge_type: EdgeType,
    path: BezPath,
    path_length: f64,
}

/// Intermediate data for computing `SvgEdgeInfo`s.
///
/// This is collated because the sum of all path lengths in an edge group are
/// needed to compute the animation keyframe percentages for each edge.
#[derive(Clone, Debug)]
struct EdgeAnimationInfo<'edge, 'id> {
    edge_id: EdgeId<'id>,
    edge: &'edge Edge<'id>,
    edge_type: EdgeType,
    path: BezPath,
    path_length: f64,
    preceding_visible_segments_lengths: f64,
}

#[derive(Clone, Copy, Debug)]
struct SvgProcessInfoBuildContext<'ctx, 'id> {
    ir_diagram: &'ctx IrDiagram<'id>,
    taffy_tree: &'ctx TaffyTree<NodeContext>,
    default_shape: &'ctx NodeShape,
    process_steps_heights: &'ctx [ProcessStepsHeight<'id>],
}

#[derive(Clone, Copy, Debug)]
struct SvgNodeInfoBuildContext<'ctx, 'id> {
    ir_diagram: &'ctx IrDiagram<'id>,
    taffy_tree: &'ctx TaffyTree<NodeContext>,
    entity_highlighted_spans: &'ctx EntityHighlightedSpans<'id>,
    default_shape: &'ctx NodeShape,
    process_steps_heights: &'ctx [ProcessStepsHeight<'id>],
    svg_process_infos: &'ctx Map<NodeId<'id>, SvgProcessInfo<'id>>,
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
    process_step_ids: Set<NodeId<'id>>,
    /// Total height of all process steps belonging to this process.
    total_height: f32,
}
