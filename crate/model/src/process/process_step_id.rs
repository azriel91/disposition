use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::common::{Id, IdInvalidFmt};

/// Unique identifier for a process step in the diagram, [`Id`] newtype.
///
/// Must begin with a letter or underscore, and contain only letters, numbers,
/// and underscores.
///
/// # Examples
///
/// ```rust
/// use disposition_model::{
///     common::{id, Id},
///     process::ProcessStepId,
/// };
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
    /// use disposition_model::{common::Id, process::ProcessStepId};
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
    /// use disposition_model::{common::Id, process::ProcessStepId};
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
