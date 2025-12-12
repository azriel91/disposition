use std::{
    borrow::Borrow,
    fmt::{self, Display},
    ops::{Deref, DerefMut},
};

use disposition_model_common::{Id, IdInvalidFmt};
use serde::{Deserialize, Serialize};

/// Unique identifier for a process step in the diagram, [`Id`] newtype.
///
/// Must begin with a letter or underscore, and contain only letters, numbers,
/// and underscores.
///
/// # Examples
///
/// ```rust
/// use disposition_input_model::process::ProcessStepId;
/// use disposition_model_common::{id, Id};
///
/// let step_id: ProcessStepId = id!("step_clone_repo").into();
///
/// assert_eq!(step_id.as_str(), "step_clone_repo");
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProcessStepId(Id);

impl ProcessStepId {
    /// Creates a new [`ProcessStepId`] from a string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_input_model::process::ProcessStepId;
    /// use disposition_model_common::Id;
    ///
    /// let step_id = ProcessStepId::new("step_clone_repo").unwrap();
    ///
    /// assert_eq!(step_id.as_str(), "step_clone_repo");
    /// ```
    pub fn new(id: &'static str) -> Result<Self, IdInvalidFmt<'static>> {
        Id::new(id).map(ProcessStepId)
    }

    /// Returns the underlying [`Id`] value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_input_model::process::ProcessStepId;
    /// use disposition_model_common::Id;
    ///
    /// let step_id = ProcessStepId::new("step_clone_repo").unwrap();
    ///
    /// assert_eq!(step_id.into_inner(), Id::new("step_clone_repo").unwrap());
    /// ```
    pub fn into_inner(self) -> Id {
        self.0
    }
}

impl From<Id> for ProcessStepId {
    fn from(id: Id) -> Self {
        ProcessStepId(id)
    }
}

impl AsRef<Id> for ProcessStepId {
    fn as_ref(&self) -> &Id {
        &self.0
    }
}

impl Borrow<Id> for ProcessStepId {
    fn borrow(&self) -> &Id {
        &self.0
    }
}

impl Deref for ProcessStepId {
    type Target = Id;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ProcessStepId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for ProcessStepId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
