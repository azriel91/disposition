use std::fmt::Write;

use base64::{prelude::BASE64_STANDARD, Engine};
use disposition_ir_model::{
    node::{NodeId, NodeInbuilt},
    IrDiagram,
};
use disposition_model_common::entity::EntityType;
use disposition_taffy_model::{
    EntityHighlightedSpans, NodeContext, NodeToTaffyNodeIds, TaffyNodeMappings, TEXT_FONT_SIZE,
    TEXT_LINE_HEIGHT,
};
use taffy::TaffyTree;

use crate::NOTO_SANS_MONO_TTF;

#[derive(Clone, Copy, Debug)]
pub struct TaffyToSvgMapper;

impl TaffyToSvgMapper {
    pub fn map(ir_diagram: &IrDiagram, taffy_node_mappings: TaffyNodeMappings) -> String {
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

        let mut content_buffer = String::with_capacity(4096);
        let mut styles_buffer = String::with_capacity(2048);
        let mut additional_tailwind_classes: Vec<String> = Vec::new();

        // Add default text styles
        writeln!(&mut styles_buffer, "text {{ font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace; font-size: {TEXT_FONT_SIZE}px; line-height: {TEXT_LINE_HEIGHT}px; }}").unwrap();

        // Add default font
        writeln!(&mut styles_buffer, "@font-face {{ font-family: 'Noto Sans Mono'; src: url(data:application/x-font-ttf;base64,{}) format('truetype'); }}", BASE64_STANDARD.encode(NOTO_SANS_MONO_TTF)).unwrap();

        // Render nodes in the order specified by node_ordering
        Self::render_nodes(
            ir_diagram,
            &taffy_tree,
            &node_id_to_taffy,
            &entity_highlighted_spans,
            &mut content_buffer,
            &mut styles_buffer,
            &mut additional_tailwind_classes,
        );

        // Generate CSS from tailwind classes (escaping underscores in brackets for
        // encre-css)
        let escaped_classes: Vec<String> = ir_diagram
            .tailwind_classes
            .values()
            .chain(additional_tailwind_classes.iter())
            .map(|classes| Self::escape_underscores_in_brackets(classes))
            .collect();
        let tailwind_classes_iter = escaped_classes.iter().map(String::as_str);
        let generated_css =
            encre_css::generate(tailwind_classes_iter, &encre_css::Config::default())
                .replace("&", "&amp;");

        // Build the style content
        let mut style_content =
            String::with_capacity(generated_css.len() + styles_buffer.len() + ir_diagram.css.len());
        style_content.push_str(&generated_css);
        if !styles_buffer.is_empty() {
            if !style_content.is_empty() {
                style_content.push('\n');
            }
            style_content.push_str(&styles_buffer);
        }
        if !ir_diagram.css.is_empty() {
            if !style_content.is_empty() {
                style_content.push('\n');
            }
            style_content.push_str(ir_diagram.css.as_str());
        }

        // Build final SVG
        let mut buffer = String::with_capacity(128 + style_content.len() + content_buffer.len());

        // Start SVG element
        write!(
            buffer,
            "<svg \
                xmlns=\"http://www.w3.org/2000/svg\" \
                width=\"{svg_width}\" \
                height=\"{svg_height}\" \
                class=\"group\"\
            >"
        )
        .unwrap();

        // Add style element first (before content)
        if !style_content.is_empty() {
            write!(buffer, "<style>{style_content}</style>").unwrap();
        }

        // Add content
        buffer.push_str(&content_buffer);

        // Close SVG element
        buffer.push_str("</svg>");

        buffer
    }

