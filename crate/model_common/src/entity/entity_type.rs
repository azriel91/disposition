use std::fmt::{self, Display};

use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

use crate::{id, Id};

/// Type for categorizing entities so they can all be styled consistently.
///
/// Entities can have multiple types, and later types' styles override earlier
/// ones.
///
/// This enum contains well-known type keys, with a `Custom` variant
/// for user-defined types.
///
/// # Examples
///
/// ```rust
/// use disposition_model_common::entity::EntityType;
///
/// let entity_type = EntityType::ThingDefault;
/// assert_eq!(entity_type.as_str(), "type_thing_default");
///
/// let custom_alias = EntityType::Custom("type_server".parse().unwrap());
/// assert_eq!(custom_alias.as_str(), "type_server");
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum EntityType {
    /// Inbuilt container.
    ContainerInbuilt,

    /// Default type for things
    ThingDefault,
    /// Default type for tags
    TagDefault,
    /// Default type for processes
    ProcessDefault,
    /// Default type for process_steps
    ProcessStepDefault,

    // Dependency edge groups
    /// Default type for dependency sequence edge groups
    DependencyEdgeSequenceDefault,
    /// Default type for dependency cyclic edge groups
    DependencyEdgeCyclicDefault,
    /// Default type for dependency symmetric edge groups
    DependencyEdgeSymmetricDefault,

    // Dependency edges
    /// Default type for dependency sequence edges
    DependencyEdgeSequenceForwardDefault,
    /// Default type for dependency cyclic edges
    DependencyEdgeCyclicForwardDefault,
    /// Default type for dependency symmetric forward edges
    DependencyEdgeSymmetricForwardDefault,
    /// Default type for dependency symmetric reverse edges
    DependencyEdgeSymmetricReverseDefault,

    // Interaction edge groups
    /// Default type for interaction sequence edge groups
    InteractionEdgeSequenceDefault,
    /// Default type for interaction cyclic edge groups
    InteractionEdgeCyclicDefault,
    /// Default type for interaction symmetric edge groups
    InteractionEdgeSymmetricDefault,

    // Interaction edges
    /// Default type for interaction sequence edges
    InteractionEdgeSequenceForwardDefault,
    /// Default type for interaction cyclic edges
    InteractionEdgeCyclicForwardDefault,
    /// Default type for interaction symmetric forward edges
    InteractionEdgeSymmetricForwardDefault,
    /// Default type for interaction symmetric reverse edges
    InteractionEdgeSymmetricReverseDefault,

    /// Custom user-defined type.
    Custom(Id),
}

impl EntityType {
    /// Returns the string representation of the style alias.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_model_common::entity::EntityType;
    ///
    /// assert_eq!(EntityType::ThingDefault.as_str(), "type_thing_default");
    /// assert_eq!(EntityType::TagDefault.as_str(), "type_tag_default");
    /// ```
    pub fn as_str(&self) -> &str {
        match self {
            EntityType::ContainerInbuilt => "container_inbuilt",
            EntityType::ThingDefault => "type_thing_default",
            EntityType::TagDefault => "type_tag_default",
            EntityType::ProcessDefault => "type_process_default",
            EntityType::ProcessStepDefault => "type_process_step_default",

            // Dependency edge groups
            EntityType::DependencyEdgeSequenceDefault => "type_dependency_edge_sequence_default",
            EntityType::DependencyEdgeCyclicDefault => "type_dependency_edge_cyclic_default",
            EntityType::DependencyEdgeSymmetricDefault => "type_dependency_edge_symmetric_default",

            // Dependency edges
            EntityType::DependencyEdgeSequenceForwardDefault => {
                "type_dependency_edge_sequence_forward_default"
            }
            EntityType::DependencyEdgeCyclicForwardDefault => {
                "type_dependency_edge_cyclic_forward_default"
            }
            EntityType::DependencyEdgeSymmetricForwardDefault => {
                "type_dependency_edge_symmetric_forward_default"
            }
            EntityType::DependencyEdgeSymmetricReverseDefault => {
                "type_dependency_edge_symmetric_reverse_default"
            }

            // Interaction edge groups
            EntityType::InteractionEdgeSequenceDefault => "type_interaction_edge_sequence_default",
            EntityType::InteractionEdgeCyclicDefault => "type_interaction_edge_cyclic_default",
            EntityType::InteractionEdgeSymmetricDefault => {
                "type_interaction_edge_symmetric_default"
            }

            // Interaction edges
            EntityType::InteractionEdgeSequenceForwardDefault => {
                "type_interaction_edge_sequence_forward_default"
            }
            EntityType::InteractionEdgeCyclicForwardDefault => {
                "type_interaction_edge_cyclic_forward_default"
            }
            EntityType::InteractionEdgeSymmetricForwardDefault => {
                "type_interaction_edge_symmetric_forward_default"
            }
            EntityType::InteractionEdgeSymmetricReverseDefault => {
                "type_interaction_edge_symmetric_reverse_default"
            }

            EntityType::Custom(id) => id.as_str(),
        }
    }

