use std::fmt;

use disposition_model_common::Id;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

/// Key to specify tailwind styles for all kinds of nodes and edges.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum IdOrDefaults {
    /// Styles to apply to all nodes.
    ///
    /// These properties control the visual appearance of nodes (things,
    /// processes, or process steps) in the diagram.
    ///
    /// # Example
    ///
    /// ```yaml
    /// node_defaults:
    ///   style_aliases_applied: [shade_light]
    ///   shape_color: "slate"
    ///   stroke_style: "solid"
    ///   stroke_width: "1"
    ///   visibility: "visible"
    ///   opacity: "1.0"
    /// ```
    NodeDefaults,
    /// Styles to apply to all edges.
    ///
    /// These properties control the visual appearance of edges (connections
    /// between things) in the diagram.
    ///
    /// # Example
    ///
    /// ```yaml
    /// edge_defaults:
    ///   style_aliases_applied: [shade_dark]
    ///   shape_color: "neutral"
    ///   stroke_style: "solid"
    ///   stroke_width: "1"
    ///   visibility: "visible"
    /// ```
    EdgeDefaults,
    /// ID of a thing, edge, tag, process, or process_step.
    Id(Id),
}

impl IdOrDefaults {
    /// Returns the underlying `Id` if this holds an ID.
    pub fn any_id(&self) -> Option<&Id> {
        if let Self::Id(any_id) = self {
            Some(any_id)
        } else {
            None
        }
    }
}

impl From<Id> for IdOrDefaults {
    fn from(any_id: Id) -> Self {
        Self::Id(any_id)
    }
}

impl Serialize for IdOrDefaults {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            IdOrDefaults::NodeDefaults => serializer.serialize_str("node_defaults"),
            IdOrDefaults::EdgeDefaults => serializer.serialize_str("edge_defaults"),
            IdOrDefaults::Id(any_id) => serializer.serialize_str(any_id),
        }
    }
}

impl<'de> Deserialize<'de> for IdOrDefaults {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(IdOrDefaultsVisitor)
    }
}

struct IdOrDefaultsVisitor;

impl Visitor<'_> for IdOrDefaultsVisitor {
    type Value = IdOrDefaults;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("one of `node_defaults`, `edge_defaults`, or a node/edge/tag ID")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let any_id_or_defaults = match value {
            "node_defaults" => IdOrDefaults::NodeDefaults,
            "edge_defaults" => IdOrDefaults::EdgeDefaults,
            _ => {
                let any_id = Id::try_from(value.to_owned()).map_err(serde::de::Error::custom)?;
                IdOrDefaults::Id(any_id)
            }
        };
        Ok(any_id_or_defaults)
    }
}