    /// Collects information about process nodes and their steps for
    /// y-coordinate calculations.
    ///
    /// Returns a vector of ProcessInfo in the order processes appear in
    /// node_ordering.
    fn process_step_heights_calculate<'id>(
        ir_diagram: &IrDiagram<'id>,
        taffy_tree: &TaffyTree<NodeContext>,
        node_id_to_taffy: &disposition_model_common::Map<
            disposition_ir_model::node::NodeId,
            NodeToTaffyNodeIds,
        >,
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
                let total_height = process_step_ids
                    .iter()
                    .filter_map(|process_step_id| node_id_to_taffy.get(process_step_id))
                    .map(
                        |process_step_taffy_node_ids| match process_step_taffy_node_ids {
                            NodeToTaffyNodeIds::Leaf { text_node_id } => *text_node_id,
                            NodeToTaffyNodeIds::Wrapper {
                                wrapper_node_id,
                                text_node_id: _,
                            } => *wrapper_node_id,
                        },
                    )
                    .filter_map(|taffy_node_id| taffy_tree.layout(taffy_node_id).ok())
                    .map(|layout| layout.size.height.min(layout.content_size.height))
                    .sum::<f32>();

                process_steps_height.push(ProcessStepsHeight {
                    process_id,
                    process_step_ids,
                    total_height,
                });
            });

        process_steps_height
    }

    /// Returns the index of the process in the `process_steps_height` list,
    /// or `None` if not found.
    ///
    /// Finds the process that a given node belongs to (if it's a process or
    /// process step).
    fn process_steps_height_index(
        node_id: &NodeId<'_>,
        ir_diagram: &IrDiagram,
        process_steps_height: &[ProcessStepsHeight],
    ) -> Option<usize> {
        let entity_types = ir_diagram.entity_types.get(node_id.as_ref());

        let is_process = entity_types
            .map(|types| types.contains(&EntityType::ProcessDefault))
            .unwrap_or(false);

        let is_process_step = entity_types
            .map(|types| types.contains(&EntityType::ProcessStepDefault))
            .unwrap_or(false);

        if is_process {
            // Find this process in the list
            process_steps_height
                .iter()
                .position(|p| &p.process_id == node_id)
        } else if is_process_step {
            // Find which process this step belongs to
            process_steps_height
                .iter()
                .position(|p| p.process_step_ids.contains(node_id))
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

    /// Builds the y-translation tailwind classes for a process or process step
    /// node.
    ///
    /// This creates:
    /// 1. A base translate-y class for the collapsed state
    /// 2. group-has-[#id:focus-within]:translate-y-[...] classes for when
    ///    previous processes are focused
    fn build_y_translate_classes(
        taffy_y: f32,
        process_index: usize,
        process_steps_height: &[ProcessStepsHeight],
    ) -> String {
        let mut classes = String::new();

        // Calculate the cumulative height of all previous processes' steps
        let process_steps_height_predecessors_cumulative =
            Self::process_steps_height_predecessors_cumulative(process_steps_height, process_index);

        // Base y position (collapsed state): taffy_y minus all previous steps' heights
        let base_y = taffy_y - process_steps_height_predecessors_cumulative;

        // Add transition class for smooth animation
        writeln!(&mut classes, "transition-transform").unwrap();
        writeln!(&mut classes, "duration-300").unwrap();

        // Base translate-y for collapsed state
        writeln!(&mut classes, "translate-y-[{base_y}px]").unwrap();

        // For each previous process, add a class that moves this node down when that
        // process is focused
        (0..process_index).for_each(|prev_idx| {
            let process_steps_height_prev = &process_steps_height[prev_idx];
            let ProcessStepsHeight { process_id, process_step_ids, total_height } = process_steps_height_prev;

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
            process_step_ids
                .iter()
                .for_each(|process_step_id| {
                    writeln!(
                        &mut classes,
                        "group-has-[#{process_step_id}:focus-within]:translate-y-[{y_when_prev_focused}px]"
                    )
                    .unwrap();
                });
        });

        classes
    }

    fn render_nodes(
        ir_diagram: &IrDiagram,
        taffy_tree: &TaffyTree<NodeContext>,
        node_id_to_taffy: &disposition_model_common::Map<
            disposition_ir_model::node::NodeId,
            NodeToTaffyNodeIds,
        >,
        entity_highlighted_spans: &EntityHighlightedSpans,
        buffer: &mut String,
        styles_buffer: &mut String,
        additional_tailwind_classes: &mut Vec<String>,
    ) {
        // First, collect process information for y-coordinate calculations
        let process_steps_heights =
            Self::process_step_heights_calculate(ir_diagram, taffy_tree, node_id_to_taffy);

        ir_diagram
            .node_ordering
            .iter()
            .for_each(|(node_id, &tab_index)| {
                // Look up taffy layout for this node
                let Some(taffy_node_ids) = node_id_to_taffy.get(node_id).copied() else {
                    return;
                };
                let taffy_node_id = match taffy_node_ids {
                    NodeToTaffyNodeIds::Leaf { text_node_id } => text_node_id,
                    NodeToTaffyNodeIds::Wrapper {
                        wrapper_node_id,
                        text_node_id: _,
                    } => wrapper_node_id,
                };
                let Ok(layout) = taffy_tree.layout(taffy_node_id) else {
                    return;
                };

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

                // Check if this is a process or process step node
                let process_index =
                    Self::process_steps_height_index(node_id, ir_diagram, &process_steps_heights);

                // Build additional y-translate classes for process/step nodes
                let y_translate_classes = process_index
                    .map(|idx| Self::build_y_translate_classes(y, idx, &process_steps_heights))
                    .unwrap_or_default();

                // Collect y-translate classes for CSS generation
                if !y_translate_classes.is_empty() {
                    additional_tailwind_classes.push(y_translate_classes.clone());
                }

                let width = layout.size.width;
                let height = {
                    let mut node_height = layout.size.height.min(layout.content_size.height);

                    // If this is a process, subtract the height of its process steps.
                    if let Some(process_steps_height) =
                        process_index.map(|process_index| &process_steps_heights[process_index])
                    {
                        node_height -= process_steps_height.total_height;
                    }

                    node_height
                };

                // Build class attribute combining existing tailwind classes and y-translate
                // classes
                let class_attr = {
                    let existing_classes = ir_diagram
                        .tailwind_classes
                        .get(node_id.as_ref())
                        .map(|s| s.as_str())
                        .unwrap_or("");

                    let combined = if y_translate_classes.is_empty() {
                        existing_classes.to_string()
                    } else if existing_classes.is_empty() {
                        y_translate_classes
                    } else {
                        format!("{existing_classes}\n{y_translate_classes}")
                    };

                    if combined.is_empty() {
                        String::new()
                    } else {
                        let mut classes_str = String::with_capacity(combined.len() + 25);
                        classes_str.push_str(r#" class=""#);
                        combined.chars().for_each(|c| {
                            if c == '&' {
                                classes_str.push_str("&amp;");
                            } else {
                                classes_str.push(c);
                            }
                        });
                        classes_str.push('"');
                        classes_str
                    }
                };

                // Start group element with id, tabindex, and optional class
                write!(
                    buffer,
                    r#"<g id="{node_id}"{class_attr} tabindex="{tab_index}">"#
                )
                .unwrap();

                // Add transform style for positioning
                //
                // For process/step nodes, only use translateX; y is handled by tailwind classes
                // For other nodes, use the full translate.
                if process_index.is_some() {
                    writeln!(
                        styles_buffer,
                        "#{node_id} {{ transform: translateX({x}px); }}"
                    )
                    .unwrap();
                } else {
                    writeln!(
                        styles_buffer,
                        "#{node_id} {{ transform: translate({x}px, {y}px); }}"
                    )
                    .unwrap();
                }

                // Add rect element
                write!(buffer, r#"<rect width="{width}" height="{height}"/>"#).unwrap();

                // Add text elements for highlighted spans if they exist
                if let Some(spans) = entity_highlighted_spans.get(node_id.as_ref()) {
                    for span in spans {
                        let text_x = span.x;
                        let text_y = span.y;
                        // let r = span.style.foreground.r;
                        // let g = span.style.foreground.g;
                        // let b = span.style.foreground.b;
                        // let fill_color = format!("#{r:02x}{g:02x}{b:02x}");
                        let text_content = Self::escape_xml(&span.text);

                        // zero stroke-width because we want the tailwind classes from `<g>` to
                        // apply to the `<rect>`, but not to the `<text>`
                        write!(
                            buffer,
                            "<text \
                                x=\"{text_x}\" \
                                y=\"{text_y}\" \
                                stroke-width=\"0\" \
                            >{text_content}</text>"
                        )
                        .unwrap();
                    }
                }

                // Close group element
                buffer.push_str("</g>");
            });
    }

    /// Escapes underscores within ID selectors inside arbitrary variant
    /// brackets (`[...]`) in a tailwind class string.
    ///
    /// This is needed because encre-css interprets underscores as spaces within
    /// arbitrary variants. By replacing `_` with `&#95;` inside ID selectors
    /// (e.g. `#some_id`), we preserve the literal underscore in the generated
    /// CSS.
    ///
    /// Only underscores that are part of an ID selector (starting with `#`) are
    /// escaped. For example:
    /// - `group-has-[#some_id:focus]` → `group-has-[#some&#95;id:focus]`
    /// - `peer/some-peer:animate-[animation-name_2s_linear_infinite]` →
    ///   unchanged
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use disposition_input_ir_rt::TaffyToSvgMapper;
    /// // ID selectors have underscores escaped
    /// assert_eq!(
    ///     TaffyToSvgMapper::escape_underscores_in_brackets(
    ///         "group-has-[#some_id:focus]:stroke-blue-500"
    ///     ),
    ///     "group-has-[#some&#95;id:focus]:stroke-blue-500"
    /// );
    ///
    /// // Multiple underscores in ID
    /// assert_eq!(
    ///     TaffyToSvgMapper::escape_underscores_in_brackets(
    ///         "group-has-[#my_element_id:hover]:fill-red-500"
    ///     ),
    ///     "group-has-[#my&#95;element&#95;id:hover]:fill-red-500"
    /// );
    ///
    /// // Animation values are NOT escaped (no ID selector)
    /// assert_eq!(
    ///     TaffyToSvgMapper::escape_underscores_in_brackets(
    ///         "peer/some-peer:animate-[animation-name_2s_linear_infinite]"
    ///     ),
    ///     "peer/some-peer:animate-[animation-name_2s_linear_infinite]"
    /// );
    ///
    /// // Mixed: ID escaped, non-ID not escaped
    /// assert_eq!(
    ///     TaffyToSvgMapper::escape_underscores_in_brackets(
    ///         "group-has-[#some_id:focus]:animate-[fade_in_1s]"
    ///     ),
    ///     "group-has-[#some&#95;id:focus]:animate-[fade_in_1s]"
    /// );
    ///
    /// // No brackets - unchanged
    /// assert_eq!(
    ///     TaffyToSvgMapper::escape_underscores_in_brackets("text_red-500"),
    ///     "text_red-500"
    /// );
    /// ```
    pub fn escape_underscores_in_brackets(classes: &str) -> String {
        let mut bracket_depth: u32 = 0;
        let mut is_parsing_id = false;

        classes
            .chars()
            .fold(String::with_capacity(classes.len()), |mut result, c| {
                match c {
                    '[' => {
                        bracket_depth += 1;
                        is_parsing_id = false;
                        result.push(c);
                    }
                    ']' => {
                        bracket_depth = bracket_depth.saturating_sub(1);
                        is_parsing_id = false;
                        result.push(c);
                    }
                    '#' if bracket_depth > 0 => {
                        is_parsing_id = true;
                        result.push(c);
                    }
                    '_' if bracket_depth > 0 && is_parsing_id => {
                        result.push_str("&#95;");
                    }
                    // Characters that end an ID context (not valid in CSS IDs)
                    ':' | ' ' | ',' | '.' | '>' | '+' | '~' | '(' | ')' if is_parsing_id => {
                        is_parsing_id = false;
                        result.push(c);
                    }
                    _ => {
                        result.push(c);
                    }
                }
                result
            })
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
