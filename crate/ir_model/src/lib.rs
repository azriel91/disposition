//! SVG diagram generator intermediate representation.
//!
//! The intermediate representation is the computed data structure from
//! combining the layered values from the input data. It is used to generate
//! the final SVG output.

// Re-exports
// This allows consumers to not need to depend on `utoipa` manually.
#[cfg(feature = "openapi")]
pub use utoipa;

#[cfg(feature = "openapi")]
pub use crate::api_doc::ApiDoc;
pub use crate::ir_diagram::IrDiagram;

pub mod edge;
pub mod entity;
pub mod layout;
pub mod node;

#[cfg(feature = "openapi")]
mod api_doc;
mod ir_diagram;
