#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg(test)]

/// Default values for input diagrams -- theme, entity types, and css.
pub const BASE_DIAGRAM_YAML: &str = include_str!("base_diagram.yaml");

mod input_ir_rt;
mod input_model;
mod ir_model;
