use std::fmt::Write;

use crate::string_xml_escaper::StringXmlEscaper;

use base64::{prelude::BASE64_STANDARD, Engine};
use disposition_input_model::InputDiagram;
use disposition_ir_model::entity::EntityTailwindClasses;
use disposition_svg_model::{
    SvgEdgeDescriptionInfo, SvgEdgeInfo, SvgEdgeLabelInfo, SvgElements, SvgNodeInfo,
};
use disposition_taffy_model::{
    MD_BLOCKQUOTE_BORDER_COLOR, MD_CODE_BG_COLOR, TEXT_FONT_SIZE, TEXT_LINE_HEIGHT,
};

use crate::NOTO_SANS_MONO_TTF;

/// Pixels to shift the inline-code background box down from the text top so its
/// bottom edge sits below the baseline and covers glyph descenders (g, p, y).
pub(crate) const CODE_BG_DESCENT_OFFSET: f32 = 3.0;

/// Corner radius (pixels) of the inline-code background box.
const CODE_BG_CORNER_RADIUS: f32 = 3.0;

/// Corner radius (pixels) of the background box drawn behind interaction
/// edge label and description text.
const INTERACTION_EDGE_BG_CORNER_RADIUS: f32 = 4.0;

/// Width (pixels) of the blockquote's left bar (its thick left border). Kept in
/// sync with `MdNodeBuilder::BLOCKQUOTE_BAR_WIDTH` so the bar sits in the
/// container's left padding without overlapping the text.
const BLOCKQUOTE_BORDER_LEFT: f32 = 7.0;

/// Width (pixels) of the blockquote's thin top / right / bottom borders.
const BLOCKQUOTE_BORDER_THIN: f32 = 1.0;

/// Corner radius (pixels) of the blockquote frame's outer rectangle. The inner
/// rectangle uses `BLOCKQUOTE_BORDER_RADIUS - BLOCKQUOTE_BORDER_THIN`.
const BLOCKQUOTE_BORDER_RADIUS: f32 = 2.0;

/// CSS variables to ensure CSS works correctly with `encre-css`.
///
/// These are copied from `DEFAULT_PREFLIGHT` in [`encre-css/src/preflight.rs`].
///
/// We use inline `<styles>` in an SVG, but when the SVG is embedded directly
/// into an HTML DOM, those styles leak to the global scope. `encre-css`'s
/// default `Preflight::Full` variant includes styling that resets all elements'
/// styles, which can break web pages.
///
/// However, for `encre-css`'s generated styles to work, the variables it
/// defines must still be present in the document, otherwise the styles (e.g.
/// transform) won't be applied.
///
/// [`encre-css/src/preflight.rs`]: https://gitlab.com/encre-org/encre-css/-/blob/v0.21.2/crates/encre-css/src/preflight.rs?ref_type=tags#L369-416
const ENCRE_CSS_VARIABLES: &str = "svg {
    --en-border-spacing-x: 0;
    --en-border-spacing-y: 0;
    --en-translate-x: 0;
    --en-translate-y: 0;
    --en-translate-z: 0;
    --en-rotate-x: 0;
    --en-rotate-y: 0;
    --en-rotate-z: 0;
    --en-skew-x: 0;
    --en-skew-y: 0;
    --en-scale-x: 1;
    --en-scale-y: 1;
    --en-scale-z: 1;
    --en-pan-x: ;
    --en-pan-y: ;
    --en-pinch-zoom: ;
    --en-scroll-snap-strictness: proximity;
    --en-ordinal: ;
    --en-slashed-zero: ;
    --en-numeric-figure: ;
    --en-numeric-spacing: ;
    --en-numeric-fraction: ;
    --en-ring-inset: ;
    --en-ring-offset-width: 0px;
    --en-ring-offset-color: #fff;
    --en-ring-color: currentColor;
    --en-ring-offset-shadow: 0 0 #0000;
    --en-ring-shadow: 0 0 #0000;
    --en-shadow: 0 0 #0000;
    --en-shadow-colored: 0 0 #0000;
    --en-blur: ;
    --en-brightness: ;
    --en-contrast: ;
    --en-grayscale: ;
    --en-hue-rotate: ;
    --en-invert: ;
    --en-saturate: ;
    --en-sepia: ;
    --en-drop-shadow: ;
    --en-backdrop-blur: ;
    --en-backdrop-brightness: ;
    --en-backdrop-contrast: ;
    --en-backdrop-grayscale: ;
    --en-backdrop-hue-rotate: ;
    --en-backdrop-invert: ;
    --en-backdrop-opacity: ;
    --en-backdrop-saturate: ;
    --en-backdrop-sepia: ;
}";

#[derive(Clone, Copy, Debug)]
pub struct SvgElementsToSvgMapper;

impl SvgElementsToSvgMapper {
    /// Renders the SVG elements to a string.
    ///
    /// See [`Self::map_with_input`] if you want the `InputDiagram` source to be
    /// included as well.
    pub fn map(svg_elements: &SvgElements) -> String {
        let mut buffer = String::new();
        Self::map_svg(&mut buffer, svg_elements, None);
        buffer
    }

