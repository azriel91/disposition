use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{Id, Map, Set};

use super::EntityType;

/// Entity types attached to things, edges, processes, process steps and tags
/// for common styling.
///
/// Each entity can have multiple types, allowing styles to be stacked.
/// The types are appended to the entity's default type (e.g.
/// `type_thing_default` for things, `type_tag_default` for tags, etc.).
///
/// Built-in types that are automatically attached to entities unless
/// overridden:
///
/// * `type_thing_default`
/// * `type_tag_default`
/// * `type_process_default`
/// * `type_process_step_default`
///
/// For edges, an edge group is generated for each dependency / interaction:
///
/// Each edge group is assigned a type from the following:
///
/// * `type_dependency_edge_sequence_default`
/// * `type_dependency_edge_cyclic_default`
/// * `type_dependency_edge_symmetric_default`
/// * `type_interaction_edge_sequence_default`
/// * `type_interaction_edge_cyclic_default`
/// * `type_interaction_edge_symmetric_default`
///
/// and each edge within each edge group is assigned a type from the following:
///
/// * `type_dependency_edge_sequence_forward_default`
/// * `type_dependency_edge_cyclic_forward_default`
/// * `type_dependency_edge_symmetric_forward_default`
/// * `type_dependency_edge_symmetric_reverse_default`
/// * `type_interaction_edge_sequence_forward_default`
/// * `type_interaction_edge_cyclic_forward_default`
/// * `type_interaction_edge_symmetric_forward_default`
/// * `type_interaction_edge_symmetric_reverse_default`
///
/// The edge ID is the edge group ID specified in `thing_dependencies` /
/// `thing_interactions`, suffixed with the zero-based index of the edge like
/// so:
///
/// ```text
/// edge_id = edge_group_id + "__" + edge_index
/// ```
///
/// # Example
///
/// ```yaml
/// entity_types:
///   # things - types are appended to type_thing_default
///   t_aws:
///     - type_organisation
///   t_aws_iam:
///     - type_service
///   t_aws_ecr:
///     - type_service
///   t_aws_ecr_repo_image_1:
///     - type_docker_image
///   t_aws_ecr_repo_image_2:
///     - type_docker_image
///   t_github:
///     - type_organisation
///
///   # tags - types are appended to type_tag_default
///   tag_app_development:
///     - tag_type_default
///
///   # edges - types are appended to the edge's default type
///   edge_t_localhost__t_github_user_repo__pull__0:
///     - type_dependency_edge_cyclic_forward_default
///   edge_t_localhost__t_github_user_repo__pull__1:
///     - type_dependency_edge_cyclic_forward_default
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct EntityTypes(Map<Id, Set<EntityType>>);

impl EntityTypes {
    /// Returns a new `EntityTypes` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `EntityTypes` map with the given preallocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<Id, Set<EntityType>> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this contains type information for an entity with the
    /// given ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl Deref for EntityTypes {
    type Target = Map<Id, Set<EntityType>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EntityTypes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<Id, Set<EntityType>>> for EntityTypes {
    fn from(inner: Map<Id, Set<EntityType>>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(Id, Set<EntityType>)> for EntityTypes {
    fn from_iter<I: IntoIterator<Item = (Id, Set<EntityType>)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
