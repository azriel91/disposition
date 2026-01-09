use std::{
    borrow::Borrow,
    fmt::{self, Display},
    ops::{Deref, DerefMut},
};

use disposition_model_common::{Id, IdInvalidFmt};
use serde::{Deserialize, Serialize};

/// Unique identifier for a node in the diagram, [`Id`] newtype.
///
/// A node can represent a thing, tag, process, or process step.
///
/// Must begin with a letter or underscore, and contain only letters, numbers,
/// and underscores.
///
/// # Examples
///
/// ```rust
/// use disposition_ir_model::node::NodeId;
/// use disposition_model_common::{id, Id};
///
/// let node_id: NodeId = id!("example_id").into();
///
/// assert_eq!(node_id.as_str(), "example_id");
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct NodeId<'s>(Id<'s>);

impl<'s> NodeId<'s> {
    /// Creates a new [`NodeId`] from a string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_ir_model::node::NodeId;
    /// use disposition_model_common::Id;
    ///
    /// let node_id = NodeId::new("example_id").unwrap();
    ///
    /// assert_eq!(node_id.as_str(), "example_id");
    /// ```
    pub fn new(id: &'s str) -> Result<Self, IdInvalidFmt<'s>> {
        Id::new(id).map(NodeId)
    }

    /// Returns the underlying [`Id`] value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_ir_model::node::NodeId;
    /// use disposition_model_common::Id;
    ///
    /// let node_id = NodeId::new("example_id").unwrap();
    ///
    /// assert_eq!(node_id.into_inner(), Id::new("example_id").unwrap());
    /// ```
    pub fn into_inner(self) -> Id<'s> {
        self.0
    }
}

impl<'s> From<Id<'s>> for NodeId<'s> {
    fn from(id: Id<'s>) -> Self {
        NodeId(id)
    }
}

impl<'s> AsRef<Id<'s>> for NodeId<'s> {
    fn as_ref(&self) -> &Id<'s> {
        &self.0
    }
}

impl<'s> Borrow<Id<'s>> for NodeId<'s> {
    fn borrow(&self) -> &Id<'s> {
        &self.0
    }
}

impl<'s> Deref for NodeId<'s> {
    type Target = Id<'s>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'s> DerefMut for NodeId<'s> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'s> Display for NodeId<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
