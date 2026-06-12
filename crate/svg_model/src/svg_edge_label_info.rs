use disposition_ir_model::edge::EdgeId;
use serde::{Deserialize, Serialize};

use crate::{SvgImageSpan, SvgTextSpan};

/// Information to render SVG elements for an edge label.
///
/// Each edge that has a face assignment (i.e. is not a contained or self-loop
/// edge) may have up to two labels -- one at the `from` endpoint and one at
/// the `to` endpoint.
///
/// # Examples
///
/// ```text
/// SvgEdgeLabelInfo {
///     edge_id: "edge_t_a__t_b__0",
///     from_label: Some(SvgEdgeLabelEndpointInfo { x: 10.0, y: 20.0, .. }),
///     to_label: None,
/// }
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct SvgEdgeLabelInfo<'id> {
    /// ID of the edge this label belongs to.
    pub edge_id: EdgeId<'id>,
    /// Label slot at the `from` endpoint of the edge.
    ///
    /// `None` when the `from` endpoint has no face assignment (contained or
    /// self-loop edges) or when the edge has no description.
    pub from_label: Option<SvgEdgeLabelEndpointInfo>,
    /// Label slot at the `to` endpoint of the edge.
    ///
    /// `None` when the `to` endpoint has no face assignment (contained or
    /// self-loop edges) or when the edge has no description.
    pub to_label: Option<SvgEdgeLabelEndpointInfo>,
}

/// Position, size, and text content of one edge label endpoint slot.
///
/// # Examples
///
/// ```text
/// SvgEdgeLabelEndpointInfo {
///     x: 120.0,
///     y: 48.0,
///     width: 80.0,
///     height: 18.0,
///     text_spans: vec![SvgTextSpan { x: 124.0, y: 61.0, text: "hello".into() }],
///     image_spans: vec![],
/// }
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct SvgEdgeLabelEndpointInfo {
    /// Absolute x coordinate of the label slot's top-left corner.
    pub x: f32,
    /// Absolute y coordinate of the label slot's top-left corner.
    pub y: f32,
    /// Width of the label slot.
    pub width: f32,
    /// Height of the label slot.
    pub height: f32,
    /// Text spans to render, with markdown styling.
    ///
    /// Coordinates are absolute (not relative to the slot). Each span may
    /// carry an `md_style` and Tailwind classes when the label text contains
    /// markdown formatting.
    pub text_spans: Vec<SvgTextSpan>,
    /// Inline image spans for label text containing markdown images.
    ///
    /// Coordinates are absolute (not relative to the slot).
    pub image_spans: Vec<SvgImageSpan>,
}
