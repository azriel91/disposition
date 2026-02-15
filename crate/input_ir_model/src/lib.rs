//! `disposition` diagram generator input to IR mapping data types.

// Re-exports
// This allows consumers to not need to depend on `utoipa` manually.
#[cfg(all(feature = "openapi", not(feature = "test")))]
pub use utoipa;

#[cfg(all(feature = "openapi", not(feature = "test")))]
pub use crate::api_doc::ApiDoc;
pub use crate::{
    edge_animation_active::EdgeAnimationActive, ir_diagram_and_issues::IrDiagramAndIssues,
};

pub mod issue;

#[cfg(all(feature = "openapi", not(feature = "test")))]
mod api_doc;
mod edge_animation_active;
mod ir_diagram_and_issues;
