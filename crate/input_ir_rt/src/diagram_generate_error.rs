use disposition_taffy_model::IrToTaffyError;
use thiserror::Error;

/// Errors that can occur while generating a diagram via `DiagramGenerator`.
#[derive(Debug, Error)]
pub enum DiagramGenerateError {
    /// Building the taffy layout tree failed.
    #[error("taffy: {0}")]
    Taffy(#[from] IrToTaffyError),
    /// The taffy builder produced no node mappings.
    #[error("no taffy node mappings generated")]
    NoTaffyMappings,
}
