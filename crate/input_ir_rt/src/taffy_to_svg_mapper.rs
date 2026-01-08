use std::fmt::Write;

use base64::{prelude::BASE64_STANDARD, Engine};
use disposition_ir_model::{node::NodeInbuilt, IrDiagram};
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
        );

        // Generate CSS from tailwind classes
        let tailwind_classes_iter = ir_diagram.tailwind_classes.values().map(String::as_str);
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
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{svg_width}" height="{svg_height}">"#
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
    ) {
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
                        x_acc += parent_layout.content_box_x();
                        y_acc += parent_layout.content_box_y();
                        current_node_id = parent_taffy_node_id;
                    }
                    (x_acc, y_acc)
                };
                let width = layout.size.width;
                let height = layout.size.height;

                let node_id_str = node_id.as_str();

                // Build class attribute if tailwind classes exist
                let class_attr = ir_diagram
                    .tailwind_classes
                    .get(node_id.as_ref())
                    .map(|classes| {
                        let mut classes_str = String::with_capacity(classes.len() + 25);
                        classes_str.push_str(r#" class=""#);
                        classes.chars().for_each(|c| {
                            if c == '&' {
                                classes_str.push_str("&amp;");
                            } else {
                                classes_str.push(c);
                            }
                        });
                        classes_str.push('"');
                        classes_str
                    })
                    .unwrap_or_default();

                // Start group element with id, tabindex, and optional class
                write!(
                    buffer,
                    r#"<g id="{node_id_str}"{class_attr} tabindex="{tab_index}">"#
                )
                .unwrap();

                // Add transform style for positioning
                writeln!(
                    styles_buffer,
                    "#{node_id_str} {{ transform: translate({x}px, {y}px); }}"
                )
                .unwrap();

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

    /// Escape XML special characters in text content
    fn escape_xml(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        for c in s.chars() {
            match c {
                '&' => result.push_str("&amp;"),
                '<' => result.push_str("&lt;"),
                '>' => result.push_str("&gt;"),
                '"' => result.push_str("&quot;"),
                '\'' => result.push_str("&apos;"),
                _ => result.push(c),
            }
        }
        result
    }
}