    /// Returns the ID representation of the style alias.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_model_common::{entity::EntityType, id, Id};
    ///
    /// assert_eq!(
    ///     EntityType::ThingDefault.into_id(),
    ///     id!("type_thing_default")
    /// );
    /// assert_eq!(EntityType::TagDefault.into_id(), id!("type_tag_default"));
    /// ```
    pub fn into_id(self) -> Id {
        match self {
            EntityType::ContainerInbuilt => id!("container_inbuilt"),
            EntityType::ThingDefault => id!("type_thing_default"),
            EntityType::TagDefault => id!("type_tag_default"),
            EntityType::ProcessDefault => id!("type_process_default"),
            EntityType::ProcessStepDefault => id!("type_process_step_default"),

            // Dependency edge groups
            EntityType::DependencyEdgeSequenceDefault => {
                id!("type_dependency_edge_sequence_default")
            }
            EntityType::DependencyEdgeCyclicDefault => {
                id!("type_dependency_edge_cyclic_default")
            }
            EntityType::DependencyEdgeSymmetricDefault => {
                id!("type_dependency_edge_symmetric_default")
            }

            // Dependency edges
            EntityType::DependencyEdgeSequenceForwardDefault => {
                id!("type_dependency_edge_sequence_forward_default")
            }
            EntityType::DependencyEdgeCyclicForwardDefault => {
                id!("type_dependency_edge_cyclic_forward_default")
            }
            EntityType::DependencyEdgeSymmetricForwardDefault => {
                id!("type_dependency_edge_symmetric_forward_default")
            }
            EntityType::DependencyEdgeSymmetricReverseDefault => {
                id!("type_dependency_edge_symmetric_reverse_default")
            }

            // Interaction edge groups
            EntityType::InteractionEdgeSequenceDefault => {
                id!("type_interaction_edge_sequence_default")
            }
            EntityType::InteractionEdgeCyclicDefault => {
                id!("type_interaction_edge_cyclic_default")
            }
            EntityType::InteractionEdgeSymmetricDefault => {
                id!("type_interaction_edge_symmetric_default")
            }

            // Interaction edges
            EntityType::InteractionEdgeSequenceForwardDefault => {
                id!("type_interaction_edge_sequence_forward_default")
            }
            EntityType::InteractionEdgeCyclicForwardDefault => {
                id!("type_interaction_edge_cyclic_forward_default")
            }
            EntityType::InteractionEdgeSymmetricForwardDefault => {
                id!("type_interaction_edge_symmetric_forward_default")
            }
            EntityType::InteractionEdgeSymmetricReverseDefault => {
                id!("type_interaction_edge_symmetric_reverse_default")
            }

            EntityType::Custom(id) => id,
        }
    }

    /// Returns the underlying `Id` if this is a custom style alias.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_model_common::entity::EntityType;
    ///
    /// assert_eq!(EntityType::ThingDefault.custom_id(), None);
    ///
    /// let custom = EntityType::Custom("type_server".parse().unwrap());
    /// assert!(custom.custom_id().is_some());
    /// ```
    pub fn custom_id(&self) -> Option<&Id> {
        if let Self::Custom(id) = self {
            Some(id)
        } else {
            None
        }
    }
}

impl From<Id> for EntityType {
    fn from(id: Id) -> Self {
        match id.as_str() {
            "container_inbuilt" => EntityType::ContainerInbuilt,
            "type_thing_default" => EntityType::ThingDefault,
            "type_tag_default" => EntityType::TagDefault,
            "type_process_default" => EntityType::ProcessDefault,
            "type_process_step_default" => EntityType::ProcessStepDefault,

            // Dependency edge groups
            "type_dependency_edge_sequence_default" => EntityType::DependencyEdgeSequenceDefault,
            "type_dependency_edge_cyclic_default" => EntityType::DependencyEdgeCyclicDefault,
            "type_dependency_edge_symmetric_default" => EntityType::DependencyEdgeSymmetricDefault,

            // Dependency edges
            "type_dependency_edge_sequence_forward_default" => {
                EntityType::DependencyEdgeSequenceForwardDefault
            }
            "type_dependency_edge_cyclic_forward_default" => {
                EntityType::DependencyEdgeCyclicForwardDefault
            }
            "type_dependency_edge_symmetric_forward_default" => {
                EntityType::DependencyEdgeSymmetricForwardDefault
            }
            "type_dependency_edge_symmetric_reverse_default" => {
                EntityType::DependencyEdgeSymmetricReverseDefault
            }

            // Interaction edge groups
            "type_interaction_edge_sequence_default" => EntityType::InteractionEdgeSequenceDefault,
            "type_interaction_edge_cyclic_default" => EntityType::InteractionEdgeCyclicDefault,
            "type_interaction_edge_symmetric_default" => {
                EntityType::InteractionEdgeSymmetricDefault
            }

            // Interaction edges
            "type_interaction_edge_sequence_forward_default" => {
                EntityType::InteractionEdgeSequenceForwardDefault
            }
            "type_interaction_edge_cyclic_forward_default" => {
                EntityType::InteractionEdgeCyclicForwardDefault
            }
            "type_interaction_edge_symmetric_forward_default" => {
                EntityType::InteractionEdgeSymmetricForwardDefault
            }
            "type_interaction_edge_symmetric_reverse_default" => {
                EntityType::InteractionEdgeSymmetricReverseDefault
            }

            _ => EntityType::Custom(id),
        }
    }
}

