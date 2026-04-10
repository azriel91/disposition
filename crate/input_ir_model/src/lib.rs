//! `disposition` diagram generator input to IR mapping data types.

pub use crate::{
    edge_animation_active::EdgeAnimationActive, ir_diagram_and_issues::IrDiagramAndIssues,
};

pub mod issue;

mod edge_animation_active;
mod ir_diagram_and_issues;
