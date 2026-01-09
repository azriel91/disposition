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
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct TagId<'s>(Id<'s>);

impl<'s> TagId<'s> {
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
    pub fn new(id: &'s str) -> Result<Self, IdInvalidFmt<'s>> {
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
    pub fn into_inner(self) -> Id<'s> {
        self.0
    }
}

impl<'s> From<Id<'s>> for TagId<'s> {
    fn from(id: Id<'s>) -> Self {
        TagId(id)
    }
}

impl<'s> AsRef<Id<'s>> for TagId<'s> {
    fn as_ref(&self) -> &Id<'s> {
        &self.0
    }
}

impl<'s> Borrow<Id<'s>> for TagId<'s> {
    fn borrow(&self) -> &Id<'s> {
        &self.0
    }
}

impl<'s> Deref for TagId<'s> {
    type Target = Id<'s>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'s> DerefMut for TagId<'s> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'s> Display for TagId<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
