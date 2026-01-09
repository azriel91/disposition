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
pub struct EdgeGroupId<'s>(Id<'s>);

impl<'s> EdgeGroupId<'s> {
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
    pub fn new(id: &'s str) -> Result<Self, IdInvalidFmt<'s>> {
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
    pub fn into_inner(self) -> Id<'s> {
        self.0
    }
}

impl<'s> From<Id<'s>> for EdgeGroupId<'s> {
    fn from(id: Id<'s>) -> Self {
        EdgeGroupId(id)
    }
}

impl<'s> AsRef<Id<'s>> for EdgeGroupId<'s> {
    fn as_ref(&self) -> &Id<'s> {
        &self.0
    }
}

impl<'s> Borrow<Id<'s>> for EdgeGroupId<'s> {
    fn borrow(&self) -> &Id<'s> {
        &self.0
    }
}

impl<'s> Deref for EdgeGroupId<'s> {
    type Target = Id<'s>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'s> DerefMut for EdgeGroupId<'s> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'s> Display for EdgeGroupId<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
