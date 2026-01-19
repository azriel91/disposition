use std::ops::{Deref, DerefMut};

use disposition_model_common::{Id, Map};
use serde::{Deserialize, Serialize};

use crate::{layout::NodeLayout, node::NodeId};

/// Map of node IDs to their layout configurations.
///
/// This map defines how each node's children should be arranged. Container
/// nodes use `NodeLayout::Flex` to specify flex layout parameters, while
/// leaf nodes use `NodeLayout::None`.
///
/// The layout cascades: if not specified, a node inherits from its parent
/// or uses the default layout.
///
/// # Example
///
/// ```yaml
/// node_layout:
///   # Root container
///   _root:
///     flex:
///       direction: "column_reverse"
///       wrap: true
///       padding_top: 4.0
///       padding_right: 4.0
///       padding_bottom: 4.0
///       padding_left: 4.0
///       margin_top: 0.0
///       margin_right: 0.0
///       margin_bottom: 0.0
///       margin_left: 0.0
///       gap: 4.0
///
///   # Processes container (groups all processes horizontally)
///   _processes_container:
///     flex:
///       direction: "row"
///       wrap: true
///       padding_top: 4.0
///       padding_right: 4.0
///       padding_bottom: 4.0
///       padding_left: 4.0
///       margin_top: 0.0
///       margin_right: 0.0
///       margin_bottom: 0.0
///       margin_left: 0.0
///       gap: 4.0
///
///   # Individual processes (steps stacked vertically)
///   proc_app_dev:
///     flex:
///       direction: "column"
///       wrap: false
///       padding_top: 2.0
///       padding_right: 2.0
///       padding_bottom: 2.0
///       padding_left: 2.0
///       margin_top: 0.0
///       margin_right: 0.0
///       margin_bottom: 0.0
///       margin_left: 0.0
///       gap: 2.0
///
///   # Process steps are leaves
///   proc_app_dev_step_repository_clone: none
///   proc_app_dev_step_project_build: none
///
///   # Tags container
///   _tags_container:
///     flex:
///       direction: "row"
///       wrap: true
///       padding_top: 2.0
///       padding_right: 2.0
///       padding_bottom: 2.0
///       padding_left: 2.0
///       margin_top: 0.0
///       margin_right: 0.0
///       margin_bottom: 0.0
///       margin_left: 0.0
///       gap: 2.0
///
///   # Tags are leaves
///   tag_app_development: none
///   tag_deployment: none
///
///   # Things container
///   _things_container:
///     flex:
///       direction: "row"
///       wrap: true
///       padding_top: 4.0
///       padding_right: 4.0
///       padding_bottom: 4.0
///       padding_left: 4.0
///       margin_top: 0.0
///       margin_right: 0.0
///       margin_bottom: 0.0
///       margin_left: 0.0
///       gap: 4.0
///
///   # Top-level things
///   t_aws:
///     flex:
///       direction: "column"
///       wrap: false
///       padding_top: 2.0
///       padding_right: 2.0
///       padding_bottom: 2.0
///       padding_left: 2.0
///       margin_top: 0.0
///       margin_right: 0.0
///       margin_bottom: 0.0
///       margin_left: 0.0
///       gap: 2.0
///   t_aws_iam_ecs_policy: none
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct NodeLayouts<'id>(Map<NodeId<'id>, NodeLayout>);

impl<'id> NodeLayouts<'id> {
    /// Returns a new `NodeLayouts` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `NodeLayouts` map with the given preallocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<NodeId<'id>, NodeLayout> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Converts this `NodeLayouts` into one with a `'static` lifetime.
    ///
    /// If any inner `Cow` is borrowed, this will clone the string to create
    /// an owned version.
    pub fn into_static(self) -> NodeLayouts<'static> {
        NodeLayouts(
            self.0
                .into_iter()
                .map(|(node_id, layout)| (node_id.into_static(), layout))
                .collect(),
        )
    }

    /// Returns true if this contains layout information for a node with the
    /// given ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'id>>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl<'id> Deref for NodeLayouts<'id> {
    type Target = Map<NodeId<'id>, NodeLayout>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for NodeLayouts<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<NodeId<'id>, NodeLayout>> for NodeLayouts<'id> {
    fn from(inner: Map<NodeId<'id>, NodeLayout>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(NodeId<'id>, NodeLayout)> for NodeLayouts<'id> {
    fn from_iter<I: IntoIterator<Item = (NodeId<'id>, NodeLayout)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
