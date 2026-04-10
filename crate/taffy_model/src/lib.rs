//! Data types for disposition to work with taffy.

// Re-exports
pub use taffy;

/// Default text font size.
pub const TEXT_FONT_SIZE: f32 = 14.0f32;
/// Default text line height.
pub const TEXT_LINE_HEIGHT: f32 = 17.0f32;

pub use crate::{
    diagram_lod::DiagramLod, diagram_node_ctx::DiagramNodeCtx, dimension::Dimension,
    dimension_and_lod::DimensionAndLod, edge_spacer_ctx::EdgeSpacerCtx,
    edge_spacer_taffy_nodes::EdgeSpacerTaffyNodes, entity_highlighted_span::EntityHighlightedSpan,
    entity_highlighted_spans::EntityHighlightedSpans, error::IrToTaffyError,
    node_to_taffy_node_ids::NodeToTaffyNodeIds, processes_included::ProcessesIncluded,
    taffy_node_ctx::TaffyNodeCtx, taffy_node_mappings::TaffyNodeMappings,
};

mod diagram_lod;
mod diagram_node_ctx;
mod dimension;
mod dimension_and_lod;
mod edge_spacer_ctx;
mod edge_spacer_taffy_nodes;
mod entity_highlighted_span;
mod entity_highlighted_spans;
mod error;
mod node_to_taffy_node_ids;
mod processes_included;
mod taffy_node_ctx;
mod taffy_node_mappings;
