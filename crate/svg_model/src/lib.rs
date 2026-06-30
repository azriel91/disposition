//! Data types for disposition to represent SVG elements.

pub use crate::{
    edge_path_bounds::EdgePathBounds,
    edge_path_midpoint::EdgePathMidpoint,
    edge_routing_diagnostic::EdgeRoutingDiagnostic,
    edge_routing_diagnostics::EdgeRoutingDiagnostics,
    ortho_protrusion_params::OrthoProtrusionParams,
    rank_gap_diagnostic_endpoint_kind::RankGapDiagnosticEndpointKind,
    rank_gap_diagnostic_side::RankGapDiagnosticSide,
    rank_gap_entry_diagnostic::RankGapEntryDiagnostic,
    spacer_protrusion_params::SpacerProtrusionParams,
    svg_edge_description_info::SvgEdgeDescriptionInfo,
    svg_edge_info::SvgEdgeInfo,
    svg_edge_label_info::{SvgEdgeLabelEndpointInfo, SvgEdgeLabelInfo},
    svg_elements::SvgElements,
    svg_image_span::SvgImageSpan,
    svg_md_style::SvgMdStyle,
    svg_node_info::SvgNodeInfo,
    svg_node_info_circle::SvgNodeInfoCircle,
    svg_process_info::SvgProcessInfo,
    svg_text_span::SvgTextSpan,
};

mod edge_path_bounds;
mod edge_path_midpoint;
mod edge_routing_diagnostic;
mod edge_routing_diagnostics;
mod ortho_protrusion_params;
mod rank_gap_diagnostic_endpoint_kind;
mod rank_gap_diagnostic_side;
mod rank_gap_entry_diagnostic;
mod spacer_protrusion_params;
mod svg_edge_description_info;
mod svg_edge_info;
mod svg_edge_label_info;
mod svg_elements;
mod svg_image_span;
mod svg_md_style;
mod svg_node_info;
mod svg_node_info_circle;
mod svg_process_info;
mod svg_text_span;
