use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::common::{Id, IdInvalidFmt};

/// Unique identifier for a tag in the diagram, [`Id`] newtype.
///
/// Must begin with a letter or underscore, and contain only letters, numbers,
/// and underscores.
///
/// # Examples
///
/// ```rust
/// use disposition_model::{
///     common::{id, Id},
///     tag::TagId,
/// };
///
/// let tag_id: TagId = id!("example_id").into();
///
/// assert_eq!(tag_id.as_str(), "example_id");
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct TagId(Id);

impl TagId {
    /// Creates a new [`TagId`] from a string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_model::{common::Id, tag::TagId};
    ///
    /// let tag_id = TagId::new("example_id").unwrap();
    ///
    /// assert_eq!(tag_id.as_str(), "example_id");
    /// ```
    pub fn new(id: &'static str) -> Result<Self, IdInvalidFmt<'static>> {
        Id::new(id).map(TagId)
    }

    /// Returns the underlying [`Id`] value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_model::{common::Id, tag::TagId};
    ///
    /// let tag_id = TagId::new("example_id").unwrap();
    ///
    /// assert_eq!(tag_id.into_inner(), Id::new("example_id").unwrap());
    /// ```
    pub fn into_inner(self) -> Id {
        self.0
    }
}

impl From<Id> for TagId {
    fn from(id: Id) -> Self {
        TagId(id)
    }
}

impl Deref for TagId {
    type Target = Id;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TagId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