impl Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl Serialize for EntityType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for EntityType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(EntityTypeVisitor)
    }
}

struct EntityTypeVisitor;

impl Visitor<'_> for EntityTypeVisitor {
    type Value = EntityType;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(
            "a type name such as:\n\
            \n\
            * `container_inbuilt`\n\
            * `type_thing_default`\n\
            * `type_tag_default`\n\
            * `type_process_default`\n\
            * `type_process_step_default`\n\
            * `type_dependency_edge_sequence_default`\n\
            * `type_dependency_edge_cyclic_default`\n\
            * `type_dependency_edge_symmetric_default`\n\
            * `type_dependency_edge_sequence_forward_default`\n\
            * `type_dependency_edge_cyclic_forward_default`\n\
            * `type_dependency_edge_symmetric_forward_default`\n\
            * `type_dependency_edge_symmetric_reverse_default`\n\
            * `type_interaction_edge_sequence_default`\n\
            * `type_interaction_edge_cyclic_default`\n\
            * `type_interaction_edge_symmetric_default`\n\
            * `type_interaction_edge_sequence_forward_default`\n\
            * `type_interaction_edge_cyclic_forward_default`\n\
            * `type_interaction_edge_symmetric_forward_default`\n\
            * `type_interaction_edge_symmetric_reverse_default`\n\
            \n\
            or a custom identifier",
        )
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let entity_type = match value {
            "container_inbuilt" => EntityType::ContainerInbuilt,
            "type_thing_default" => EntityType::ThingDefault,
            "type_tag_default" => EntityType::TagDefault,
            "type_process_default" => EntityType::ProcessDefault,
            "type_process_step_default" => EntityType::ProcessStepDefault,

            // Dependency edge groups
            "type_dependency_edge_sequence_default" => EntityType::DependencyEdgeSequenceDefault,
            "type_dependency_edge_cyclic_default" => EntityType::DependencyEdgeCyclicDefault,
            "type_dependency_edge_symmetric_default" => EntityType::DependencyEdgeSymmetricDefault,

            // Dependency edges
            "type_dependency_edge_sequence_forward_default" => {
                EntityType::DependencyEdgeSequenceForwardDefault
            }
            "type_dependency_edge_cyclic_forward_default" => {
                EntityType::DependencyEdgeCyclicForwardDefault
            }
            "type_dependency_edge_symmetric_forward_default" => {
                EntityType::DependencyEdgeSymmetricForwardDefault
            }
            "type_dependency_edge_symmetric_reverse_default" => {
                EntityType::DependencyEdgeSymmetricReverseDefault
            }

            // Interaction edge groups
            "type_interaction_edge_sequence_default" => EntityType::InteractionEdgeSequenceDefault,
            "type_interaction_edge_cyclic_default" => EntityType::InteractionEdgeCyclicDefault,
            "type_interaction_edge_symmetric_default" => {
                EntityType::InteractionEdgeSymmetricDefault
            }

            // Interaction edges
            "type_interaction_edge_sequence_forward_default" => {
                EntityType::InteractionEdgeSequenceForwardDefault
            }
            "type_interaction_edge_cyclic_forward_default" => {
                EntityType::InteractionEdgeCyclicForwardDefault
            }
            "type_interaction_edge_symmetric_forward_default" => {
                EntityType::InteractionEdgeSymmetricForwardDefault
            }
            "type_interaction_edge_symmetric_reverse_default" => {
                EntityType::InteractionEdgeSymmetricReverseDefault
            }

            _ => {
                let id = Id::try_from(value.to_owned()).map_err(serde::de::Error::custom)?;
                EntityType::Custom(id)
            }
        };
        Ok(entity_type)
    }
}
