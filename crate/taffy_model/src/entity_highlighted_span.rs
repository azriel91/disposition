use serde::{Deserialize, Serialize};

use crate::MdStyle;

/// `syntect` highlighted span and its `layout_run` x/y/w/h from `cosmic-text`.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct EntityHighlightedSpan {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub text: String,
    /// Markdown style for this span. `None` for plain/unstyled text spans
    /// produced by the legacy path.
    pub md_style: Option<MdStyle>,
    /// Tailwind classes to apply to this span. Empty for plain/unstyled text.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tailwind_classes: Vec<String>,
}
