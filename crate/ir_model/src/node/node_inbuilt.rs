use std::fmt::{self, Display};

use disposition_model_common::{entity::EntityType, id, Id};
use enum_iterator::Sequence;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

/// Nodes built into `disposition` necessary for computing layout.
///
/// # Examples
///
/// ```rust
/// use disposition_ir_model::node::NodeInbuilt;
///
/// let root_node = NodeInbuilt::Root;
/// assert_eq!(root_node.as_str(), "_root");
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Sequence)]
pub enum NodeInbuilt {
    /// Root container
    ///
    /// This is in `column_reverse` order so tag node state controls `things`
    /// and `processes`:
    ///
    /// ```text
    /// .------------------------------------.
    /// | 1. _things_and_processes_container |
    /// | -----------------------------------|
    /// | 0. _tags_container                 |
    /// '------------------------------------'
    /// ```
    Root,
    /// Processes container (groups all processes horizontally)
    ///
    /// This is in `row` order so processes are laid out left to right:
    ///
    /// ```text
    /// .-----------------------------------------------------------------------------.
    /// | 0. proc_app_dev | 1. proc_app_release | 2. proc_i12e_region_tier_app_deploy |
    /// '-----------------------------------------------------------------------------'
    /// ```
    ProcessesContainer,
    /// Container for `thing`s and `process`es to be laid out next to each
    /// other.
    ///
    /// ```text
    /// .------------------------------------------------.
    /// | 1. _things_container | 0. _processes_container |
    /// '------------------------------------------------'
    /// ```
    ThingsAndProcessesContainer,
    /// Things container.
    ///
    /// This is in `row` order so things are laid out left to right:
    ///
    /// ```text
    /// .--------------------------------------.
    /// | 0. thing_0 | 1. thing_1 | 2. thing_2 |
    /// |--------------------------------------|
    /// | 3. thing_3 | 4. thing_4 | 5. thing_5 |
    /// '--------------------------------------'
    /// ```
    ThingsContainer,
    /// Tags container.
    ///
    /// This is in `row` order so tags are laid out left to right:
    ///
    /// ```text
    /// .--------------------------------------------.
    /// | 0. tag_app_development | 1. tag_deployment |
    /// '--------------------------------------------'
    /// ```
    TagsContainer,
}

impl NodeInbuilt {
    /// Returns the string representation of the built-in node.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_ir_model::node::NodeInbuilt;
    ///
    /// assert_eq!(NodeInbuilt::Root.as_str(), "_root");
    /// assert_eq!(NodeInbuilt::TagsContainer.as_str(), "_tags_container");
    /// ```
    pub const fn as_str(self) -> &'static str {
        match self {
            NodeInbuilt::Root => "_root",
            NodeInbuilt::ProcessesContainer => "_processes_container",
            NodeInbuilt::ThingsAndProcessesContainer => "_things_and_processes_container",
            NodeInbuilt::ThingsContainer => "_things_container",
            NodeInbuilt::TagsContainer => "_tags_container",
        }
    }

    /// Returns the ID representation of the built-in node.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_ir_model::node::NodeInbuilt;
    /// use disposition_model_common::{id, Id};
    ///
    /// assert_eq!(NodeInbuilt::Root.id(), id!("_root"));
    /// assert_eq!(NodeInbuilt::TagsContainer.id(), id!("_tags_container"));
    /// ```
    pub const fn id(self) -> Id {
        match self {
            NodeInbuilt::Root => id!("_root"),
            NodeInbuilt::ProcessesContainer => id!("_processes_container"),
            NodeInbuilt::ThingsAndProcessesContainer => id!("_things_and_processes_container"),
            NodeInbuilt::ThingsContainer => id!("_things_container"),
            NodeInbuilt::TagsContainer => id!("_tags_container"),
        }
    }

    /// Returns the entity type of the built-in node.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_ir_model::node::NodeInbuilt;
    /// use disposition_model_common::entity::EntityType;
    ///
    /// assert_eq!(
    ///     NodeInbuilt::Root.entity_type(),
    ///     EntityType::ContainerInbuilt
    /// );
    /// assert_eq!(
    ///     NodeInbuilt::TagsContainer.entity_type(),
    ///     EntityType::ContainerInbuilt
    /// );
    /// ```
    pub fn entity_type(self) -> EntityType {
        EntityType::ContainerInbuilt
    }
}

impl TryFrom<Id> for NodeInbuilt {
    type Error = Id;

    fn try_from(id: Id) -> Result<Self, Id> {
        match id.as_str() {
            "_root" => Ok(NodeInbuilt::Root),
            "_processes_container" => Ok(NodeInbuilt::ProcessesContainer),
            "_things_and_processes_container" => Ok(NodeInbuilt::ThingsAndProcessesContainer),
            "_things_container" => Ok(NodeInbuilt::ThingsContainer),
            "_tags_container" => Ok(NodeInbuilt::TagsContainer),
            _ => Err(id),
        }
    }
}

impl Display for NodeInbuilt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl Serialize for NodeInbuilt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for NodeInbuilt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(NodeInbuiltVisitor)
    }
}

struct NodeInbuiltVisitor;

impl Visitor<'_> for NodeInbuiltVisitor {
    type Value = NodeInbuilt;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(
            "a built-in node ID such as `_root`, `_processes_container`, `_things_and_processes_container`, or `_tags_container`",
        )
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let node_inbuilt = match value {
            "_root" => NodeInbuilt::Root,
            "_processes_container" => NodeInbuilt::ProcessesContainer,
            "_things_and_processes_container" => NodeInbuilt::ThingsAndProcessesContainer,
            "_things_container" => NodeInbuilt::ThingsContainer,
            "_tags_container" => NodeInbuilt::TagsContainer,
            id => {
                return Err(serde::de::Error::custom(format!(
                    "Could not parse node ID: `{}` as a built-in node ID",
                    id
                )));
            }
        };
        Ok(node_inbuilt)
    }
}
