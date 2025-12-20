//! SVG diagram generator input data model.
//!
//! The diagram input model is hand written, as an OpenAPI spec doesn't support
//! modelling certain data structures such as a Map with a particular key type.

// Re-exports
// This allows consumers to not need to depend on `utoipa` manually.
#[cfg(all(feature = "openapi", not(feature = "test")))]
pub use utoipa;

#[cfg(all(feature = "openapi", not(feature = "test")))]
pub use crate::api_doc::ApiDoc;
pub use crate::input_diagram::InputDiagram;

pub mod edge;
pub mod entity;
pub mod process;
pub mod tag;
pub mod theme;
pub mod thing;

#[cfg(all(feature = "openapi", not(feature = "test")))]
mod api_doc;
mod input_diagram;
