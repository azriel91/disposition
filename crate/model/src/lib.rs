//! SVG diagram generator input data model.
//!
//! The diagram input model is hand written, as an OpenAPI spec doesn't support
//! modelling certain data structures such as a Map with a particular key type.

#[macro_use]
extern crate id_newtype;

pub use crate::diagram_spec::DiagramSpec;

pub mod common;
pub mod group;
pub mod process;
pub mod thing;

mod diagram_spec;
