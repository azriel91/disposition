use std::{
    borrow::Borrow,
    fmt::{self, Display},
    ops::{Deref, DerefMut},
};

use disposition_model_common::{Id, IdInvalidFmt};
use serde::{Deserialize, Serialize};

/// Unique identifier for a tag in the diagram, [`Id`] newtype.
///
/// Must begin with a letter or underscore, and contain only letters, numbers,
/// and underscores.
///
/// # Examples
///
/// ```rust
/// use disposition_input_model::tag::TagId;
/// use disposition_model_common::{id, Id};
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
    /// use disposition_input_model::tag::TagId;
    /// use disposition_model_common::Id;
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
    /// use disposition_input_model::tag::TagId;
    /// use disposition_model_common::Id;
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

impl AsRef<Id> for TagId {
    fn as_ref(&self) -> &Id {
        &self.0
    }
}

impl Borrow<Id> for TagId {
    fn borrow(&self) -> &Id {
        &self.0
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

impl Display for TagId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
