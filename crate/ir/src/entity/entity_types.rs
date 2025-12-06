use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::{
    common::{Id, Map},
    entity::EntityTypeId,
};

/// Entity types attached to things, edges, processes, process steps and tags
/// for common styling.
///
/// Each entity can have multiple types, allowing styles to be stacked.
/// This differs from the input schema where each entity can only have one type.
///
/// # Example
///
/// ```yaml
/// entity_types:
///   # things
///   t_aws: [type_thing_default, type_organisation]
///   t_aws_iam: [type_thing_default, type_service]
///   t_aws_iam_ecs_policy: [type_thing_default]
///   t_aws_ecr: [type_thing_default, type_service]
///   t_aws_ecr_repo: [type_thing_default]
///   t_aws_ecr_repo_image_1: [type_thing_default, type_docker_image]
///   t_aws_ecr_repo_image_2: [type_thing_default, type_docker_image]
///
///   # tags
///   tag_app_development: [tag_type_default]
///   tag_deployment: [tag_type_default]
///
///   # processes
///   proc_app_dev: [type_process_default]
///   proc_app_release: [type_process_default]
///
///   # process steps
///   proc_app_dev_step_repository_clone: [type_process_step_default]
///   proc_app_dev_step_project_build: [type_process_step_default]
///
///   # edges
///   edge_t_localhost__t_github_user_repo__pull__0:
///     [type_edge_dependency_cyclic_default, type_edge_interaction_cyclic_default]
///   edge_t_localhost__t_github_user_repo__push__0:
///     [type_edge_dependency_sequence_request_default, type_edge_interaction_sequence_request_default]
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct EntityTypes(Map<Id, Vec<EntityTypeId>>);

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
    pub fn into_inner(self) -> Map<Id, Vec<EntityTypeId>> {
        self.0
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for EntityTypes {
    type Target = Map<Id, Vec<EntityTypeId>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EntityTypes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Map<Id, Vec<EntityTypeId>>> for EntityTypes {
    fn from(inner: Map<Id, Vec<EntityTypeId>>) -> Self {
        Self(inner)
    }
}

impl FromIterator<(Id, Vec<EntityTypeId>)> for EntityTypes {
    fn from_iter<I: IntoIterator<Item = (Id, Vec<EntityTypeId>)>>(iter: I) -> Self {
        Self(Map::from_iter(iter))
    }
}
