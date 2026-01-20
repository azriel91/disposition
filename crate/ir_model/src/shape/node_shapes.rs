use std::ops::{Deref, DerefMut};

use disposition_model_common::{Id, Map};
use serde::{Deserialize, Serialize};

use crate::{node::NodeId, shape::NodeShape};

/// Map of node IDs to their shape configurations.
///
/// This map defines the shape for each node in the diagram. Currently only
/// rectangular shapes with corner radii are supported.
///
/// # Example
///
/// ```yaml
/// node_shapes:
///   t_aws:
///     rect:
///       top_left: 4.0
///       top_right: 4.0
///       bottom_left: 4.0
///       bottom_right: 4.0
///   t_localhost:
///     rect:
///       top_left: 8.0
///       top_right: 8.0
///       bottom_left: 8.0
///       bottom_right: 8.0
///   # Sharp corners (default)
///   t_aws_iam_ecs_policy:
///     rect:
///       top_left: 0.0
///       top_right: 0.0
///       bottom_left: 0.0
///       bottom_right: 0.0
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct NodeShapes<'id>(Map<NodeId<'id>, NodeShape>);

impl<'id> NodeShapes<'id> {
    /// Returns a new `NodeShapes` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `NodeShapes` map with the given preallocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<NodeId<'id>, NodeShape> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Converts this `NodeShapes` into one with a `'static` lifetime.
    ///
    /// If any inner `Cow` is borrowed, this will clone the string to create
    /// an owned version.
    pub fn into_static(self) -> NodeShapes<'static> {
        NodeShapes(
            self.0
                .into_iter()
                .map(|(node_id, shape)| (node_id.into_static(), shape))
                .collect(),
        )
    }

    /// Returns true if this contains shape information for a node with the
    /// given ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'id>>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl<'id> Deref for NodeShapes<'id> {
    type Target = Map<NodeId<'id>, NodeShape>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for NodeShapes<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<NodeId<'id>, NodeShape>> for NodeShapes<'id> {
    fn from(inner: Map<NodeId<'id>, NodeShape>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(NodeId<'id>, NodeShape)> for NodeShapes<'id> {
    fn from_iter<I: IntoIterator<Item = (NodeId<'id>, NodeShape)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
