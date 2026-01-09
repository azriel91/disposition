use std::ops::{Deref, DerefMut};

use disposition_model_common::{Id, Map};
use serde::{Deserialize, Serialize};

use crate::thing::ThingId;

/// Hierarchy of `thing`s as a recursive tree structure.
///
/// The `ThingHierarchy` is a tree structure stored as a map of `ThingId` to
/// `ThingHierarchy`. This structure is strictly unidirectional (no cycles).
///
/// This defines the nesting of things, which affects:
/// * Visual containment in the diagram
/// * The order of declaration affects the position of the `thing` in a flex box
///
/// # Example
///
/// ```yaml
/// thing_hierarchy:
///   t_aws: # <-- `ThingHierarchy` (recursive)
///     t_aws_iam: # <-- `ThingHierarchy` (recursive)
///       t_aws_iam_ecs_policy: {}
///     t_aws_ecr:
///       t_aws_ecr_repo:
///         t_aws_ecr_repo_image_1: {}
///         t_aws_ecr_repo_image_2: {}
///
///   t_github:
///     t_github_user_repo: {}
///
///   t_localhost:
///     t_localhost_repo:
///       t_localhost_repo_src: {}
///       t_localhost_repo_target:
///         t_localhost_repo_target_file_zip: {}
///         t_localhost_repo_target_dist_dir: {}
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThingHierarchy<'id>(Map<ThingId<'id>, ThingHierarchy<'id>>);

impl<'id> ThingHierarchy<'id> {
    /// Returns a new empty `ThingHierarchy`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ThingHierarchy` with the given preallocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<ThingId<'id>, ThingHierarchy<'id>> {
        self.0
    }

    /// Returns true if this hierarchy node has no children.
    pub fn is_leaf(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of direct children of this hierarchy node.
    pub fn children_count(&self) -> usize {
        self.0.len()
    }

    /// Recursively counts all descendant things in this hierarchy.
    pub fn total_descendants(&self) -> usize {
        self.0
            .values()
            .map(|child| 1 + child.total_descendants())
            .sum()
    }

    /// Returns true if the hierarchy is empty (no children).
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this contains the hierarchy for a thing with the given
    /// ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id<'id>>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl<'id> Deref for ThingHierarchy<'id> {
    type Target = Map<ThingId<'id>, ThingHierarchy<'id>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'id> DerefMut for ThingHierarchy<'id> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'id> From<Map<ThingId<'id>, ThingHierarchy<'id>>> for ThingHierarchy<'id> {
    fn from(inner: Map<ThingId<'id>, ThingHierarchy<'id>>) -> Self {
        Self(inner)
    }
}

impl<'id> FromIterator<(ThingId<'id>, ThingHierarchy<'id>)> for ThingHierarchy<'id> {
    fn from_iter<I: IntoIterator<Item = (ThingId<'id>, ThingHierarchy<'id>)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
