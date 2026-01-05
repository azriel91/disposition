//! Data types that for disposition to work with taffy.

// Re-exports
pub use cosmic_text;
pub use syntect;
pub use taffy;
// This allows consumers to not need to depend on `utoipa` manually.
#[cfg(all(feature = "openapi", not(feature = "test")))]
pub use utoipa;

#[cfg(all(feature = "openapi", not(feature = "test")))]
pub use crate::api_doc::ApiDoc;
#[cfg(all(feature = "openapi", not(feature = "test")))]
mod api_doc;

pub use crate::{
    diagram_lod::DiagramLod, dimension::Dimension, dimension_and_lod::DimensionAndLod,
    entity_highlighted_span::EntityHighlightedSpan,
    entity_highlighted_spans::EntityHighlightedSpans, error::IrToTaffyError,
    node_context::NodeContext, processes_included::ProcessesIncluded,
    taffy_node_mappings::TaffyNodeMappings,
};

mod diagram_lod;
mod dimension;
mod dimension_and_lod;
mod entity_highlighted_span;
mod entity_highlighted_spans;
mod error;
mod node_context;
mod processes_included;
mod taffy_node_mappings;