    /// Renders the SVG elements to a string, prepended with an XML declaration
    /// and a brief XML comment, with the source `input_diagram` serialized as
    /// YAML inside a `<source><![CDATA[...]]></source>` element within the SVG.
    ///
    /// The output follows the format:
    ///
    /// ```xml
    /// <?xml version="1.0" encoding="UTF-8"?>
    /// <!--
    ///     This diagram was generated using `disposition` on `2026-05-13 06:15:00.000+13:00`.
    ///
    ///     See <https://azriel.im/disposition>.
    /// -->
    /// <svg xmlns="http://www.w3.org/2000/svg" ...>
    ///   <source><![CDATA[---
    /// things:
    ///   t_alice: Alice
    /// ]]></source>
    ///   <!-- .. -->
    /// </svg>
    /// ```
    ///
    /// # Notes
    ///
    /// - The only sequence that would break a CDATA section (`]]>`) is escaped
    ///   by splitting it across two adjacent CDATA sections: `]]]]><![CDATA[>`.
    /// - If `input_diagram` cannot be serialized to YAML, the `<source>`
    ///   element is omitted.
    pub fn map_with_input(input_diagram: &InputDiagram<'_>, svg_elements: &SvgElements) -> String {
        let timestamp = jiff::Zoned::now()
            .strftime("%Y-%m-%d %H:%M:%S%.3f%:z")
            .to_string();

        let yaml = {
            let mut yaml_buffer = String::new();
            let yaml_result = serde_saphyr::to_fmt_writer(&mut yaml_buffer, input_diagram);
            if yaml_result.is_ok() {
                // `]]>` is the only sequence that cannot appear unescaped
                // inside a CDATA section. Split it across two adjacent
                // CDATA sections so the content remains valid.
                if yaml_buffer.contains("]]>") {
                    yaml_buffer.replace("]]>", "]]]]><![CDATA[>")
                } else {
                    yaml_buffer
                }
            } else {
                String::new()
            }
        };

        let source_yaml = if yaml.is_empty() {
            None
        } else {
            Some(yaml.as_str())
        };

        let mut buffer = String::with_capacity(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n".len() + 256 + yaml.len(),
        );

        // XML declaration
        buffer.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");

        // Brief comment with generation info (source YAML goes inside the SVG)
        buffer.push_str("<!--\n");
        writeln!(
            buffer,
            "    This diagram was generated using `disposition` on `{timestamp}`."
        )
        .unwrap();
        buffer.push('\n');
        buffer.push_str("    See <https://azriel.im/disposition>.\n");
        buffer.push_str("-->\n");

        Self::map_svg(&mut buffer, svg_elements, source_yaml);

        buffer
    }

