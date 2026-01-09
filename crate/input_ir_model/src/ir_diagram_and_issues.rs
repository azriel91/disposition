use disposition_ir_model::IrDiagram;

use crate::issue::ModelToIrIssue;

/// An input diagram and any issues encountered during mapping from the input
/// model.
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug)]
pub struct IrDiagramAndIssues<'id> {
    /// The mapped intermediate representation diagram.
    pub diagram: IrDiagram<'id>,
    /// Issues encountered during mapping.
    pub issues: Vec<ModelToIrIssue>,
}

impl<'id> IrDiagramAndIssues<'id> {
    /// Returns a reference to the intermediate representation diagram.
    pub fn diagram(&self) -> &IrDiagram<'id> {
        &self.diagram
    }

    /// Returns a reference to the issues encountered during mapping.
    pub fn issues(&self) -> &[ModelToIrIssue] {
        &self.issues
    }
}
