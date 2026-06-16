//! SVG diagram generator input data model.
//!
//! The diagram input model is hand written, as an OpenAPI spec doesn't support
//! modelling certain data structures such as a Map with a particular key type.

pub use crate::{diagram_focus::DiagramFocus, input_diagram::InputDiagram};

pub mod edge;
pub mod entity;
pub mod process;
pub mod tag;
pub mod theme;
pub mod thing;

mod diagram_focus;
mod input_diagram;
