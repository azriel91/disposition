use std::fmt;

use disposition_model_common::{id, Id};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

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
/// use disposition_ir_model::entity::EntityType;
///
/// let entity_type = EntityType::ThingDefault;
/// assert_eq!(entity_type.as_str(), "type_thing_default");
///
/// let custom_alias = EntityType::Custom("type_server".parse().unwrap());
/// assert_eq!(custom_alias.as_str(), "type_server");
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum EntityType {
    /// Default type for things
    ThingDefault,
    /// Default type for tags
    TagDefault,
    /// Default type for processes
    ProcessDefault,
    /// Default type for process_steps
    ProcessStepDefault,
    /// Default type for edge dependency sequence requests
    EdgeDependencySequenceRequestDefault,
    /// Default type for edge dependency sequence responses
    EdgeDependencySequenceResponseDefault,
    /// Default type for edge dependency cyclics
    EdgeDependencyCyclicDefault,
    /// Default type for edge interaction sequence requests
    EdgeInteractionSequenceRequestDefault,
    /// Default type for edge interaction sequence responses
    EdgeInteractionSequenceResponseDefault,
    /// Default type for edge interaction cyclics
    EdgeInteractionCyclicDefault,
    /// Custom user-defined type.
    Custom(Id),
}

impl EntityType {
    /// Returns the string representation of the style alias.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_ir_model::entity::EntityType;
    ///
    /// assert_eq!(EntityType::ThingDefault.as_str(), "type_thing_default");
    /// assert_eq!(EntityType::TagDefault.as_str(), "type_tag_default");
    /// ```
    pub fn as_str(&self) -> &str {
        match self {
            EntityType::ThingDefault => "type_thing_default",
            EntityType::TagDefault => "type_tag_default",
            EntityType::ProcessDefault => "type_process_default",
            EntityType::ProcessStepDefault => "type_process_step_default",
            EntityType::EdgeDependencySequenceRequestDefault => {
                "type_edge_dependency_sequence_request_default"
            }
            EntityType::EdgeDependencySequenceResponseDefault => {
                "type_edge_dependency_sequence_response_default"
            }
            EntityType::EdgeDependencyCyclicDefault => "type_edge_dependency_cyclic_default",
            EntityType::EdgeInteractionSequenceRequestDefault => {
                "type_edge_interaction_sequence_request_default"
            }
            EntityType::EdgeInteractionSequenceResponseDefault => {
                "type_edge_interaction_sequence_response_default"
            }
            EntityType::EdgeInteractionCyclicDefault => "type_edge_interaction_cyclic_default",
            EntityType::Custom(id) => id.as_str(),
        }
    }

    /// Returns the ID representation of the style alias.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_ir_model::entity::EntityType;
    ///
    /// assert_eq!(EntityType::ThingDefault.into_id(), "type_thing_default");
    /// assert_eq!(EntityType::TagDefault.into_id(), "type_tag_default");
    /// ```
    pub fn into_id(self) -> Id {
        match self {
            EntityType::ThingDefault => id!("type_thing_default"),
            EntityType::TagDefault => id!("type_tag_default"),
            EntityType::ProcessDefault => id!("type_process_default"),
            EntityType::ProcessStepDefault => id!("type_process_step_default"),
            EntityType::EdgeDependencySequenceRequestDefault => {
                id!("type_edge_dependency_sequence_request_default")
            }
            EntityType::EdgeDependencySequenceResponseDefault => {
                id!("type_edge_dependency_sequence_response_default")
            }
            EntityType::EdgeDependencyCyclicDefault => id!("type_edge_dependency_cyclic_default"),
            EntityType::EdgeInteractionSequenceRequestDefault => {
                id!("type_edge_interaction_sequence_request_default")
            }
            EntityType::EdgeInteractionSequenceResponseDefault => {
                id!("type_edge_interaction_sequence_response_default")
            }
            EntityType::EdgeInteractionCyclicDefault => id!("type_edge_interaction_cyclic_default"),
            EntityType::Custom(id) => id,
        }
    }

    /// Returns the underlying `Id` if this is a custom style alias.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_ir_model::entity::EntityType;
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
            "type_thing_default" => EntityType::ThingDefault,
            "type_tag_default" => EntityType::TagDefault,
            "type_process_default" => EntityType::ProcessDefault,
            "type_process_step_default" => EntityType::ProcessStepDefault,
            "type_edge_dependency_sequence_request_default" => {
                EntityType::EdgeDependencySequenceRequestDefault
            }
            "type_edge_dependency_sequence_response_default" => {
                EntityType::EdgeDependencySequenceResponseDefault
            }
            "type_edge_dependency_cyclic_default" => EntityType::EdgeDependencyCyclicDefault,
            "type_edge_interaction_sequence_request_default" => {
                EntityType::EdgeInteractionSequenceRequestDefault
            }
            "type_edge_interaction_sequence_response_default" => {
                EntityType::EdgeInteractionSequenceResponseDefault
            }
            "type_edge_interaction_cyclic_default" => EntityType::EdgeInteractionCyclicDefault,
            _ => EntityType::Custom(id),
        }
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
            "a type name such as `type_thing_default`, `type_tag_default`, `type_process_default`, \
             `type_process_step_default`, `type_edge_dependency_sequence_request_default`, \
             `type_edge_dependency_sequence_response_default`, or a custom identifier",
        )
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let entity_type = match value {
            "type_thing_default" => EntityType::ThingDefault,
            "type_tag_default" => EntityType::TagDefault,
            "type_process_default" => EntityType::ProcessDefault,
            "type_process_step_default" => EntityType::ProcessStepDefault,
            "type_edge_dependency_sequence_request_default" => {
                EntityType::EdgeDependencySequenceRequestDefault
            }
            "type_edge_dependency_sequence_response_default" => {
                EntityType::EdgeDependencySequenceResponseDefault
            }
            "type_edge_dependency_cyclic_default" => EntityType::EdgeDependencyCyclicDefault,
            "type_edge_interaction_sequence_request_default" => {
                EntityType::EdgeInteractionSequenceRequestDefault
            }
            "type_edge_interaction_sequence_response_default" => {
                EntityType::EdgeInteractionSequenceResponseDefault
            }
            "type_edge_interaction_cyclic_default" => EntityType::EdgeInteractionCyclicDefault,
            _ => {
                let id = Id::try_from(value.to_owned()).map_err(serde::de::Error::custom)?;
                EntityType::Custom(id)
            }
        };
        Ok(entity_type)
    }
}
