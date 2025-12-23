use std::{
    borrow::Borrow,
    fmt::{self, Display},
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use crate::{Id, IdInvalidFmt};

/// Unique identifier for an edge group in the diagram, [`Id`] newtype.
///
/// Edge groups contain one or more edges.
///
/// Must begin with a letter or underscore, and contain only letters, numbers,
/// and underscores.
///
/// # Examples
///
/// ```rust
/// use disposition_model_common::{edge::EdgeGroupId, id, Id};
///
/// let edge_group_id: EdgeGroupId = id!("edge_t_localhost__t_github_user_repo__pull").into();
///
/// assert_eq!(
///     edge_group_id.as_str(),
///     "edge_t_localhost__t_github_user_repo__pull"
/// );
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct EdgeGroupId(Id);

impl EdgeGroupId {
    /// Creates a new [`EdgeGroupId`] from a string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_model_common::{edge::EdgeGroupId, Id};
    ///
    /// let edge_group_id = EdgeGroupId::new("edge_a_to_b").unwrap();
    ///
    /// assert_eq!(edge_group_id.as_str(), "edge_a_to_b");
    /// ```
    pub fn new(id: &'static str) -> Result<Self, IdInvalidFmt<'static>> {
        Id::new(id).map(EdgeGroupId)
    }

    /// Returns the underlying [`Id`] value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_model_common::{edge::EdgeGroupId, Id};
    ///
    /// let edge_group_id = EdgeGroupId::new("edge_a_to_b").unwrap();
    ///
    /// assert_eq!(edge_group_id.into_inner(), Id::new("edge_a_to_b").unwrap());
    /// ```
    pub fn into_inner(self) -> Id {
        self.0
    }
}

impl From<Id> for EdgeGroupId {
    fn from(id: Id) -> Self {
        EdgeGroupId(id)
    }
}

impl AsRef<Id> for EdgeGroupId {
    fn as_ref(&self) -> &Id {
        &self.0
    }
}

impl Borrow<Id> for EdgeGroupId {
    fn borrow(&self) -> &Id {
        &self.0
    }
}

impl Deref for EdgeGroupId {
    type Target = Id;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EdgeGroupId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for EdgeGroupId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
