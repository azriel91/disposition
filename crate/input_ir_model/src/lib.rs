//! `disposition` diagram generator input to IR mapping data types.

// Re-exports
// This allows consumers to not need to depend on `utoipa` manually.
#[cfg(feature = "openapi")]
pub use utoipa;

#[cfg(feature = "openapi")]
pub use crate::api_doc::ApiDoc;
pub use crate::ir_diagram_and_issues::IrDiagramAndIssues;

pub mod issue;

#[cfg(feature = "openapi")]
mod api_doc;
mod ir_diagram_and_issues;
