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
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProcessStepId<'s>(Id<'s>);

impl<'s> ProcessStepId<'s> {
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
    pub fn new(id: &'s str) -> Result<Self, IdInvalidFmt<'s>> {
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
    pub fn into_inner(self) -> Id<'s> {
        self.0
    }
}

impl<'s> From<Id<'s>> for ProcessStepId<'s> {
    fn from(id: Id<'s>) -> Self {
        ProcessStepId(id)
    }
}

impl<'s> AsRef<Id<'s>> for ProcessStepId<'s> {
    fn as_ref(&self) -> &Id<'s> {
        &self.0
    }
}

impl<'s> Borrow<Id<'s>> for ProcessStepId<'s> {
    fn borrow(&self) -> &Id<'s> {
        &self.0
    }
}

impl<'s> Deref for ProcessStepId<'s> {
    type Target = Id<'s>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'s> DerefMut for ProcessStepId<'s> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'s> Display for ProcessStepId<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
