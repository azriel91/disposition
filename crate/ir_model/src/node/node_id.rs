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
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct NodeId(Id);

impl NodeId {
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
    pub fn new(id: &'static str) -> Result<Self, IdInvalidFmt<'static>> {
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
    pub fn into_inner(self) -> Id {
        self.0
    }
}

impl From<Id> for NodeId {
    fn from(id: Id) -> Self {
        NodeId(id)
    }
}

impl AsRef<Id> for NodeId {
    fn as_ref(&self) -> &Id {
        &self.0
    }
}

impl Borrow<Id> for NodeId {
    fn borrow(&self) -> &Id {
        &self.0
    }
}

impl Deref for NodeId {
    type Target = Id;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NodeId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
