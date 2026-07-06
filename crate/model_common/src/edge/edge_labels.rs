use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::Map;

use super::{EdgeGroupId, EdgeId, EdgeLabel};

/// Text labels for edges at each endpoint.
///
/// Each entry maps an edge instance ID (`{edge_group_id}__{edge_index}`), or
/// an edge group ID (applies to every edge in the group), to its `from` and
/// `to` endpoint labels -- see [`Self::get_for_edge`] for the lookup
/// precedence when both are present. Both labels are optional -- set them to
/// an empty string to show no text at that endpoint.
///
/// # Example
///
/// ```yaml
/// edge_labels:
///   # edge group -- applies to every edge in `edge_t_localhost__t_github_user_repo__pull`.
///   edge_t_localhost__t_github_user_repo__pull:
///     from: "local branch"
///     to: "remote branch"
///
///   # edge instance -- overrides the group label for this one edge.
///   edge_t_localhost__t_github_user_repo__pull__0:
///     from: "local branch"
///     to: "remote branch"
///   edge_t_localhost__t_github_user_repo__push__0:
///     from: "local commit"
///     to: ""
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct EdgeLabels<'id>(Map<EdgeId<'id>, EdgeLabel>);

impl<'id> EdgeLabels<'id> {
    /// Returns a new `EdgeLabels` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `EdgeLabels` map with the given preallocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<EdgeId<'id>, EdgeLabel> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Converts this `EdgeLabels` into one with a `'static` lifetime.
    ///
    /// If any inner `Cow` is borrowed, this will clone the string to create
    /// an owned version.
    pub fn into_static(self) -> EdgeLabels<'static> {
        EdgeLabels(
            self.0
                .into_iter()
                .map(|(edge_id, label)| (edge_id.into_static(), label))
                .collect(),
        )
    }

    /// Returns the label for an edge, checking the edge's own instance ID
    /// first, and falling back to its edge group ID if the instance has no
    /// entry of its own.
    ///
    /// Mirrors [`EdgeDescs::get_for_edge`](super::EdgeDescs::get_for_edge) --
    /// see its docs for the precedence rule.
    pub fn get_for_edge(
        &self,
        edge_id: &EdgeId<'id>,
        edge_group_id: &EdgeGroupId<'id>,
    ) -> Option<&EdgeLabel> {
        self.0
            .get(edge_id)
            .or_else(|| self.0.get(edge_group_id.as_ref()))
    }
}

impl<'id> Deref for EdgeLabels<'id> {
    type Target = Map<EdgeId<'id>, EdgeLabel>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for EdgeLabels<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<EdgeId<'id>, EdgeLabel>> for EdgeLabels<'id> {
    fn from(inner: Map<EdgeId<'id>, EdgeLabel>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(EdgeId<'id>, EdgeLabel)> for EdgeLabels<'id> {
    fn from_iter<I: IntoIterator<Item = (EdgeId<'id>, EdgeLabel)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
