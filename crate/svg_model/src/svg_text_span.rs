use serde::{Deserialize, Serialize};

/// Information for a `<text>` element within an SVG node.
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct SvgTextSpan {
    /// X coordinate for the text element.
    pub x: f32,
    /// Y coordinate for the text element.
    pub y: f32,
    /// The text content (already XML-escaped).
    pub text: String,
}

impl SvgTextSpan {
    /// Creates a new `SvgTextSpan`.
    pub fn new(x: f32, y: f32, text: String) -> Self {
        Self { x, y, text }
    }
}
