//! Tests for input to IR diagram mapping.

pub(crate) const EXAMPLE_INPUT: &str = include_str!("example_input.yaml");
pub(crate) const EXAMPLE_INPUT_MERGED: &str = include_str!("example_input_merged.yaml");
pub(crate) const EXAMPLE_IR: &str = include_str!("example_ir.yaml");
pub(crate) const INPUT_DIAGRAM_NESTED_NODE_EDGE_PROTRUSION: &str =
    include_str!("input_diagram/0001_nested_node_edge_protrusion.yaml");
pub(crate) const INPUT_DIAGRAM_EDGES_SYMMETRIC_2_NODES: &str =
    include_str!("input_diagram/0002_edges_symmetric_2_nodes.yaml");
pub(crate) const INPUT_DIAGRAM_EDGES_SYMMETRIC_3_NODES: &str =
    include_str!("input_diagram/0003_edges_symmetric_3_nodes.yaml");
pub(crate) const INPUT_DIAGRAM_TAG_NODES_CYCLIC_EDGE: &str =
    include_str!("input_diagram/0004_tag_nodes_cyclic_edge.yaml");
pub(crate) const INPUT_DIAGRAM_PROCESS_STEP_NODES_CYCLIC_EDGE: &str =
    include_str!("input_diagram/0005_process_step_nodes_cyclic_edge.yaml");

mod input_diagram_merger;
mod input_to_ir_diagram_mapper;
mod ir_to_taffy_builder;
mod node_ranks_calculator;
mod svg_elements_to_svg_mapper;
mod taffy_to_svg_elements_mapper;
