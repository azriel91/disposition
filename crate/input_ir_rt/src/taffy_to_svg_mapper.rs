use std::fmt::Write;

use disposition_ir_model::{
    node::{NodeHierarchy, NodeInbuilt},
    IrDiagram,
};
use disposition_taffy_model::{EntityHighlightedSpans, NodeContext, TaffyNodeMappings};
use taffy::TaffyTree;

#[derive(Clone, Copy, Debug)]
pub struct TaffyToSvgMapper;

impl TaffyToSvgMapper {
    pub fn map(ir_diagram: &IrDiagram, taffy_node_mappings: TaffyNodeMappings) -> String {
        let TaffyNodeMappings {
            taffy_tree,
            node_inbuilt_to_taffy,
            node_id_to_taffy,
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

        // Recursively render node hierarchy
        Self::render_node_hierarchy(
            &ir_diagram.node_hierarchy,
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

    fn render_node_hierarchy(
        hierarchy: &NodeHierarchy,
        ir_diagram: &IrDiagram,
        taffy_tree: &TaffyTree<NodeContext>,
        node_id_to_taffy: &disposition_model_common::Map<
            disposition_ir_model::node::NodeId,
            taffy::NodeId,
        >,
        entity_highlighted_spans: &EntityHighlightedSpans,
        buffer: &mut String,
        styles_buffer: &mut String,
    ) {
        for (node_id, children) in hierarchy.iter() {
            // Look up taffy layout for this node
            let Some(&taffy_node_id) = node_id_to_taffy.get(node_id) else {
                continue;
            };
            let Ok(layout) = taffy_tree.layout(taffy_node_id) else {
                continue;
            };

            let x = layout.location.x;
            let y = layout.location.y;
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
                    classes_str.push_str("\"");
                    classes_str
                })
                .unwrap_or_default();

            // Start group element with id, tabindex, and optional class
            write!(buffer, r#"<g id="{node_id_str}"{class_attr} tabindex="0">"#).unwrap();

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
                    let r = span.style.foreground.r;
                    let g = span.style.foreground.g;
                    let b = span.style.foreground.b;
                    let fill_color = format!("#{r:02x}{g:02x}{b:02x}");
                    let text_content = Self::escape_xml(&span.text);

                    write!(
                        buffer,
                        r#"<text x="{text_x}" y="{text_y}" fill="{fill_color}">{text_content}</text>"#
                    )
                    .unwrap();
                }
            }

            // Recursively render children
            if !children.is_empty() {
                Self::render_node_hierarchy(
                    children,
                    ir_diagram,
                    taffy_tree,
                    node_id_to_taffy,
                    entity_highlighted_spans,
                    buffer,
                    styles_buffer,
                );
            }

            // Close group element
            buffer.push_str("</g>");
        }
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
