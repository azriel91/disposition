use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::edge::Edge;

/// A group of related edges.
///
/// An edge group contains one or more edges that share a logical
/// relationship. For example, a bidirectional connection between two nodes
/// would be represented as an edge group with two edges going in opposite
/// directions.
///
/// # Example
///
/// ```yaml
/// edge_groups:
///   edge_t_localhost__t_github_user_repo:
///     - from: t_github_user_repo
///       to: t_localhost
///     - from: t_localhost
///       to: t_github_user_repo
///   edge_t_localhost__t_github_user_repo__push:
///     - from: t_localhost
///       to: t_github_user_repo
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct EdgeGroup(Vec<Edge>);

impl EdgeGroup {
    /// Returns a new empty `EdgeGroup`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `EdgeGroup` with the given preallocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    /// Returns the underlying vector.
    pub fn into_inner(self) -> Vec<Edge> {
        self.0
    }

    /// Returns true if the group is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of edges in this group.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Deref for EdgeGroup {
    type Target = Vec<Edge>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EdgeGroup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<Edge>> for EdgeGroup {
    fn from(inner: Vec<Edge>) -> Self {
        Self(inner)
    }
}

impl FromIterator<Edge> for EdgeGroup {
    fn from_iter<I: IntoIterator<Item = Edge>>(iter: I) -> Self {
        Self(Vec::from_iter(iter))
    }
}
