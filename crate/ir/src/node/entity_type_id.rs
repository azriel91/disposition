use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::common::{Id, IdInvalidFmt};

/// Unique identifier for an entity type, [`Id`] newtype.
///
/// Entity types are used to apply common styling to groups of nodes or edges.
/// Examples include `type_thing_default`, `type_tag_default`,
/// `type_process_default`, `type_organisation`, `type_service`, etc.
///
/// Must begin with a letter or underscore, and contain only letters, numbers,
/// and underscores.
///
/// # Examples
///
/// ```rust
/// use disposition_ir::{
///     common::{id, Id},
///     node::EntityTypeId,
/// };
///
/// let entity_type_id: EntityTypeId = id!("type_thing_default").into();
///
/// assert_eq!(entity_type_id.as_str(), "type_thing_default");
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
    /// use disposition_ir::{common::Id, node::EntityTypeId};
    ///
    /// let entity_type_id = EntityTypeId::new("type_thing_default").unwrap();
    ///
    /// assert_eq!(entity_type_id.as_str(), "type_thing_default");
    /// ```
    pub fn new(id: &'static str) -> Result<Self, IdInvalidFmt<'static>> {
        Id::new(id).map(EntityTypeId)
    }

    /// Returns the underlying [`Id`] value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_ir::{common::Id, node::EntityTypeId};
    ///
    /// let entity_type_id = EntityTypeId::new("type_thing_default").unwrap();
    ///
    /// assert_eq!(
    ///     entity_type_id.into_inner(),
    ///     Id::new("type_thing_default").unwrap()
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
