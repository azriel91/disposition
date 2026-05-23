use disposition_ir_model::edge::EdgeId;
use serde::{Deserialize, Serialize};

use crate::SvgTextSpan;

/// Information to render SVG elements for an edge description.
///
/// Each edge that has an `entity_descs` entry and a top-level or nested LCA
/// produces one `SvgEdgeDescriptionInfo`, which is rendered as a `<g>` element
/// with `<text>` children positioned at the edge description container leaf
/// node's absolute coordinates.
///
/// # Examples
///
/// ```text
/// SvgEdgeDescriptionInfo {
///     edge_id: "edge_dep_alice_bob__0",
///     x: 120.0,
///     y: 48.0,
///     width: 80.0,
///     height: 18.0,
///     text_spans: vec![SvgTextSpan { x: 124.2, y: 61.0, text: "connects to".into() }],
/// }
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct SvgEdgeDescriptionInfo<'id> {
    /// ID of the edge whose description this info represents.
    pub edge_id: EdgeId<'id>,
    /// Absolute x coordinate of the description leaf node's top-left corner.
    pub x: f32,
    /// Absolute y coordinate of the description leaf node's top-left corner.
    pub y: f32,
    /// Width of the description leaf node.
    pub width: f32,
    /// Height of the description leaf node.
    pub height: f32,
    /// Text spans to render, with absolute diagram-level coordinates.
    pub text_spans: Vec<SvgTextSpan>,
}
