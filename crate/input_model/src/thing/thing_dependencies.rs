use std::ops::{Deref, DerefMut};

use disposition_model_common::Map;
use serde::{Deserialize, Serialize};

use crate::edge::{EdgeId, EdgeKind};

/// Dependencies between things can be one way, or cyclic.
///
/// Dependencies are static relationships between things, and should be rendered
/// as "on" or "off" depending on whether a `thing` is focused / targeted, and
/// whether the user wants to see:
///
/// * Predecessors / successors linked to this thing.
/// * Immediate dependencies vs transitive (maybe closest `n` neighbours).
///
/// * When B depends on A, it means A must exist before B.
/// * Changes to A means B is out of date.
///
/// How we render dependencies (forward / backward / undirected / bidirectional
/// arrows) can be defined separately from the meaning of the dependency.
///
/// # Example
///
/// ```yaml
/// thing_dependencies:
///   edge_t_localhost__t_github_user_repo__pull:
///     cyclic:
///       - t_localhost
///       - t_github_user_repo
///   edge_t_localhost__t_github_user_repo__push:
///     sequence:
///       - t_localhost
///       - t_github_user_repo
///   edge_t_localhost__t_localhost__within:
///     cyclic:
///       - t_localhost
///   edge_t_github_user_repo__t_aws_ecr_repo__push:
///     sequence:
///       - t_github_user_repo
///       - t_aws_ecr_repo
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThingDependencies(Map<EdgeId, EdgeKind>);

impl ThingDependencies {
    /// Returns a new `ThingDependencies` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ThingDependencies` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<EdgeId, EdgeKind> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for ThingDependencies {
    type Target = Map<EdgeId, EdgeKind>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ThingDependencies {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<EdgeId, EdgeKind>> for ThingDependencies {
    fn from(inner: Map<EdgeId, EdgeKind>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(EdgeId, EdgeKind)> for ThingDependencies {
    fn from_iter<I: IntoIterator<Item = (EdgeId, EdgeKind)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
