use serde::{Deserialize, Serialize};
use syntect::highlighting::Style;

/// `syntect` highlighted span and its `layout_run` x/y/w/h from `cosmic-text`.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct EntityHighlightedSpan {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub style: Style,
    pub text: String,
}
