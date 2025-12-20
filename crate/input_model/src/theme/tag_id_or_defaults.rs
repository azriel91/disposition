use std::fmt::{self, Display};

use disposition_model_common::Id;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

use crate::tag::TagId;

/// Key to specify styles for tag focus, either defaults or a specific tag.
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TagIdOrDefaults {
    /// Styles to apply to all tags by default.
    ///
    /// These properties control the visual appearance of things when any tag
    /// is focused, unless overridden by a specific tag's styles.
    ///
    /// # Example
    ///
    /// ```yaml
    /// theme_tag_things_focus:
    ///   tag_defaults:
    ///     node_defaults:
    ///       style_aliases_applied: [shade_pale, stroke_dashed_animated]
    ///     node_excluded_defaults:
    ///       opacity: "0.5"
    /// ```
    TagDefaults,
    /// Styles specific to a particular tag.
    ///
    /// When this tag is focused, these styles override `TagDefaults`.
    ///
    /// # Example
    ///
    /// ```yaml
    /// theme_tag_things_focus:
    ///   tag_app_development:
    ///     node_defaults:
    ///       style_aliases_applied: [stroke_dashed_animated]
    ///     node_excluded_defaults:
    ///       opacity: "0.3"
    /// ```
    Custom(TagId),
}

impl TagIdOrDefaults {
    /// Returns the string representation of the `TagIdOrDefaults`.
    pub fn as_str(&self) -> &str {
        match self {
            TagIdOrDefaults::TagDefaults => "tag_defaults",
            TagIdOrDefaults::Custom(tag_id) => tag_id.as_str(),
        }
    }

    /// Returns the underlying `TagId` if this holds a custom tag ID.
    pub fn tag_id(&self) -> Option<&TagId> {
        if let Self::Custom(tag_id) = self {
            Some(tag_id)
        } else {
            None
        }
    }
}

impl From<TagId> for TagIdOrDefaults {
    fn from(tag_id: TagId) -> Self {
        Self::Custom(tag_id)
    }
}

impl Display for TagIdOrDefaults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl Serialize for TagIdOrDefaults {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for TagIdOrDefaults {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(TagIdOrDefaultsVisitor)
    }
}

struct TagIdOrDefaultsVisitor;

impl Visitor<'_> for TagIdOrDefaultsVisitor {
    type Value = TagIdOrDefaults;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("`tag_defaults` or a tag ID")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let tag_id_or_defaults = match value {
            "tag_defaults" => TagIdOrDefaults::TagDefaults,
            _ => {
                let id = Id::try_from(value.to_owned()).map_err(serde::de::Error::custom)?;
                let tag_id = TagId::from(id);
                TagIdOrDefaults::Custom(tag_id)
            }
        };
        Ok(tag_id_or_defaults)
    }
}
