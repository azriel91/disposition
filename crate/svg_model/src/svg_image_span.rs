use serde::{Deserialize, Serialize};

/// Information for an inline `<image>` SVG element.
///
/// # Examples
///
/// ```rust
/// use disposition_svg_model::SvgImageSpan;
///
/// let span = SvgImageSpan {
///     x: 10.0,
///     y: 20.0,
///     width: 100.0,
///     height: 80.0,
///     src: "diagram.png".to_string(),
///     alt: "A diagram".to_string(),
/// };
/// assert_eq!(span.src, "diagram.png");
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct SvgImageSpan {
    /// Absolute x coordinate of the image's top-left corner.
    pub x: f32,
    /// Absolute y coordinate of the image's top-left corner.
    pub y: f32,
    /// Rendered width in pixels.
    pub width: f32,
    /// Rendered height in pixels.
    pub height: f32,
    /// Image source (data URL or path, unescaped).
    ///
    /// Example: `"data:image/png;base64,iVBORw0K..."`, `"diagram.png"`.
    pub src: String,
    /// Alt text.
    pub alt: String,
}
