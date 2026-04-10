//! SVG diagram generator intermediate representation.
//!
//! The intermediate representation is the computed data structure from
//! combining the layered values from the input data. It is used to generate
//! the final SVG output.

// Re-exports
pub use crate::ir_diagram::IrDiagram;
pub use enum_iterator;

pub mod edge;
pub mod entity;
pub mod layout;
pub mod node;
pub mod process;

mod ir_diagram;
