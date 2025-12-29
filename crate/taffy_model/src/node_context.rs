use disposition_model_common::{entity::EntityType, Id};

use crate::CosmicTextContext;

/// Data stored with each node in the taffy tree.
#[derive(Clone, Debug)]
pub struct NodeContext {
    /// ID of the entity from the input / IR diagram.
    pub entity_id: Id,
    /// Tracks whether this is a `thing`, `process`, `tag`, etc.
    ///
    /// This should be the default type assigned to an entity, so that we can
    /// tell if it is a `thing`, `process`, `process_step`, `tag`, `edge_group`,
    /// or `edge`.
    pub entity_type: EntityType,
    /// Context for rendering text within a `taffy` leaf node.
    pub cosmic_text_context: CosmicTextContext,
}
