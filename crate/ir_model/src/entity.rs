pub use disposition_model_common::entity::{EntityDescs, EntityTypeId};

pub use self::{
    entity_tailwind_classes::EntityTailwindClasses, entity_type::EntityType,
    entity_types::EntityTypes,
};

mod entity_tailwind_classes;
mod entity_type;
mod entity_types;
