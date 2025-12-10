use std::{
    borrow::Borrow,
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
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThingId(Id);

impl ThingId {
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
    pub fn new(id: &'static str) -> Result<Self, IdInvalidFmt<'static>> {
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
    pub fn into_inner(self) -> Id {
        self.0
    }
}

impl From<Id> for ThingId {
    fn from(id: Id) -> Self {
        ThingId(id)
    }
}

impl AsRef<Id> for ThingId {
    fn as_ref(&self) -> &Id {
        &self.0
    }
}

impl Borrow<Id> for ThingId {
    fn borrow(&self) -> &Id {
        &self.0
    }
}

impl Deref for ThingId {
    type Target = Id;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ThingId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
