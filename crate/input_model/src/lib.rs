//! SVG diagram generator input data model.
//!
//! The diagram input model is hand written, as an OpenAPI spec doesn't support
//! modelling certain data structures such as a Map with a particular key type.

pub use crate::input_diagram::InputDiagram;

pub mod edge;
pub mod entity;
pub mod process;
pub mod tag;
pub mod theme;
pub mod thing;

mod input_diagram;
