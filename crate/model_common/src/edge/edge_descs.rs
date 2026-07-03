use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{Id, Map};

use super::{EdgeGroupId, EdgeId};

/// Descriptions for edges and edge groups.
///
/// This map contains text (typically markdown) that provides additional
/// context about edges in the diagram. These descriptions can be displayed
/// when an edge or edge group is focused.
///
/// A key may be either an edge group ID, which applies to every edge in the
/// group, or a specific edge instance ID (`{edge_group_id}__{edge_index}`),
/// which overrides the group's description for that one edge -- see
/// [`Self::get_for_edge`].
///
/// # Example
///
/// ```yaml
/// edge_descs:
///   # edge groups
///   #
///   # Shown when any of the edges in this group are focused.
///   edge_t_localhost__t_github_user_repo__pull: |-
///     Fetch from GitHub
///
///   # edges
///   edge_t_localhost__t_github_user_repo__pull__0: |-
///     `git pull`
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct EdgeDescs<'id>(Map<Id<'id>, String>);

impl<'id> EdgeDescs<'id> {
    /// Returns a new `EdgeDescs` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `EdgeDescs` map with the given preallocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<Id<'id>, String> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Converts this `EdgeDescs` into one with a `'static` lifetime.
    ///
    /// If any inner `Cow` is borrowed, this will clone the string to create
    /// an owned version.
    pub fn into_static(self) -> EdgeDescs<'static> {
        EdgeDescs(
            self.0
                .into_iter()
                .map(|(id, desc)| (id.into_static(), desc))
                .collect(),
        )
    }

    /// Returns true if this contains a description for an edge with the given
    /// ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'id>>,
    {
        self.0.contains_key(id.as_ref())
    }

    /// Returns the description for an edge, checking the edge's own instance
    /// ID first (e.g. `edge_t_a__t_b__0`), and falling back to its edge group
    /// ID (e.g. `edge_t_a__t_b`) if the instance has no entry of its own.
    ///
    /// This is the precedence rule documented on [`EdgeDescs`]: a description
    /// keyed by the group ID applies to every edge instance in the group, but
    /// a description keyed by a specific instance ID overrides it for that
    /// edge.
    pub fn get_for_edge(
        &self,
        edge_id: &EdgeId<'id>,
        edge_group_id: &EdgeGroupId<'id>,
    ) -> Option<&String> {
        self.0
            .get(edge_id.as_ref())
            .or_else(|| self.0.get(edge_group_id.as_ref()))
    }
}

impl<'id> Deref for EdgeDescs<'id> {
    type Target = Map<Id<'id>, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for EdgeDescs<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<Id<'id>, String>> for EdgeDescs<'id> {
    fn from(inner: Map<Id<'id>, String>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(Id<'id>, String)> for EdgeDescs<'id> {
    fn from_iter<I: IntoIterator<Item = (Id<'id>, String)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