    /// Writes the `<svg>` element to `buffer`.
    ///
    /// If `source_yaml` is `Some`, a `<source><![CDATA[...]]></source>` element
    /// is injected immediately after the opening `<svg ...>` tag, embedding the
    /// YAML source so it can be copied verbatim.
    fn map_svg(buffer: &mut String, svg_elements: &SvgElements, source_yaml: Option<&str>) {
        let SvgElements {
            svg_width,
            svg_height,
            svg_node_infos,
            svg_edge_infos,
            edge_label_infos,
            edge_description_infos,
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

        // Add link styles.
        //
        // The inline-code background fill and the link text colour are now
        // expressed as Tailwind `fill-[var(--tw-...)]` classes (see
        // `MD_CODE_BG_COLOR` / `MD_LINK_COLOR`), generated by `encre_css` and
        // backed by `--tw-...` theme variables that flip with the configured
        // dark-mode selector. Only the cursor affordance remains as bespoke CSS.
        writeln!(&mut styles_buffer, "a {{ cursor: pointer; }}").unwrap();

        // Render nodes
        Self::render_nodes(&mut content_buffer, svg_node_infos, tailwind_classes);

        // Render edges
        Self::render_edges(&mut content_buffer, svg_edge_infos, tailwind_classes);

        // Render edge labels
        Self::render_edge_labels(&mut content_buffer, edge_label_infos, tailwind_classes);

        // Render edge descriptions
        Self::render_edge_descriptions(
            &mut content_buffer,
            edge_description_infos,
            tailwind_classes,
        );

        // Generate CSS from tailwind classes
        //
        // We also need to escape underscores in brackets for correct tailwind class
        // generation.
        let escaped_classes: Vec<String> = tailwind_classes
            .values()
            .map(|classes| Self::escape_ids_in_brackets(classes))
            .collect();
        // Markdown span classes -- the inline-code background fill plus every
        // node, edge-description, and edge-label text-span class. These may
        // contain arbitrary `fill-[var(...)]` values, so escape them the same
        // way as entity classes before handing them to `encre_css`.
        let escaped_md_span_classes: Vec<String> = {
            let code_bg_class = MD_CODE_BG_COLOR.fill_class();
            let blockquote_border_class = MD_BLOCKQUOTE_BORDER_COLOR.fill_class();
            [code_bg_class.as_str(), blockquote_border_class.as_str()]
                .into_iter()
                .chain(
                    svg_node_infos
                        .iter()
                        .flat_map(|svg_node_info| {
                            svg_node_info
                                .text_spans
                                .iter()
                                .flat_map(|text_span| text_span.tailwind_classes.iter())
                        })
                        .map(|class| class.as_str()),
                )
                .chain(
                    edge_description_infos
                        .iter()
                        .flat_map(|edge_desc_info| {
                            edge_desc_info
                                .text_spans
                                .iter()
                                .flat_map(|text_span| text_span.tailwind_classes.iter())
                        })
                        .map(|class| class.as_str()),
                )
                .chain(
                    edge_label_infos
                        .iter()
                        .flat_map(|edge_label_info| {
                            edge_label_info
                                .from_label
                                .iter()
                                .chain(edge_label_info.to_label.iter())
                        })
                        .flat_map(|endpoint_info| {
                            endpoint_info
                                .text_spans
                                .iter()
                                .flat_map(|text_span| text_span.tailwind_classes.iter())
                        })
                        .map(|class| class.as_str()),
                )
                .map(Self::escape_ids_in_brackets)
                .collect()
        };
        let tailwind_classes_iter = escaped_classes
            .iter()
            .map(String::as_str)
            .chain(
                svg_node_infos
                    .iter()
                    .flat_map(|svg_node_info| svg_node_info.wrapper_tailwind_classes.iter())
                    .map(|wrapper_tailwind_classes| wrapper_tailwind_classes.as_ref()),
            )
            .chain(escaped_md_span_classes.iter().map(String::as_str));
        // TODO: generate an ID for the SVG so that the `<styles>` don't leak to outer
        // document.
        let encre_css_config = {
            let mut encre_css_config = encre_css::Config::default();
            encre_css_config.preflight = encre_css::Preflight::new_custom(ENCRE_CSS_VARIABLES);
            encre_css_config
        };
        let generated_css =
            encre_css::generate(tailwind_classes_iter, &encre_css_config).replace("&", "&amp;");

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

        // Reserve capacity for the SVG content before writing.
        let source_len = source_yaml
            .map(|yaml| "<source><![CDATA[".len() + yaml.len() + "]]></source>".len() + 1)
            .unwrap_or(0);
        buffer.reserve(128 + style_content.len() + content_buffer.len() + source_len);

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

        // Embed YAML source in a CDATA section so it can be copied verbatim.
        if let Some(yaml) = source_yaml {
            buffer.push_str("<source><![CDATA[");
            buffer.push_str(yaml);
            // Ensure the YAML ends with a newline before the closing marker.
            if !yaml.ends_with('\n') {
                buffer.push('\n');
            }
            buffer.push_str("]]></source>");
        }

        // Add style element first (before content)
        if !style_content.is_empty() {
            write!(buffer, "<style>{style_content}</style>").unwrap();
        }

        // Add content
        buffer.push_str(&content_buffer);

        // Close SVG element
        buffer.push_str("</svg>");
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
                let node_tailwind_classes = tailwind_classes
                    .get(node_id.as_ref())
                    .map(|node_tailwind_classes| node_tailwind_classes.as_str())
                    .unwrap_or_default();

                Self::class_attr_escaped(node_tailwind_classes)
            };

            // Start group element with id, tabindex, and optional class
            write!(
                content_buffer,
                r#"<g id="{node_id}"{class_attr} tabindex="{tab_index}">"#
            )
            .unwrap();

            // Add tooltip element if present
            if !svg_node_info.tooltip.is_empty() {
                let tooltip_escaped = StringXmlEscaper::escape(&svg_node_info.tooltip);
                write!(content_buffer, "<title>{tooltip_escaped}</title>").unwrap();
            }

            // Add path element with corner radii.
            // If a circle is present, apply wrapper_tailwind_classes to make the
            // rect path invisible, and render the circle path separately.
            write!(content_buffer, r#"<path d="{path_d}" class="wrapper"#).unwrap();
            if let Some(wrapper_tw) = svg_node_info.wrapper_tailwind_classes.as_ref() {
                write!(content_buffer, " {wrapper_tw}").unwrap();
            }
            write!(content_buffer, r#"" />"#).unwrap();

            // Add circle path element if present
            if let Some(ref circle) = svg_node_info.circle {
                let circle_path_d = &circle.path_d;
                write!(content_buffer, r#"<path d="{circle_path_d}" />"#).unwrap();
            }

            // Add text and image elements
            Self::render_text_and_images(
                content_buffer,
                &svg_node_info.text_spans,
                &svg_node_info.image_spans,
            );

            // Close group element
            content_buffer.push_str("</g>");
        });
    }

    /// Writes inline image elements for a node to the SVG content buffer.
    /// Renders text spans with markdown styling and inline images.
    ///
    /// Handles:
    /// - Code background rectangles with `.md-code-bg` class
    /// - Text styling via Tailwind classes (bold, italic, strikethrough,
    ///   headings, links)
    /// - Inline images
    fn render_text_and_images(
        content_buffer: &mut String,
        svg_text_spans: &[disposition_svg_model::SvgTextSpan],
        svg_image_spans: &[disposition_svg_model::SvgImageSpan],
    ) {
        // Add text elements for styled spans
        svg_text_spans.iter().for_each(|svg_text_span| {
            let text_x = svg_text_span.x;
            let text_y = svg_text_span.y;
            let text_content = &svg_text_span.text;

            // Emit a blockquote border frame for blockquote marker spans.
            //
            // The span carries no text; `text_y` is the box bottom and `height`
            // its full height, so the box top is `text_y - height`. The frame
            // has a thick left bar and thin top / right / bottom borders, with
            // slightly rounded corners.
            if svg_text_span
                .md_style
                .as_ref()
                .is_some_and(|svg_md_style| svg_md_style.blockquote)
            {
                let rect_w = svg_text_span.width;
                let rect_h = svg_text_span.height;
                let rect_y = text_y - rect_h;
                let path_d = Self::blockquote_border_path_d(text_x, rect_y, rect_w, rect_h);
                let border_class = MD_BLOCKQUOTE_BORDER_COLOR.fill_class();
                // `stroke-width="0"` so the frame is drawn purely as an
                // even-odd fill: otherwise the `<g>`'s node-border stroke is
                // inherited and outlines both the outer and inner rectangles,
                // doubling the visible top / right / bottom borders.
                write!(
                    content_buffer,
                    "<path d=\"{path_d}\" fill-rule=\"evenodd\" stroke-width=\"0\" \
                        class=\"{border_class}\" />"
                )
                .unwrap();
            }

            // Emit a rounded background path before code spans.
            //
            // `text_y` is the text baseline, so the box top is
            // `text_y - height` and it is shifted down by `CODE_BG_DESCENT_OFFSET`
            // so its bottom clears the baseline and covers glyph descenders.
            if svg_text_span
                .md_style
                .as_ref()
                .is_some_and(|svg_md_style| svg_md_style.code)
            {
                let rect_w = svg_text_span.width;
                let rect_h = svg_text_span.height;
                let rect_y = text_y - rect_h + CODE_BG_DESCENT_OFFSET;
                let path_d =
                    Self::code_bg_path_d(text_x, rect_y, rect_w, rect_h, CODE_BG_CORNER_RADIUS);
                // Background fill comes from a Tailwind `fill-[var(--tw-...)]`
                // class (see `MD_CODE_BG_COLOR`) so it is generated by
                // `encre_css` and flips with the configured dark-mode selector.
                let code_bg_class = MD_CODE_BG_COLOR.fill_class();
                write!(
                    content_buffer,
                    "<path d=\"{path_d}\" class=\"{code_bg_class}\" />"
                )
                .unwrap();
            }

            // Wrap in <a> element if this is a link
            if let Some(link_dest) = svg_text_span
                .md_style
                .as_ref()
                .and_then(|svg_md_style| svg_md_style.link_dest.as_ref())
            {
                write!(
                    content_buffer,
                    "<a href=\"{link_dest}\" target=\"_blank\" rel=\"noopener noreferrer\">",
                )
                .unwrap();
            }

            // Build class attribute from tailwind classes
            let class_attr = if !svg_text_span.tailwind_classes.is_empty() {
                format!(" class=\"{}\"", svg_text_span.tailwind_classes.join(" "))
            } else {
                String::new()
            };

            // Preserve whitespace for markdown spans so the leading spaces that
            // right-align ordered list markers (e.g. `"   i."`) are rendered
            // instead of collapsed. Legacy (non-markdown) spans keep the default.
            let space_attr = if svg_text_span.md_style.is_some() {
                " xml:space=\"preserve\""
            } else {
                ""
            };

            // zero stroke-width because we want the tailwind classes from `<g>` to
            // apply to the `<path>`, but not to the `<text>`
            write!(
                content_buffer,
                "<text x=\"{text_x}\" y=\"{text_y}\" stroke-width=\"0\"{space_attr}{class_attr}>\
                    {text_content}</text>",
            )
            .unwrap();

            // Close <a> element if this was a link
            if svg_text_span
                .md_style
                .as_ref()
                .is_some_and(|svg_md_style| svg_md_style.link_dest.is_some())
            {
                write!(content_buffer, "</a>").unwrap();
            }
        });

        // Add image elements for inline images
        svg_image_spans.iter().for_each(|svg_image_span| {
            let x = svg_image_span.x;
            let y = svg_image_span.y;
            let w = svg_image_span.width;
            let h = svg_image_span.height;
            let src = &svg_image_span.src;
            let alt = StringXmlEscaper::escape(&svg_image_span.alt);
            write!(
                content_buffer,
                "<g transform=\"translate({x}, {y})\">\
                    <image \
                        width=\"{w}\" \
                        height=\"{h}\" \
                        href=\"{src}\" \
                        alt=\"{alt}\" />\
                </g>",
            )
            .unwrap();
        });
    }

    /// Builds an SVG `<path>` `d` attribute for a rounded rectangle at absolute
    /// coordinates `(x, y)` with the given `width`, `height`, and corner
    /// `radius`, used for the inline-code background.
    ///
    /// The path proceeds clockwise from just after the top-left corner, drawing
    /// each corner with an elliptical arc. The radius is clamped so it never
    /// exceeds half the width or height. Example: a `75x17` box at `(96, 109)`
    /// with radius `3` yields a `d` starting `M 99 109 H 168 A 3 3 0 0 1 171
    /// 112`.
    fn code_bg_path_d(x: f32, y: f32, width: f32, height: f32, radius: f32) -> String {
        let mut d = String::with_capacity(160);
        Self::rounded_rect_subpath(&mut d, x, y, width, height, radius);
        d
    }

    /// Builds the `<path>` element for an interaction edge label/description
    /// background box, if `tailwind_classes` has an entry for `bg_id`.
    ///
    /// Returns `None` for dependency edges (no entry is ever inserted for
    /// them by `TailwindClassesBuilder`), so the background is a zero-cost
    /// no-op there, mirroring the halo pattern. `stroke-width="0"` is set
    /// explicitly so the box does not inherit the parent `<g>`'s edge stroke
    /// colour/width. Visibility is not set here -- the box inherits it via
    /// normal CSS inheritance from its parent `<g>`, which already carries
    /// the edge's own visibility classes.
    fn interaction_edge_bg_path(
        tailwind_classes: &EntityTailwindClasses<'_>,
        bg_id: &disposition_model_common::Id<'static>,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Option<String> {
        let bg_classes = tailwind_classes.get(bg_id)?;
        let path_d = Self::code_bg_path_d(x, y, width, height, INTERACTION_EDGE_BG_CORNER_RADIUS);
        let class_attr = Self::class_attr_escaped(bg_classes.as_str());
        Some(format!(
            "<path d=\"{path_d}\" stroke-width=\"0\"{class_attr} />"
        ))
    }

    /// Appends a single clockwise rounded-rectangle subpath (`M ... Z`) to `d`,
    /// starting just after the top-left corner. The radius is clamped so it
    /// never exceeds half the width or height.
    fn rounded_rect_subpath(d: &mut String, x: f32, y: f32, width: f32, height: f32, radius: f32) {
        let r = radius.clamp(0.0, (width / 2.0).min(height / 2.0));
        let x_r = x + r;
        let x_w = x + width;
        let y_r = y + r;
        let y_h = y + height;

        // Top edge, starting after the top-left corner.
        write!(d, "M {x_r} {y}").unwrap();
        write!(d, " H {}", x_w - r).unwrap();
        // Top-right corner.
        write!(d, " A {r} {r} 0 0 1 {x_w} {y_r}").unwrap();
        // Right edge.
        write!(d, " V {}", y_h - r).unwrap();
        // Bottom-right corner.
        write!(d, " A {r} {r} 0 0 1 {} {y_h}", x_w - r).unwrap();
        // Bottom edge.
        write!(d, " H {x_r}").unwrap();
        // Bottom-left corner.
        write!(d, " A {r} {r} 0 0 1 {x} {}", y_h - r).unwrap();
        // Left edge.
        write!(d, " V {y_r}").unwrap();
        // Top-left corner.
        write!(d, " A {r} {r} 0 0 1 {x_r} {y}").unwrap();

        d.push_str(" Z");
    }

    /// Builds an SVG `<path>` `d` attribute for a blockquote border frame at
    /// absolute coordinates `(x, y)` with the given `width` and `height`.
    ///
    /// The path is an outer rounded rectangle with an inner rounded rectangle
    /// subtracted (drawn with `fill-rule="evenodd"`), leaving a frame whose
    /// left side is [`BLOCKQUOTE_BORDER_LEFT`] thick (the quote bar) and whose
    /// top, right, and bottom are [`BLOCKQUOTE_BORDER_THIN`] thick. Corners are
    /// rounded with a small [`BLOCKQUOTE_BORDER_RADIUS`] arc; the inner radius
    /// is reduced by the thin border so the rounded borders keep a uniform
    /// width.
    fn blockquote_border_path_d(x: f32, y: f32, width: f32, height: f32) -> String {
        let inner_x = x + BLOCKQUOTE_BORDER_LEFT;
        let inner_y = y + BLOCKQUOTE_BORDER_THIN;
        let inner_w = (width - BLOCKQUOTE_BORDER_LEFT - BLOCKQUOTE_BORDER_THIN).max(0.0);
        let inner_h = (height - BLOCKQUOTE_BORDER_THIN * 2.0).max(0.0);
        let inner_radius = (BLOCKQUOTE_BORDER_RADIUS - BLOCKQUOTE_BORDER_THIN).max(0.0);

        let mut d = String::with_capacity(320);
        // Outer rounded rectangle, clockwise.
        Self::rounded_rect_subpath(&mut d, x, y, width, height, BLOCKQUOTE_BORDER_RADIUS);
        // Inner rounded rectangle (the hole), subtracted via the even-odd rule.
        d.push(' ');
        Self::rounded_rect_subpath(&mut d, inner_x, inner_y, inner_w, inner_h, inner_radius);

        d
    }

    /// Writes edges to the SVG content buffer.
    ///
    /// These will be rendered as the following elements (some attributes
    /// omitted):
    ///
    /// ```svg
    /// <g id="{edge_id}" .. >
    ///   <path d="{path_d}" class="edge_halo .." />
    ///   <path d="{rail_a_d}" class="edge_halo_outline .." />
    ///   <path d="{rail_b_d}" class="edge_halo_outline .." />
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
    ///
    /// When `RenderOptions::interaction_edge_halo` is enabled, interaction
    /// edges also get a `<path class="edge_halo ..">` as the *first* child
    /// of the `<g>`, sharing the same `d` as `edge_body` -- so it paints
    /// behind the edge -- with tailwind classes looked up via the
    /// `{edge_id}__halo` entity key. Dependency edges (and interaction edges
    /// when the halo is disabled) have no such entry, so the halo path is
    /// omitted.
    ///
    /// Immediately after the halo path (still before `edge_body`), two more
    /// `<path class="edge_halo_outline ..">` elements are emitted -- the
    /// halo's outline rails, running along the long sides of the halo
    /// ribbon (with no line closing off the short ends). They paint on top
    /// of the halo fill (since they sit exactly on its outer edge), styled
    /// via tailwind classes looked up via the `{edge_id}__halo_outline`
    /// entity key. Omitted under the same conditions as the halo path.
    fn render_edges(
        content_buffer: &mut String,
        svg_edge_infos: &[SvgEdgeInfo<'_>],
        tailwind_classes: &EntityTailwindClasses<'_>,
    ) {
        svg_edge_infos.iter().for_each(|svg_edge_info| {
            let edge_id = &svg_edge_info.edge_id;
            let path_d = &svg_edge_info.path_d;
            let arrow_head_path_d = &svg_edge_info.arrow_head_path_d;
            let locus_path_d = &svg_edge_info.locus_path_d;

            // Build class attribute from tailwind_classes for the edge.
            //
            // Each edge's class string already contains the fully merged
            // classes (edge group base + edge-specific overrides), so only
            // the edge ID needs to be looked up.
            let class_attr = {
                let edge_tailwind_classes = tailwind_classes
                    .get(edge_id.as_ref())
                    .map(|edge_tailwind_classes| edge_tailwind_classes.as_str())
                    .unwrap_or("");

                Self::class_attr_escaped(edge_tailwind_classes)
            };

            // Build class attribute for the arrowhead element.
            //
            // For interaction edges the builder stores offset-path and
            // animation tailwind classes under the key
            // `{edge_id}__arrow_head`.  For dependency edges no such entry
            // exists, so we fall back to a plain `arrow_head` class.
            let arrow_head_entity_key = format!("{edge_id}__arrow_head");
            let arrow_head_class_attr = if let Ok(arrow_head_id) =
                disposition_model_common::Id::try_from(arrow_head_entity_key)
            {
                let extra = tailwind_classes
                    .get(&arrow_head_id)
                    .map(|s| s.as_str())
                    .unwrap_or("");
                if extra.is_empty() {
                    Self::class_attr_escaped("arrow_head")
                } else {
                    Self::class_attr_escaped(format!("arrow_head\n{extra}").as_str())
                }
            } else {
                Self::class_attr_escaped("arrow_head")
            };

            // Build class attribute for the halo element, if the interaction
            // edge halo is enabled.  The tailwind classes builder only
            // stores an entry under `{edge_id}__halo` for interaction edges
            // when `RenderOptions::interaction_edge_halo` is `Enabled`, so
            // its presence is the sole signal for whether to render it.
            let halo_id = crate::EdgeHaloIdGenerator::generate(edge_id);
            let halo_path = tailwind_classes.get(&halo_id).map(|halo_classes| {
                let halo_class_attr =
                    Self::class_attr_escaped(format!("edge_halo\n{halo_classes}").as_str());
                format!("<path d=\"{path_d}\" fill=\"none\"{halo_class_attr} />")
            });

            // Build the halo outline rails' `<path>` elements, if the halo
            // outline has a resolved entry under `{edge_id}__halo_outline`
            // (only present for interaction edges when the halo is
            // enabled).
            let halo_outline_id = crate::EdgeHaloOutlineIdGenerator::generate(edge_id);
            let halo_outline_paths =
                tailwind_classes
                    .get(&halo_outline_id)
                    .map(|outline_classes| {
                        let outline_class_attr = Self::class_attr_escaped(
                            format!("edge_halo_outline\n{outline_classes}").as_str(),
                        );
                        let rail_a_d = &svg_edge_info.halo_outline_rail_a_path_d;
                        let rail_b_d = &svg_edge_info.halo_outline_rail_b_path_d;
                        format!(
                            "<path d=\"{rail_a_d}\" fill=\"none\"{outline_class_attr} />\
                     <path d=\"{rail_b_d}\" fill=\"none\"{outline_class_attr} />"
                        )
                    });

            // Render edge as a group with an optional halo path, a path, and
            // an arrowhead path.
            //
            // The edge path has fill="none" since edges are stroked lines,
            // not filled shapes.  The arrowhead is a closed V-shape that
            // inherits stroke/fill from the <g>.
            write!(
                content_buffer,
                "<g \
                    id=\"{edge_id}\" \
                    tabindex=\"-1\" \
                    {class_attr}\
                >"
            )
            .unwrap();

            // Add tooltip element if present
            if !svg_edge_info.tooltip.is_empty() {
                let tooltip_escaped = StringXmlEscaper::escape(&svg_edge_info.tooltip);
                write!(content_buffer, "<title>{tooltip_escaped}</title>").unwrap();
            }

            // The halo path is drawn first so it paints behind edge_body.
            if let Some(halo_path) = halo_path {
                content_buffer.push_str(&halo_path);
            }

            // The outline rails paint above the halo fill (they sit exactly
            // on its outer edge) but still behind edge_body.
            if let Some(halo_outline_paths) = halo_outline_paths {
                content_buffer.push_str(&halo_outline_paths);
            }

            write!(
                content_buffer,
                "<path \
                    d=\"{path_d}\" \
                    fill=\"none\" \
                    class=\"edge_body\"
                />\
                <path \
                    d=\"{locus_path_d}\" \
                    fill=\"none\" \
                    class=\"locus\" \
                />\
                <g \
                    {arrow_head_class_attr} \
                >\
                    <path \
                        d=\"{arrow_head_path_d}\" \
                    />\
                </g>\
                </g>"
            )
            .unwrap();
        });
    }

    /// Writes edge labels to the SVG content buffer.
    ///
    /// For each [`SvgEdgeLabelInfo`], emits a `<g>` element for the `from`
    /// label and a `<g>` element for the `to` label (when their text or image
    /// spans are non-empty). Each `<g>` carries the edge's Tailwind CSS classes
    /// so that the label inherits the edge's colour and visibility behaviour.
    ///
    /// For interaction edges, a background `<path>` (see
    /// [`Self::interaction_edge_bg_path`]) is emitted as the first child of
    /// each populated `<g>`, behind the text/image content, so the label
    /// stays legible if it overlaps a dependency edge's label or description
    /// (dependency and interaction edges are positioned via separate,
    /// uncoordinated offset pools -- see `doc/src/edge_paths.md`).
    ///
    /// Label text is rendered with markdown styling (bold, italic, inline
    /// code, links) and inline images via [`Self::render_text_and_images`],
    /// matching node and edge description rendering. The span coordinates are
    /// absolute (not relative to the enclosing `<g>`).
    ///
    /// ```svg
    /// <g id="{edge_id}__from_label" {class_attr}>
    ///   <path d="..." stroke-width="0" {bg_class_attr} />
    ///   <text x="{x}" y="{y}" stroke-width="0">{text}</text>
    /// </g>
    /// <g id="{edge_id}__to_label" {class_attr}>
    ///   <path d="..." stroke-width="0" {bg_class_attr} />
    ///   <text x="{x}" y="{y}" stroke-width="0">{text}</text>
    /// </g>
    /// ```
    fn render_edge_labels(
        content_buffer: &mut String,
        edge_label_infos: &[SvgEdgeLabelInfo<'_>],
        tailwind_classes: &EntityTailwindClasses<'_>,
    ) {
        edge_label_infos.iter().for_each(|svg_edge_label_info| {
            let edge_id = &svg_edge_label_info.edge_id;
            let class_attr = {
                let edge_tailwind_classes = tailwind_classes
                    .get(edge_id.as_ref())
                    .map(|s| s.as_str())
                    .unwrap_or("");
                Self::class_attr_escaped(edge_tailwind_classes)
            };
            let label_bg_id = crate::EdgeLabelBgIdGenerator::generate(edge_id);

            if let Some(from_label) = &svg_edge_label_info.from_label
                && (!from_label.text_spans.is_empty() || !from_label.image_spans.is_empty())
            {
                write!(
                    content_buffer,
                    "<g id=\"{edge_id}__from_label\"{class_attr}>"
                )
                .unwrap();
                if let Some(bg_path) = Self::interaction_edge_bg_path(
                    tailwind_classes,
                    &label_bg_id,
                    from_label.x,
                    from_label.y,
                    from_label.width,
                    from_label.height,
                ) {
                    content_buffer.push_str(&bg_path);
                }
                Self::render_text_and_images(
                    content_buffer,
                    &from_label.text_spans,
                    &from_label.image_spans,
                );
                content_buffer.push_str("</g>");
            }

            if let Some(to_label) = &svg_edge_label_info.to_label
                && (!to_label.text_spans.is_empty() || !to_label.image_spans.is_empty())
            {
                write!(content_buffer, "<g id=\"{edge_id}__to_label\"{class_attr}>").unwrap();
                if let Some(bg_path) = Self::interaction_edge_bg_path(
                    tailwind_classes,
                    &label_bg_id,
                    to_label.x,
                    to_label.y,
                    to_label.width,
                    to_label.height,
                ) {
                    content_buffer.push_str(&bg_path);
                }
                Self::render_text_and_images(
                    content_buffer,
                    &to_label.text_spans,
                    &to_label.image_spans,
                );
                content_buffer.push_str("</g>");
            }
        });
    }

    /// Writes edge descriptions to the SVG content buffer.
    ///
    /// For each [`SvgEdgeDescriptionInfo`], emits a `<g>` element with
    /// `<text>` children for each line of wrapped description text.
    ///
    /// For interaction edges, a background `<path>` (see
    /// [`Self::interaction_edge_bg_path`]) is emitted as the first child,
    /// behind the text/image content, so the description stays legible if it
    /// overlaps a dependency edge's label or description (dependency and
    /// interaction edges are positioned via separate, uncoordinated offset
    /// pools -- see `doc/src/edge_paths.md`).
    ///
    /// ```svg
    /// <g id="{edge_id}__desc" class="edge-description">
    ///   <path d="..." stroke-width="0" {bg_class_attr} />
    ///   <text x="{x}" y="{y}" stroke-width="0">{text}</text>
    /// </g>
    /// ```
    fn render_edge_descriptions(
        content_buffer: &mut String,
        edge_description_infos: &[SvgEdgeDescriptionInfo<'_>],
        tailwind_classes: &EntityTailwindClasses<'_>,
    ) {
        edge_description_infos
            .iter()
            .filter(|svg_edge_description_info| {
                !svg_edge_description_info.text_spans.is_empty()
                    || !svg_edge_description_info.image_spans.is_empty()
            })
            .for_each(|svg_edge_description_info| {
                let edge_id = &svg_edge_description_info.edge_id;

                let class_attr = {
                    let edge_classes = tailwind_classes
                        .get(edge_id.as_ref())
                        .map(|edge_tailwind_classes| edge_tailwind_classes.as_str())
                        .unwrap_or("");
                    Self::class_attr_escaped(edge_classes)
                };

                write!(content_buffer, "<g id=\"{edge_id}__desc\" {class_attr}>").unwrap();

                let desc_bg_id = crate::EdgeDescBgIdGenerator::generate(edge_id);
                if let Some(bg_path) = Self::interaction_edge_bg_path(
                    tailwind_classes,
                    &desc_bg_id,
                    svg_edge_description_info.x,
                    svg_edge_description_info.y,
                    svg_edge_description_info.width,
                    svg_edge_description_info.height,
                ) {
                    content_buffer.push_str(&bg_path);
                }

                // Add text and image elements
                Self::render_text_and_images(
                    content_buffer,
                    &svg_edge_description_info.text_spans,
                    &svg_edge_description_info.image_spans,
                );

                content_buffer.push_str("</g>");
            });
    }

    /// Returns the `class=\"..\"` attribute with `&` escaped as `&amp;`.
    fn class_attr_escaped(tailwind_classes: &str) -> String {
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
    /// Only underscores that are part of an ID selector (starting with `#`) or
    /// class selector (starting with `.`) are escaped. For example:
    /// - `group-has-[#some_id:focus]` -> `group-has-[#some&#95;id:focus]`
    /// - `[&>.edge_body]` -> `[&>.edge&#95;body]`
    /// - `peer/some-peer:animate-[animation-name_2s_linear_infinite]` ->
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
    /// // Class selectors have underscores escaped
    /// assert_eq!(
    ///     SvgElementsToSvgMapper::escape_ids_in_brackets("[&>.edge_body]:stroke-blue-500"),
    ///     "[&>.edge&#95;body]:stroke-blue-500"
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
        let mut is_parsing_id_or_class = false;
        let mut is_last_character_non_id = true;

        classes
            .chars()
            .fold(String::with_capacity(classes.len()), |mut result, c| {
                // https://docs.rs/encre-css/latest/encre_css/plugins/typography/content/index.html
                match c {
                    '[' => {
                        bracket_depth += 1;
                        is_parsing_id_or_class = false;
                        result.push(c);
                    }
                    ']' => {
                        bracket_depth = bracket_depth.saturating_sub(1);
                        is_parsing_id_or_class = false;
                        result.push(c);
                    }
                    '#' if bracket_depth > 0 => {
                        is_parsing_id_or_class = true;
                        result.push(c);
                    }
                    '.' if bracket_depth > 0 && is_last_character_non_id => {
                        is_parsing_id_or_class = true;
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
                    '_' if bracket_depth > 0 && is_parsing_id_or_class => {
                        result.push_str("&#95;");
                    }
                    // Characters that end an ID or CSS class context (not valid in CSS IDs)
                    ':' | ' ' | ',' | '.' | '>' | '+' | '~' | '(' | ')' | '&'
                        if is_parsing_id_or_class =>
                    {
                        is_parsing_id_or_class = false;
                        result.push(c);
                    }
                    _ => {
                        result.push(c);
                    }
                }

                is_last_character_non_id =
                    matches!(c, ':' | ' ' | ',' | '.' | '>' | '+' | '~' | '(' | ')' | '&');

                result
            })
    }
}
