//! Data types for disposition to represent SVG elements.

pub use crate::{
    svg_edge_info::SvgEdgeInfo, svg_elements::SvgElements, svg_node_info::SvgNodeInfo,
    svg_node_info_circle::SvgNodeInfoCircle, svg_process_info::SvgProcessInfo,
    svg_text_span::SvgTextSpan,
};

mod svg_edge_info;
mod svg_elements;
mod svg_node_info;
mod svg_node_info_circle;
mod svg_process_info;
mod svg_text_span;
