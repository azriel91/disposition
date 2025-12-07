use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{Id, IdInvalidFmt};

/// Unique identifier for an edge in the diagram, [`Id`] newtype.
///
/// Must begin with a letter or underscore, and contain only letters, numbers,
/// and underscores.
///
/// # Examples
///
/// ```rust
/// use disposition_model_common::{edge::EdgeId, id, Id};
///
/// let edge_id: EdgeId = id!("edge_a_to_b").into();
///
/// assert_eq!(edge_id.as_str(), "edge_a_to_b");
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct EdgeId(Id);

impl EdgeId {
    /// Creates a new [`EdgeId`] from a string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_model_common::{edge::EdgeId, Id};
    ///
    /// let edge_id = EdgeId::new("edge_a_to_b").unwrap();
    ///
    /// assert_eq!(edge_id.as_str(), "edge_a_to_b");
    /// ```
    pub fn new(id: &'static str) -> Result<Self, IdInvalidFmt<'static>> {
        Id::new(id).map(EdgeId)
    }

    /// Returns the underlying [`Id`] value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_model_common::{edge::EdgeId, Id};
    ///
    /// let edge_id = EdgeId::new("edge_a_to_b").unwrap();
    ///
    /// assert_eq!(edge_id.into_inner(), Id::new("edge_a_to_b").unwrap());
    /// ```
    pub fn into_inner(self) -> Id {
        self.0
    }
}

impl From<Id> for EdgeId {
    fn from(id: Id) -> Self {
        EdgeId(id)
    }
}

impl Deref for EdgeId {
    type Target = Id;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EdgeId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
