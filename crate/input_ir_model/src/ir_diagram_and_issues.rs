use disposition_ir_model::IrDiagram;

use crate::issue::ModelToIrIssue;

/// An input diagram and any issues encountered during mapping from the input
/// model.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug)]
pub struct IrDiagramAndIssues {
    /// The mapped intermediate representation diagram.
    pub diagram: IrDiagram,
    /// Issues encountered during mapping.
    pub issues: Vec<ModelToIrIssue>,
}

impl IrDiagramAndIssues {
    /// Returns a reference to the intermediate representation diagram.
    pub fn diagram(&self) -> &IrDiagram {
        &self.diagram
    }

    /// Returns a reference to the issues encountered during mapping.
    pub fn issues(&self) -> &[ModelToIrIssue] {
        &self.issues
    }
}
