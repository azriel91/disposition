//! Tests for input to IR diagram mapping.

pub(crate) const EXAMPLE_INPUT: &str = include_str!("example_input.yaml");
pub(crate) const EXAMPLE_INPUT_MERGED: &str = include_str!("example_input_merged.yaml");
pub(crate) const EXAMPLE_IR: &str = include_str!("example_ir.yaml");

mod input_diagram_merger;
mod input_to_ir_diagram_mapper;
mod ir_to_taffy_builder;
mod taffy_to_svg_mapper;
