use serde::{Deserialize, Serialize};

/// An inline image positioned in the diagram's coordinate space.
///
/// Defined in `taffy_model` (not `svg_model`) so it can be stored in
/// `TaffyNodeMappings` without creating a dependency on `svg_model`.
///
/// # Examples
///
/// ```rust
/// use disposition_taffy_model::MdImageSpan;
///
/// let span = MdImageSpan {
///     x: 10.0,
///     y: 20.0,
///     width: 100.0,
///     height: 80.0,
///     src: "diagram.png".to_string(),
///     alt: "A diagram".to_string(),
/// };
/// assert_eq!(span.src, "diagram.png");
/// ```
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct MdImageSpan {
    /// Absolute x coordinate of the image's top-left corner.
    pub x: f32,
    /// Absolute y coordinate of the image's top-left corner.
    pub y: f32,
    /// Rendered width in pixels.
    pub width: f32,
    /// Rendered height in pixels.
    pub height: f32,
    /// Image source (data URL or path).
    pub src: String,
    /// Alt text.
    pub alt: String,
}
