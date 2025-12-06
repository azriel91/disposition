use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{Id, IdInvalidFmt};

/// Unique identifier for an entity type in the diagram, [`Id`] newtype.
///
/// Entity types allow styling of things, edges, processes, process steps, and
/// tags in common based on their type.
///
/// Must begin with a letter or underscore, and contain only letters, numbers,
/// and underscores.
///
/// # Examples
///
/// ```rust
/// use disposition_model_common::{entity::EntityTypeId, id, Id};
///
/// let entity_type_id: EntityTypeId = id!("type_organisation").into();
///
/// assert_eq!(entity_type_id.as_str(), "type_organisation");
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct EntityTypeId(Id);

impl EntityTypeId {
    /// Creates a new [`EntityTypeId`] from a string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_model_common::{entity::EntityTypeId, Id};
    ///
    /// let entity_type_id = EntityTypeId::new("type_organisation").unwrap();
    ///
    /// assert_eq!(entity_type_id.as_str(), "type_organisation");
    /// ```
    pub fn new(id: &'static str) -> Result<Self, IdInvalidFmt<'static>> {
        Id::new(id).map(EntityTypeId)
    }

    /// Returns the underlying [`Id`] value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_model_common::{entity::EntityTypeId, Id};
    ///
    /// let entity_type_id = EntityTypeId::new("type_organisation").unwrap();
    ///
    /// assert_eq!(
    ///     entity_type_id.into_inner(),
    ///     Id::new("type_organisation").unwrap()
    /// );
    /// ```
    pub fn into_inner(self) -> Id {
        self.0
    }
}

impl From<Id> for EntityTypeId {
    fn from(id: Id) -> Self {
        EntityTypeId(id)
    }
}

impl Deref for EntityTypeId {
    type Target = Id;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EntityTypeId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
