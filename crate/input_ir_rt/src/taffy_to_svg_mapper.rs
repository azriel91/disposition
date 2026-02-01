use std::fmt::Write;

use base64::{prelude::BASE64_STANDARD, Engine};
use disposition_ir_model::IrDiagram;
use disposition_svg_model::SvgElements;
use disposition_taffy_model::{TaffyNodeMappings, TEXT_FONT_SIZE, TEXT_LINE_HEIGHT};

use crate::{TaffyToSvgElementsMapper, NOTO_SANS_MONO_TTF};

#[derive(Clone, Copy, Debug)]
pub struct TaffyToSvgMapper;

impl TaffyToSvgMapper {
    pub fn map(ir_diagram: &IrDiagram, taffy_node_mappings: TaffyNodeMappings) -> String {
        // First, compute the SVG elements
        let svg_elements = TaffyToSvgElementsMapper::map(ir_diagram, &taffy_node_mappings);

        // Then render to string
        Self::render(ir_diagram, &svg_elements)
    }

    /// Renders the SVG elements to a string.
    ///
    /// This is separate from `map` to allow users to modify the `SvgElements`
    /// before rendering (e.g., to add edges).
    pub fn render(ir_diagram: &IrDiagram, svg_elements: &SvgElements) -> String {
        let SvgElements {
            svg_width,
            svg_height,
            svg_node_infos,
            svg_edge_infos: _,
            svg_process_infos: _,
            additional_tailwind_classes,
        } = svg_elements;

        let mut content_buffer = String::with_capacity(4096);
        let mut styles_buffer = String::with_capacity(2048);

        // Add default text styles
        writeln!(&mut styles_buffer, "text {{ font-family: 'Noto Sans Mono', ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', monospace; font-size: {TEXT_FONT_SIZE}px; line-height: {TEXT_LINE_HEIGHT}px; }}").unwrap();

        // Add default font
        writeln!(&mut styles_buffer, "@font-face {{ font-family: 'Noto Sans Mono'; src: url(data:application/x-font-ttf;base64,{}) format('truetype'); }}", BASE64_STANDARD.encode(NOTO_SANS_MONO_TTF)).unwrap();

        // Render nodes
        for svg_node_info in svg_node_infos {
            let node_id = &svg_node_info.node_id;
            let tab_index = svg_node_info.tab_index;
            let path_d = &svg_node_info.path_d_collapsed;

            // Build class attribute combining existing tailwind classes and translate
            // classes from additional_tailwind_classes (index corresponds to node order)
            let node_index = svg_node_infos
                .iter()
                .position(|n| n.node_id == *node_id)
                .unwrap_or(0);
            let translate_classes = additional_tailwind_classes
                .get(node_index)
                .map(|s| s.as_str())
                .unwrap_or("");

            let class_attr = {
                let existing_classes = ir_diagram
                    .tailwind_classes
                    .get(node_id.as_ref())
                    .map(|s| s.as_str())
                    .unwrap_or("");

                let combined = if existing_classes.is_empty() {
                    translate_classes.to_string()
                } else {
                    format!("{existing_classes}\n{translate_classes}")
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
                content_buffer,
                r#"<g id="{node_id}"{class_attr} tabindex="{tab_index}">"#
            )
            .unwrap();

            // Add path element with corner radii
            write!(content_buffer, r#"<path d="{path_d}" />"#).unwrap();

            // Add text elements for highlighted spans
            for span in &svg_node_info.text_spans {
                let text_x = span.x;
                let text_y = span.y;
                let text_content = &span.text;

                // zero stroke-width because we want the tailwind classes from `<g>` to
                // apply to the `<path>`, but not to the `<text>`
                write!(
                    content_buffer,
                    "<text \
                        x=\"{text_x}\" \
                        y=\"{text_y}\" \
                        stroke-width=\"0\" \
                    >{text_content}</text>"
                )
                .unwrap();
            }

            // Close group element
            content_buffer.push_str("</g>");
        }

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
                // https://docs.rs/encre-css/latest/encre_css/plugins/typography/content/index.html
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
                    '"' if bracket_depth > 0 => {
                        result.push_str("&#34;");
                    }
                    '\'' if bracket_depth > 0 => {
                        result.push_str("&#39;");
                    }
                    '(' if bracket_depth > 0 => {
                        result.push_str("&#40;");
                    }
                    ')' if bracket_depth > 0 => {
                        result.push_str("&#41;");
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
}
