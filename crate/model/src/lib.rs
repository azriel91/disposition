//! SVG diagram generator input data model.
//!
//! The diagram input model is hand written, as an OpenAPI spec doesn't support
//! modelling certain data structures such as a Map with a particular key type.

#[macro_use]
extern crate id_newtype;

pub use crate::input_diagram::InputDiagram;

pub mod common;
pub mod process;
pub mod tag;
pub mod thing;

mod input_diagram;
