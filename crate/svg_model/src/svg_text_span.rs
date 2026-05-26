use serde::{Deserialize, Serialize};

use crate::SvgMdStyle;

/// Information for a `<text>` element within an SVG node.
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct SvgTextSpan {
    /// X coordinate for the text element.
    pub x: f32,
    /// Y coordinate for the text element.
    pub y: f32,
    /// Height of the span in pixels (equals `effective_line_height` for the
    /// heading scale). Used to size the code-background `<rect>`.
    ///
    /// Set to `0.0` for spans created via [`SvgTextSpan::new`].
    pub height: f32,
    /// The text content (already XML-escaped).
    pub text: String,
    /// Markdown style for this span. `None` for plain/unstyled text.
    pub md_style: Option<SvgMdStyle>,
}

impl SvgTextSpan {
    /// Creates a new `SvgTextSpan` with no markdown style.
    ///
    /// Sets `height` to `0.0` and `md_style` to `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_svg_model::SvgTextSpan;
    ///
    /// let span = SvgTextSpan::new(1.0, 2.0, "hello".to_string());
    /// assert_eq!(span.text, "hello");
    /// assert_eq!(span.height, 0.0);
    /// assert!(span.md_style.is_none());
    /// ```
    pub fn new(x: f32, y: f32, text: String) -> Self {
        Self {
            x,
            y,
            height: 0.0,
            text,
            md_style: None,
        }
    }

    /// Creates a new `SvgTextSpan` with explicit height and markdown style.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_svg_model::{SvgMdStyle, SvgTextSpan};
    ///
    /// let style = SvgMdStyle {
    ///     bold: true,
    ///     italic: false,
    ///     strikethrough: false,
    ///     code: false,
    ///     heading_level: 0,
    ///     link_dest: None,
    /// };
    /// let span = SvgTextSpan::new_styled(1.0, 2.0, 17.0, "hello".to_string(), Some(style));
    /// assert_eq!(span.height, 17.0);
    /// assert!(span.md_style.is_some());
    /// ```
    pub fn new_styled(
        x: f32,
        y: f32,
        height: f32,
        text: String,
        md_style: Option<SvgMdStyle>,
    ) -> Self {
        Self {
            x,
            y,
            height,
            text,
            md_style,
        }
    }
}
