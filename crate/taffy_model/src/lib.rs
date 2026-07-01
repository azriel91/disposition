//! Data types for disposition to work with taffy.

// Re-exports
pub use taffy;

/// Default text font size.
pub const TEXT_FONT_SIZE: f32 = 14.0f32;
/// Default text line height.
pub const TEXT_LINE_HEIGHT: f32 = 17.0f32;
/// Width of a single lane column in the process step git-graph layout.
///
/// Each process step circle is centred within a lane of this width, so adjacent
/// lanes (and the connectors that run in them) are visually separated. The step
/// text column always begins after `lane_count * LANE_WIDTH`, keeping step
/// labels left-aligned in a single column.
pub const LANE_WIDTH: f32 = 32.0f32;

pub use crate::{
    diagram_lod::DiagramLod,
    dimension::Dimension,
    dimension_and_lod::DimensionAndLod,
    edge_description_ctx::EdgeDescriptionCtx,
    edge_description_taffy_nodes::EdgeDescriptionTaffyNodes,
    edge_id_to_edge_description_taffy_nodes::EdgeIdToEdgeDescriptionTaffyNodes,
    edge_id_to_edge_label_taffy_node_ids::EdgeIdToEdgeLabelTaffyNodeIds,
    edge_id_to_edge_spacer_taffy_nodes::EdgeIdToEdgeSpacerTaffyNodes,
    edge_id_to_highlighted_spans::EdgeIdToHighlightedSpans,
    edge_id_to_image_spans::EdgeIdToImageSpans,
    edge_label_ctx::EdgeLabelCtx,
    edge_label_taffy_node_ids::EdgeLabelTaffyNodeIds,
    edge_spacer_ctx::EdgeSpacerCtx,
    edge_spacer_taffy_nodes::EdgeSpacerTaffyNodes,
    entity_highlighted_span::EntityHighlightedSpan,
    entity_highlighted_spans::EntityHighlightedSpans,
    error::IrToTaffyError,
    md_block_taffy_ids::MdBlockTaffyIds,
    md_colors::{MdColor, MD_BLOCKQUOTE_BORDER_COLOR, MD_CODE_BG_COLOR, MD_LINK_COLOR},
    md_heading_level::MdHeadingLevel,
    md_image_ctx::MdImageCtx,
    md_image_span::MdImageSpan,
    md_node_taffy_ids::MdNodeTaffyIds,
    md_style::MdStyle,
    md_token_ctx::MdTokenCtx,
    node_id_to_envelope_taffy_node::NodeIdToEnvelopeTaffyNode,
    node_id_to_image_spans::NodeIdToImageSpans,
    node_id_to_md_node_taffy_ids::NodeIdToMdNodeTaffyIds,
    node_id_to_taffy_node_ids::NodeIdToTaffyNodeIds,
    node_inbuilt_to_taffy_node::NodeInbuiltToTaffyNode,
    node_to_taffy_node_ids::NodeToTaffyNodeIds,
    processes_included::ProcessesIncluded,
    taffy_node_ctx::TaffyNodeCtx,
    taffy_node_kind::TaffyNodeKind,
    taffy_node_mappings::TaffyNodeMappings,
    taffy_node_to_kind::TaffyNodeToKind,
    taffy_node_to_node_id::TaffyNodeToNodeId,
    taffy_tree_fmt::TaffyTreeFmt,
};

mod diagram_lod;
mod dimension;
mod dimension_and_lod;
mod edge_description_ctx;
mod edge_description_taffy_nodes;
mod edge_id_to_edge_description_taffy_nodes;
mod edge_id_to_edge_label_taffy_node_ids;
mod edge_id_to_edge_spacer_taffy_nodes;
mod edge_id_to_highlighted_spans;
mod edge_id_to_image_spans;
mod edge_label_ctx;
mod edge_label_taffy_node_ids;
mod edge_spacer_ctx;
mod edge_spacer_taffy_nodes;
mod entity_highlighted_span;
mod entity_highlighted_spans;
mod error;
mod md_block_taffy_ids;
mod md_colors;
mod md_heading_level;
mod md_image_ctx;
mod md_image_span;
mod md_node_taffy_ids;
mod md_style;
mod md_token_ctx;
mod node_id_to_envelope_taffy_node;
mod node_id_to_image_spans;
mod node_id_to_md_node_taffy_ids;
mod node_id_to_taffy_node_ids;
mod node_inbuilt_to_taffy_node;
mod node_to_taffy_node_ids;
mod processes_included;
mod taffy_node_ctx;
mod taffy_node_kind;
mod taffy_node_mappings;
mod taffy_node_to_kind;
mod taffy_node_to_node_id;
mod taffy_tree_fmt;
