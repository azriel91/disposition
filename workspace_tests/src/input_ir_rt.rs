//! Tests for input to IR diagram mapping.

pub(crate) const EXAMPLE_INPUT: &str = include_str!("example_input.yaml");
pub(crate) const EXAMPLE_INPUT_MERGED: &str = include_str!("example_input_merged.yaml");
pub(crate) const EXAMPLE_IR: &str = include_str!("example_ir.yaml");
pub(crate) const INPUT_DIAGRAM_NESTED_NODE_EDGE_PROTRUSION: &str =
    include_str!("input_diagram/0001_nested_node_edge_protrusion.yaml");

mod input_diagram_merger;
mod input_to_ir_diagram_mapper;
mod ir_to_taffy_builder;
mod node_ranks_calculator;
mod svg_elements_to_svg_mapper;
mod taffy_to_svg_elements_mapper;
