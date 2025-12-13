use std::fmt::{self, Display};

use disposition_model_common::Id;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

/// Style alias for grouping style properties under a single name.
///
/// Style aliases allow grouping of style properties under a single name,
/// which can then be applied to nodes and edges using `style_aliases_applied`.
///
/// This enum contains well-known style alias keys, with a `Custom` variant
/// for user-defined aliases.
///
/// # Examples
///
/// ```rust
/// use disposition_input_model::theme::StyleAlias;
///
/// let style_alias = StyleAlias::PaddingNormal;
/// assert_eq!(style_alias.as_str(), "padding_normal");
///
/// let custom_alias = StyleAlias::Custom("my_custom_style".parse().unwrap());
/// assert_eq!(custom_alias.as_str(), "my_custom_style");
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum StyleAlias {
    /// No padding.
    PaddingNone,
    /// Tight padding (2 units).
    PaddingTight,
    /// Normal padding (4 units).
    PaddingNormal,
    /// Wide padding (6 units).
    PaddingWide,
    /// Pale shade (lightest).
    ShadePale,
    /// Light shade.
    ShadeLight,
    /// Medium shade.
    ShadeMedium,
    /// Dark shade (darkest).
    ShadeDark,
    /// Dashed stroke with animation.
    StrokeDashedAnimated,
    /// Dashed stroke with animation for request direction.
    StrokeDashedAnimatedRequest,
    /// Dashed stroke with animation for response direction.
    StrokeDashedAnimatedResponse,
    /// Custom user-defined style alias.
    Custom(Id),
}

impl StyleAlias {
    /// Returns the string representation of the style alias.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_input_model::theme::StyleAlias;
    ///
    /// assert_eq!(StyleAlias::PaddingNormal.as_str(), "padding_normal");
    /// assert_eq!(StyleAlias::ShadeLight.as_str(), "shade_light");
    /// ```
    pub fn as_str(&self) -> &str {
        match self {
            StyleAlias::PaddingNone => "padding_none",
            StyleAlias::PaddingTight => "padding_tight",
            StyleAlias::PaddingNormal => "padding_normal",
            StyleAlias::PaddingWide => "padding_wide",
            StyleAlias::ShadePale => "shade_pale",
            StyleAlias::ShadeLight => "shade_light",
            StyleAlias::ShadeMedium => "shade_medium",
            StyleAlias::ShadeDark => "shade_dark",
            StyleAlias::StrokeDashedAnimated => "stroke_dashed_animated",
            StyleAlias::StrokeDashedAnimatedRequest => "stroke_dashed_animated_request",
            StyleAlias::StrokeDashedAnimatedResponse => "stroke_dashed_animated_response",
            StyleAlias::Custom(id) => id.as_str(),
        }
    }

    /// Returns the underlying `Id` if this is a custom style alias.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_input_model::theme::StyleAlias;
    ///
    /// assert_eq!(StyleAlias::PaddingNormal.custom_id(), None);
    ///
    /// let custom = StyleAlias::Custom("my_style".parse().unwrap());
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

impl From<Id> for StyleAlias {
    fn from(id: Id) -> Self {
        match id.as_str() {
            "padding_none" => StyleAlias::PaddingNone,
            "padding_tight" => StyleAlias::PaddingTight,
            "padding_normal" => StyleAlias::PaddingNormal,
            "padding_wide" => StyleAlias::PaddingWide,
            "shade_pale" => StyleAlias::ShadePale,
            "shade_light" => StyleAlias::ShadeLight,
            "shade_medium" => StyleAlias::ShadeMedium,
            "shade_dark" => StyleAlias::ShadeDark,
            "stroke_dashed_animated" => StyleAlias::StrokeDashedAnimated,
            "stroke_dashed_animated_request" => StyleAlias::StrokeDashedAnimatedRequest,
            "stroke_dashed_animated_response" => StyleAlias::StrokeDashedAnimatedResponse,
            _ => StyleAlias::Custom(id),
        }
    }
}

impl Display for StyleAlias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl Serialize for StyleAlias {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for StyleAlias {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StyleAliasVisitor)
    }
}

struct StyleAliasVisitor;

impl Visitor<'_> for StyleAliasVisitor {
    type Value = StyleAlias;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(
            "a style alias name such as `padding_none`, `padding_tight`, `padding_normal`, \
             `padding_wide`, `shade_pale`, `shade_light`, `shade_medium`, `shade_dark`, \
             `stroke_dashed_animated`, `stroke_dashed_animated_request`, \
             `stroke_dashed_animated_response`, or a custom identifier",
        )
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let style_alias = match value {
            "padding_none" => StyleAlias::PaddingNone,
            "padding_tight" => StyleAlias::PaddingTight,
            "padding_normal" => StyleAlias::PaddingNormal,
            "padding_wide" => StyleAlias::PaddingWide,
            "shade_pale" => StyleAlias::ShadePale,
            "shade_light" => StyleAlias::ShadeLight,
            "shade_medium" => StyleAlias::ShadeMedium,
            "shade_dark" => StyleAlias::ShadeDark,
            "stroke_dashed_animated" => StyleAlias::StrokeDashedAnimated,
            "stroke_dashed_animated_request" => StyleAlias::StrokeDashedAnimatedRequest,
            "stroke_dashed_animated_response" => StyleAlias::StrokeDashedAnimatedResponse,
            _ => {
                let id = Id::try_from(value.to_owned()).map_err(serde::de::Error::custom)?;
                StyleAlias::Custom(id)
            }
        };
        Ok(style_alias)
    }
}
