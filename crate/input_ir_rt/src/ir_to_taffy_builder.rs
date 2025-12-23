use disposition_taffy_model::{
    taffy::{Style, TaffyError, TaffyTree},
    DimensionAndLod, NodeContext, TaffyTreeAndRoot,
};
use serde::{Deserialize, Serialize};

/// Maps an intermediate representation diagram to a `TaffyTreeAndRoot`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IrToTaffyBuilder {
    /// The dimensions at which elements should be repositioned.
    responsive_dimensions_and_detail: Vec<DimensionAndLod>,
}

impl Default for IrToTaffyBuilder {
    fn default() -> Self {
        Self {
            responsive_dimensions_and_detail: vec![
                DimensionAndLod::default_sm(),
                DimensionAndLod::default_md(),
                DimensionAndLod::default_lg(),
            ],
        }
    }
}

impl IrToTaffyBuilder {
    /// Returns a `TaffyTreeAndRoot` with all processes as part of the diagram.
    ///
    /// This includes the processes container. Clicking on each process node
    /// reveals the process steps.
    pub fn build(self) -> Result<TaffyTreeAndRoot, TaffyError> {
        let mut taffy_tree = TaffyTree::new();
        let root = taffy_tree.new_leaf_with_context(Style::default(), NodeContext {})?;
        Ok(TaffyTreeAndRoot { taffy_tree, root })
    }
}
