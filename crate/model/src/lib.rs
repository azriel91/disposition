//! SVG diagram generator input data model.
//!
//! The diagram input model is hand written, as an OpenAPI spec doesn't support
//! modelling certain data structures such as a Map with a particular key type.

#[macro_use]
extern crate id_newtype;

// Re-exports
// This allows consumers to not need to depend on `utoipa` manually.
#[cfg(feature = "openapi")]
pub use utoipa;

#[cfg(feature = "openapi")]
pub use crate::api_doc::ApiDoc;
pub use crate::input_diagram::InputDiagram;

pub mod common;
pub mod edge;
pub mod entity;
pub mod process;
pub mod tag;
pub mod theme;
pub mod thing;

#[cfg(feature = "openapi")]
mod api_doc;
mod input_diagram;
