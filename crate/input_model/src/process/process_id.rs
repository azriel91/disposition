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
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProcessId(Id);

impl ProcessId {
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
    pub fn new(id: &'static str) -> Result<Self, IdInvalidFmt<'static>> {
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
    pub fn into_inner(self) -> Id {
        self.0
    }
}

impl From<Id> for ProcessId {
    fn from(id: Id) -> Self {
        ProcessId(id)
    }
}

impl AsRef<Id> for ProcessId {
    fn as_ref(&self) -> &Id {
        &self.0
    }
}

impl Borrow<Id> for ProcessId {
    fn borrow(&self) -> &Id {
        &self.0
    }
}

impl Deref for ProcessId {
    type Target = Id;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ProcessId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for ProcessId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
