use std::{
    borrow::Borrow,
    fmt::{self, Display},
    ops::{Deref, DerefMut},
};

use disposition_model_common::{Id, IdInvalidFmt};
use serde::{Deserialize, Serialize};

/// Unique identifier for a process in the diagram, [`Id`] newtype.
///
/// Must begin with a letter or underscore, and contain only letters, numbers,
/// and underscores.
///
/// # Examples
///
/// ```rust
/// use disposition_input_model::process::ProcessId;
/// use disposition_model_common::{id, Id};
///
/// let process_id: ProcessId = id!("example_id").into();
///
/// assert_eq!(process_id.as_str(), "example_id");
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProcessId<'s>(Id<'s>);

impl<'s> ProcessId<'s> {
    /// Creates a new [`ProcessId`] from a string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_input_model::process::ProcessId;
    /// use disposition_model_common::Id;
    ///
    /// let process_id = ProcessId::new("example_id").unwrap();
    ///
    /// assert_eq!(process_id.as_str(), "example_id");
    /// ```
    pub fn new(id: &'s str) -> Result<Self, IdInvalidFmt<'s>> {
        Id::new(id).map(ProcessId)
    }

    /// Returns the underlying [`Id`] value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_input_model::process::ProcessId;
    /// use disposition_model_common::Id;
    ///
    /// let process_id = ProcessId::new("example_id").unwrap();
    ///
    /// assert_eq!(process_id.into_inner(), Id::new("example_id").unwrap());
    /// ```
    pub fn into_inner(self) -> Id<'s> {
        self.0
    }
}

impl<'s> From<Id<'s>> for ProcessId<'s> {
    fn from(id: Id<'s>) -> Self {
        ProcessId(id)
    }
}

impl<'s> AsRef<Id<'s>> for ProcessId<'s> {
    fn as_ref(&self) -> &Id<'s> {
        &self.0
    }
}

impl<'s> Borrow<Id<'s>> for ProcessId<'s> {
    fn borrow(&self) -> &Id<'s> {
        &self.0
    }
}

impl<'s> Deref for ProcessId<'s> {
    type Target = Id<'s>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'s> DerefMut for ProcessId<'s> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'s> Display for ProcessId<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
