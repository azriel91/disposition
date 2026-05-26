use serde::{Deserialize, Serialize};

/// Context placed on an image leaf node.
///
/// The node is given a fixed size during construction so taffy does not call
/// the measure function for it; the context is retained for SVG rendering.
///
/// # Examples
///
/// ```rust
/// use disposition_taffy_model::MdImageCtx;
///
/// let ctx = MdImageCtx {
///     src: "diagram.png".to_string(),
///     alt: "A diagram".to_string(),
///     width: 100.0,
///     height: 80.0,
/// };
/// assert_eq!(ctx.src, "diagram.png");
/// ```
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct MdImageCtx {
    /// Data URL or relative path of the image.
    ///
    /// Example: `"data:image/png;base64,iVBORw0K..."`, `"diagram.png"`.
    pub src: String,
    /// Alt text for the image.
    pub alt: String,
    /// Rendered width in pixels (already resolved from priority rules).
    pub width: f32,
    /// Rendered height in pixels (already resolved from priority rules).
    pub height: f32,
}
