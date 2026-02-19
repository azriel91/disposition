use std::fmt::Write;

use base64::{prelude::BASE64_STANDARD, Engine};
use disposition_ir_model::entity::EntityTailwindClasses;
use disposition_svg_model::{SvgEdgeInfo, SvgElements, SvgNodeInfo};
use disposition_taffy_model::{TEXT_FONT_SIZE, TEXT_LINE_HEIGHT};

use crate::NOTO_SANS_MONO_TTF;

#[derive(Clone, Copy, Debug)]
pub struct SvgElementsToSvgMapper;

impl SvgElementsToSvgMapper {
    /// Renders the SVG elements to a string.
    pub fn map(svg_elements: &SvgElements) -> String {
        let SvgElements {
            svg_width,
            svg_height,
            svg_node_infos,
            svg_edge_infos,
            svg_process_infos: _,
            tailwind_classes,
            css,
        } = svg_elements;

        let mut content_buffer = String::with_capacity(4096);
        let mut styles_buffer = String::with_capacity(2048);

        // Add default text styles
        writeln!(
            &mut styles_buffer,
            "text {{ \
                font-family: 'Noto Sans Mono', \
                    ui-monospace, \
                    SFMono-Regular, \
                    Menlo, \
                    Monaco, \
                    Consolas, \
                    'Liberation Mono', \
                    monospace; \
                font-size: {TEXT_FONT_SIZE}px; \
                line-height: {TEXT_LINE_HEIGHT}px; \
            }}"
        )
        .unwrap();

        // Add default font
        writeln!(
            &mut styles_buffer,
            "@font-face {{ \
                font-family: 'Noto Sans Mono'; \
                src: url(data:application/x-font-ttf;base64,{}) format('truetype'); \
            }}",
            BASE64_STANDARD.encode(NOTO_SANS_MONO_TTF)
        )
        .unwrap();

        // Render nodes
        Self::render_nodes(&mut content_buffer, svg_node_infos, tailwind_classes);

        // Render edges
        Self::render_edges(&mut content_buffer, svg_edge_infos, tailwind_classes);

        // Generate CSS from tailwind classes
        //
        // We also need to escape underscores in brackets for correct tailwind class
        // generation.
        let escaped_classes: Vec<String> = tailwind_classes
            .values()
            .map(|classes| Self::escape_ids_in_brackets(classes))
            .collect();
        let tailwind_classes_iter = escaped_classes.iter().map(String::as_str).chain(
            svg_node_infos
                .iter()
                .flat_map(|svg_node_info| svg_node_info.wrapper_tailwind_classes.iter())
                .map(|wrapper_tailwind_classes| wrapper_tailwind_classes.as_ref()),
        );
        let generated_css =
            encre_css::generate(tailwind_classes_iter, &encre_css::Config::default())
                .replace("&", "&amp;");

        // Build the style content
        let mut style_content =
            String::with_capacity(generated_css.len() + styles_buffer.len() + css.len());
        style_content.push_str(&generated_css);
        if !styles_buffer.is_empty() {
            if !style_content.is_empty() {
                style_content.push('\n');
            }
            style_content.push_str(&styles_buffer);
        }
        if !css.is_empty() {
            if !style_content.is_empty() {
                style_content.push('\n');
            }
            style_content.push_str(css.as_str());
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

    /// Writes nodes to the SVG content buffer.
    ///
    /// These will be rendered as the following elements (some attributes
    /// omitted):
    ///
    /// ```svg
    /// <g id="{node_id}" ..>
    ///   <!-- background rectangle -->
    ///   <path d="{path_d}" .. />
    ///
    ///   <!-- node text -->
    ///   <text .. >{line_1}</text>
    ///   <text .. >{line_2}</text>
    /// </g>
    /// ```
    fn render_nodes(
        content_buffer: &mut String,
        svg_node_infos: &[SvgNodeInfo<'_>],
        tailwind_classes: &EntityTailwindClasses<'_>,
    ) {
        svg_node_infos.iter().for_each(|svg_node_info| {
            let node_id = &svg_node_info.node_id;
            let tab_index = svg_node_info.tab_index;
            let path_d = &svg_node_info.path_d_collapsed;

            let class_attr = {
                let tailwind_classes = tailwind_classes
                    .get(node_id.as_ref())
                    .cloned()
                    .unwrap_or_default();

                Self::class_attr_escaped(tailwind_classes)
            };

            // Start group element with id, tabindex, and optional class
            write!(
                content_buffer,
                r#"<g id="{node_id}"{class_attr} tabindex="{tab_index}">"#
            )
            .unwrap();

            // Add path element with corner radii.
            // If a circle is present, apply wrapper_tailwind_classes to make the
            // rect path invisible, and render the circle path separately.
            match svg_node_info.wrapper_tailwind_classes.as_ref() {
                Some(wrapper_tw) => {
                    write!(
                        content_buffer,
                        r#"<path d="{path_d}" class="{wrapper_tw}" />"#
                    )
                    .unwrap();
                }
                None => {
                    write!(content_buffer, r#"<path d="{path_d}" />"#).unwrap();
                }
            }

            // Add circle path element if present
            if let Some(ref circle) = svg_node_info.circle {
                let circle_path_d = &circle.path_d;
                write!(content_buffer, r#"<path d="{circle_path_d}" />"#).unwrap();
            }

            // Add text elements for highlighted spans
            svg_node_info.text_spans.iter().for_each(|span| {
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
            });

            // Close group element
            content_buffer.push_str("</g>");
        });
    }

    /// Writes edges to the SVG content buffer.
    ///
    /// These will be rendered as the following elements (some attributes
    /// omitted):
    ///
    /// ```svg
    /// <g id="{edge_id}" .. >
    ///   <path d="{path_d}" .. />
    ///   <g class="arrow_head" .. >
    ///     <path d="{arrow_head_path_d}" .. />
    ///   </g>
    /// </g>
    /// ```
    ///
    /// For interaction edges the arrowhead `<path>` also carries
    /// `offset-path`, `offset-rotate`, and animation tailwind classes
    /// looked up via the `{edge_id}__arrow_head` entity key.
    fn render_edges(
        content_buffer: &mut String,
        svg_edge_infos: &[SvgEdgeInfo<'_>],
        tailwind_classes: &EntityTailwindClasses<'_>,
    ) {
        svg_edge_infos.iter().for_each(|svg_edge_info| {
            let edge_id = &svg_edge_info.edge_id;
            let edge_group_id = &svg_edge_info.edge_group_id;
            let path_d = &svg_edge_info.path_d;
            let arrow_head_path_d = &svg_edge_info.arrow_head_path_d;

            // Build class attribute from tailwind_classes for the edge
            // First check for edge-specific classes, then fall back to edge group classes
            let class_attr = {
                let edge_classes = tailwind_classes
                    .get(edge_id.as_ref())
                    .map(|s| s.as_str())
                    .unwrap_or("");

                let edge_group_classes = tailwind_classes
                    .get(edge_group_id.as_ref())
                    .map(|s| s.as_str())
                    .unwrap_or("");

                let combined = if edge_classes.is_empty() {
                    edge_group_classes.to_string()
                } else if edge_group_classes.is_empty() {
                    edge_classes.to_string()
                } else {
                    format!("{edge_group_classes}\n{edge_classes}")
                };

                Self::class_attr_escaped(combined)
            };

            // Build class attribute for the arrowhead element.
            //
            // For interaction edges the builder stores offset-path and
            // animation tailwind classes under the key
            // `{edge_id}__arrow_head`.  For dependency edges no such entry
            // exists, so we fall back to a plain `arrow_head` class.
            let arrow_head_entity_key = format!("{edge_id}_arrow_head");
            let arrow_head_class_attr = if let Ok(arrow_head_id) =
                disposition_model_common::Id::try_from(arrow_head_entity_key)
            {
                let extra = tailwind_classes
                    .get(&arrow_head_id)
                    .map(|s| s.as_str())
                    .unwrap_or("");
                if extra.is_empty() {
                    Self::class_attr_escaped("arrow_head".to_string())
                } else {
                    Self::class_attr_escaped(format!("arrow_head\n{extra}"))
                }
            } else {
                Self::class_attr_escaped("arrow_head".to_string())
            };

            // Render edge as a group with a path and an arrowhead path
            //
            // The edge path has fill="none" since edges are stroked lines,
            // not filled shapes.  The arrowhead is a closed V-shape that
            // inherits stroke/fill from the <g>.
            write!(
                content_buffer,
                "<g \
                    id=\"{edge_id}\"\
                    {class_attr}\
                >\
                    <path \
                        d=\"{path_d}\" \
                        fill=\"none\" \
                    />\
                    <g \
                        {arrow_head_class_attr} \
                    >\
                        <path \
                            d=\"{arrow_head_path_d}\" \
                        />\
                    </g>
                </g>"
            )
            .unwrap();
        });
    }

    /// Returns the `class=".."` attribute with `&` escaped as `&amp;`.
    fn class_attr_escaped(tailwind_classes: String) -> String {
        if tailwind_classes.is_empty() {
            String::new()
        } else {
            let ampersand_count = tailwind_classes.matches('&').count();
            let mut classes_str =
                String::with_capacity(tailwind_classes.len() + ampersand_count * 5 + 10);
            classes_str.push_str(r#" class=""#);
            tailwind_classes.chars().for_each(|c| {
                if c == '&' {
                    classes_str.push_str("&amp;");
                } else {
                    classes_str.push(c);
                }
            });
            classes_str.push('"');
            classes_str
        }
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
    /// # use disposition_input_ir_rt::SvgElementsToSvgMapper;
    /// // ID selectors have underscores escaped
    /// assert_eq!(
    ///     SvgElementsToSvgMapper::escape_ids_in_brackets(
    ///         "group-has-[#some_id:focus]:stroke-blue-500"
    ///     ),
    ///     "group-has-[#some&#95;id:focus]:stroke-blue-500"
    /// );
    ///
    /// // Multiple underscores in ID
    /// assert_eq!(
    ///     SvgElementsToSvgMapper::escape_ids_in_brackets(
    ///         "group-has-[#my_element_id:hover]:fill-red-500"
    ///     ),
    ///     "group-has-[#my&#95;element&#95;id:hover]:fill-red-500"
    /// );
    ///
    /// // Animation values are NOT escaped (no ID selector)
    /// assert_eq!(
    ///     SvgElementsToSvgMapper::escape_ids_in_brackets(
    ///         "peer/some-peer:animate-[animation-name_2s_linear_infinite]"
    ///     ),
    ///     "peer/some-peer:animate-[animation-name_2s_linear_infinite]"
    /// );
    ///
    /// // Mixed: ID escaped, non-ID not escaped
    /// assert_eq!(
    ///     SvgElementsToSvgMapper::escape_ids_in_brackets(
    ///         "group-has-[#some_id:focus]:animate-[fade_in_1s]"
    ///     ),
    ///     "group-has-[#some&#95;id:focus]:animate-[fade_in_1s]"
    /// );
    ///
    /// // No brackets - unchanged
    /// assert_eq!(
    ///     SvgElementsToSvgMapper::escape_ids_in_brackets("text_red-500"),
    ///     "text_red-500"
    /// );
    /// ```
    pub fn escape_ids_in_brackets(classes: &str) -> String {
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
