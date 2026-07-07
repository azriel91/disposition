//! Shared class-name / selector-prefix constants for the SVG sub-elements
//! that Tailwind classes need to target precisely.
//!
//! `SvgElementsToSvgMapper` writes the literal `class="..."` attributes
//! using the `*_CLASS` constants; `TailwindClassState` (via
//! `TailwindClassesBuilder`) writes the `[&>.class]:` arbitrary-variant
//! selectors using the paired `*_SELECTOR` constants. Both sides reference
//! these constants instead of hardcoding the literal strings, so they can't
//! drift out of sync.
//!
//! Rust's `concat!` macro cannot concatenate named `const &str`s (only
//! literals), so each selector is spelled out in full rather than derived
//! from its paired class constant -- keep the two halves of each pair in
//! sync by hand if you ever rename one.

/// Class attached to a node's background rect `<path>`. See
/// `SvgElementsToSvgMapper::render_nodes`.
pub(crate) const NODE_WRAPPER_CLASS: &str = "wrapper";
/// Arbitrary-variant selector targeting [`NODE_WRAPPER_CLASS`]. Used to
/// scope Stroke/Fill-derived classes to the node's background shape.
pub(crate) const NODE_WRAPPER_SELECTOR: &str = "[&>.wrapper]:";

/// Class attached to a node's circle `<path>`, present only when the node
/// has a circle shape. See `SvgElementsToSvgMapper::render_nodes`.
pub(crate) const NODE_CIRCLE_CLASS: &str = "circle";
/// Arbitrary-variant selector targeting [`NODE_CIRCLE_CLASS`]. Emitted
/// alongside [`NODE_WRAPPER_SELECTOR`] for every node, since it is not known
/// at class-resolution time whether the node will end up using a circle
/// shape.
pub(crate) const NODE_CIRCLE_SELECTOR: &str = "[&>.circle]:";

/// Class attached to an edge's line `<path>`. See
/// `SvgElementsToSvgMapper::render_edges`.
pub(crate) const EDGE_BODY_CLASS: &str = "edge_body";
/// Arbitrary-variant selector targeting [`EDGE_BODY_CLASS`]. Used to scope
/// Stroke-derived classes (the line colour) on edge entities.
pub(crate) const EDGE_BODY_SELECTOR: &str = "[&>.edge_body]:";

/// Class attached to an edge's arrow head `<g>`. See
/// `SvgElementsToSvgMapper::render_edges`.
pub(crate) const EDGE_ARROW_HEAD_CLASS: &str = "arrow_head";
/// Arbitrary-variant selector targeting [`EDGE_ARROW_HEAD_CLASS`]. Used to
/// scope Fill-derived classes (the arrow head fill colour) on edge
/// entities.
pub(crate) const EDGE_ARROW_HEAD_SELECTOR: &str = "[&>.arrow_head]:";
