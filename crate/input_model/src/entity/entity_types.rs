use std::ops::{Deref, DerefMut};

use disposition_model_common::{entity::EntityTypeId, Id, Map};
use serde::{Deserialize, Serialize};

/// Entity types attached to things, edges, processes, process steps and tags
/// for common styling.
///
/// Entity types are like tags, but don't require the user to click on the tag
/// to apply the style.
///
/// Built-in types that are automatically attached to entities unless
/// overridden:
///
/// * `type_thing_default`
/// * `type_tag_default`
/// * `type_process_default`
/// * `type_process_step_default`
///
/// For edges, multiple edges are generated for each dependency / interaction,
/// and each edge is assigned a type from the following:
///
/// * `type_edge_dependency_sequence_request_default`
/// * `type_edge_dependency_sequence_response_default`
/// * `type_edge_dependency_cyclic_default`
/// * `type_edge_interaction_sequence_request_default`
/// * `type_edge_interaction_sequence_response_default`
/// * `type_edge_interaction_cyclic_default`
///
/// The edge ID will be the edge group ID specified in `thing_dependencies` /
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
///   t_aws: "type_organisation"
///   t_aws_iam: "type_service"
///   t_aws_ecr: "type_service"
///   t_aws_ecr_repo_image_1: "type_docker_image"
///   t_aws_ecr_repo_image_2: "type_docker_image"
///   t_github: "type_organisation"
///   tag_app_development: tag_type_default
///   edge_t_localhost__t_github_user_repo__pull__0: >-
///     "type_edge_dependency_sequence_request_default"
///   edge_t_localhost__t_github_user_repo__pull__1: >-
///     "type_edge_dependency_sequence_response_default"
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct EntityTypes(Map<Id, EntityTypeId>);

impl EntityTypes {
    /// Returns a new `EntityTypes` map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `EntityTypes` map with the given preallocated
    /// capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Map::with_capacity(capacity))
    }

    /// Returns the underlying map.
    pub fn into_inner(self) -> Map<Id, EntityTypeId> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns true if this contains types for any entity with the given ID.
    pub fn contains_key<IdT>(&self, id: &IdT) -> bool
    where
        IdT: AsRef<Id>,
    {
        self.0.contains_key(id.as_ref())
    }
}

impl Deref for EntityTypes {
    type Target = Map<Id, EntityTypeId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EntityTypes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<Id, EntityTypeId>> for EntityTypes {
    fn from(inner: Map<Id, EntityTypeId>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(Id, EntityTypeId)> for EntityTypes {
    fn from_iter<I: IntoIterator<Item = (Id, EntityTypeId)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
