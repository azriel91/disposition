use std::{
    borrow::Borrow,
    fmt::{self, Display},
    ops::{Deref, DerefMut},
};

use disposition_model_common::{Id, IdInvalidFmt};
use serde::{Deserialize, Serialize};

/// Unique identifier for a thing in the diagram, [`Id`] newtype.
///
/// Must begin with a letter or underscore, and contain only letters, numbers,
/// and underscores.
///
/// # Examples
///
/// ```rust
/// use disposition_input_model::thing::ThingId;
/// use disposition_model_common::{id, Id};
///
/// let thing_id: ThingId = id!("example_id").into();
///
/// assert_eq!(thing_id.as_str(), "example_id");
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThingId<'s>(Id<'s>);

impl<'s> ThingId<'s> {
    /// Creates a new [`ThingId`] from a string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_input_model::thing::ThingId;
    /// use disposition_model_common::Id;
    ///
    /// let thing_id = ThingId::new("example_id").unwrap();
    ///
    /// assert_eq!(thing_id.as_str(), "example_id");
    /// ```
    pub fn new(id: &'s str) -> Result<Self, IdInvalidFmt<'s>> {
        Id::new(id).map(ThingId)
    }

    /// Returns the underlying [`Id`] value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_input_model::thing::ThingId;
    /// use disposition_model_common::Id;
    ///
    /// let thing_id = ThingId::new("example_id").unwrap();
    ///
    /// assert_eq!(thing_id.into_inner(), Id::new("example_id").unwrap());
    /// ```
    pub fn into_inner(self) -> Id<'s> {
        self.0
    }
}

impl<'s> From<Id<'s>> for ThingId<'s> {
    fn from(id: Id<'s>) -> Self {
        ThingId(id)
    }
}

impl<'s> AsRef<Id<'s>> for ThingId<'s> {
    fn as_ref(&self) -> &Id<'s> {
        &self.0
    }
}

impl<'s> Borrow<Id<'s>> for ThingId<'s> {
    fn borrow(&self) -> &Id<'s> {
        &self.0
    }
}

impl<'s> Deref for ThingId<'s> {
    type Target = Id<'s>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'s> DerefMut for ThingId<'s> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'s> Display for ThingId<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
