//! Tests for input to IR diagram mapping.

pub(crate) const EXAMPLE_INPUT: &str = include_str!("example_input.yaml");
pub(crate) const EXAMPLE_INPUT_MERGED: &str = include_str!("example_input_merged.yaml");
pub(crate) const EXAMPLE_IR: &str = include_str!("example_ir.yaml");
pub(crate) const INPUT_DIAGRAM_0001_NESTED_NODE_EDGE_PROTRUSION: &str =
    include_str!("input_diagram/0001_nested_node_edge_protrusion.yaml");
pub(crate) const INPUT_DIAGRAM_0002_NESTED_NODE_EDGE_PROTRUSION: &str =
    include_str!("input_diagram/0002_nested_x2_node_edge_protrusion.yaml");
pub(crate) const INPUT_DIAGRAM_0003_EDGES_SYMMETRIC_2_NODES: &str =
    include_str!("input_diagram/0003_edges_symmetric_2_nodes.yaml");
pub(crate) const INPUT_DIAGRAM_0004_EDGES_SYMMETRIC_3_NODES: &str =
    include_str!("input_diagram/0004_edges_symmetric_3_nodes.yaml");
pub(crate) const INPUT_DIAGRAM_0005_TAG_NODES_CYCLIC_EDGE: &str =
    include_str!("input_diagram/0005_tag_nodes_cyclic_edge.yaml");
pub(crate) const INPUT_DIAGRAM_0006_PROCESS_STEP_NODES_CYCLIC_EDGE: &str =
    include_str!("input_diagram/0006_process_step_nodes_cyclic_edge.yaml");
pub(crate) const INPUT_DIAGRAM_0007_EDGE_FROM_NODE_TO_NESTED_NODE: &str =
    include_str!("input_diagram/0007_edge_from_node_to_nested_node.yaml");
pub(crate) const INPUT_DIAGRAM_0008_EDGE_FROM_NODE_TO_NESTED_RANK_1_NODE: &str =
    include_str!("input_diagram/0008_edge_from_node_to_nested_rank_1_node.yaml");
pub(crate) const INPUT_DIAGRAM_0009_EDGE_WITH_DESCRIPTION: &str =
    include_str!("input_diagram/0009_edge_with_description.yaml");
pub(crate) const INPUT_DIAGRAM_0010_SELF_LOOP_EDGE_WITH_DESCRIPTION: &str =
    include_str!("input_diagram/0010_self_loop_edge_with_description.yaml");
pub(crate) const INPUT_DIAGRAM_0011_CONTAINED_EDGE_WITH_DESCRIPTION: &str =
    include_str!("input_diagram/0011_contained_edge_with_description.yaml");
pub(crate) const INPUT_DIAGRAM_0012_EDGE_FROM_NESTED_NODE_TO_OUTER_NODE_CYCLIC: &str =
    include_str!("input_diagram/0012_edge_from_nested_node_to_outer_node_cyclic.yaml");
pub(crate) const INPUT_DIAGRAM_0013_EDGE_FROM_NESTED_NODE_TO_OUTER_NODE_CYCLIC_2: &str =
    include_str!("input_diagram/0013_edge_from_nested_node_to_outer_node_cyclic_2.yaml");
pub(crate) const INPUT_DIAGRAM_0017_EDGE_INNER_TO_INNER: &str =
    include_str!("input_diagram/0017_edge_inner_to_inner.yaml");
pub(crate) const INPUT_DIAGRAM_0018_PROCESS_STEP_BRANCH_MERGE: &str =
    include_str!("input_diagram/0018_process_step_branch_merge.yaml");

mod input_diagram_merger;
mod input_to_ir_diagram_mapper;
mod ir_to_taffy_builder;
mod node_ranks_calculator;
mod svg_elements_to_svg_mapper;
mod taffy_to_svg_elements_mapper;
