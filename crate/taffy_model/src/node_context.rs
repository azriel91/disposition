use disposition_model_common::{entity::EntityType, Id};

/// Data stored with each node in the taffy tree.
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
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
}
