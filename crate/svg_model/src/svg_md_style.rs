use serde::{Deserialize, Serialize};

/// The SVG-layer representation of markdown inline formatting.
///
/// Mirrors the fields of `MdStyle` but lives in `svg_model` rather than
/// `taffy_model`, maintaining the existing crate separation.
///
/// # Examples
///
/// ```rust
/// use disposition_svg_model::SvgMdStyle;
///
/// let style = SvgMdStyle {
///     bold: true,
///     italic: false,
///     strikethrough: false,
///     code: false,
///     blockquote: false,
///     heading_level: 2,
///     link_dest: None,
/// };
/// assert_eq!(style.heading_level, 2);
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct SvgMdStyle {
    /// Whether the span is bold.
    pub bold: bool,
    /// Whether the span is italic.
    pub italic: bool,
    /// Whether the span is strikethrough.
    pub strikethrough: bool,
    /// Whether the span is inline code.
    pub code: bool,
    /// Whether the span is a blockquote bordered-box frame.
    pub blockquote: bool,
    /// Heading level: `1`--`6`, or `0` for non-heading text.
    pub heading_level: u8,
    /// Destination URL when the span is part of a link. `None` otherwise.
    ///
    /// Example: `"https://example.com"`.
    pub link_dest: Option<String>,
}
