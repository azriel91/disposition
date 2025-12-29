use std::fmt;

use miette::Diagnostic;
use serde::{Deserialize, Serialize};

/// Errors that can occur when building a Taffy tree from an `IrDiagram`.
#[derive(Clone, Debug, PartialEq, Diagnostic, Deserialize, Serialize, thiserror::Error)]
pub enum IrToTaffyError {
    NodeLayoutsUnset,
}

impl fmt::Display for IrToTaffyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrToTaffyError::NodeLayoutsUnset => write!(f, "`node_layouts` was not set."),
        }
    }
}
