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
/// let style_alias = StyleAlias::<'static>::PaddingNormal;
/// assert_eq!(style_alias.as_str(), "padding_normal");
///
/// let custom_alias = StyleAlias::Custom("my_custom_style".parse().unwrap());
/// assert_eq!(custom_alias.as_str(), "my_custom_style");
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum StyleAlias<'id> {
    /// Extra small circle (radius 2 units).
    CircleXs,
    /// Small circle (radius 4 units).
    CircleSm,
    /// Medium circle (radius 6 units).
    CircleMd,
    /// Large circle (radius 8 units).
    CircleLg,
    /// Extra large circle (radius 12 units).
    CircleXl,
    /// No padding.
    PaddingNone,
    /// Tight padding (2 units).
    PaddingTight,
    /// Normal padding (4 units).
    PaddingNormal,
    /// Wide padding (6 units).
    PaddingWide,
    /// Extra small rounded corners (2 units).
    RoundedXs,
    /// Small rounded corners (4 units).
    RoundedSm,
    /// Medium rounded corners (6 units).
    RoundedMd,
    /// Large rounded corners (8 units).
    RoundedLg,
    /// Extra large rounded corners (12 units).
    RoundedXl,
    /// 2x extra large rounded corners (16 units).
    Rounded2xl,
    /// 3x extra large rounded corners (24 units).
    Rounded3xl,
    /// 4x extra large rounded corners (32 units).
    Rounded4xl,
    /// Pale fill shade (lightest fill, no stroke shades).
    FillPale,
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
    Custom(Id<'id>),
}

impl<'id> StyleAlias<'id> {
    /// Returns the string representation of the style alias.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_input_model::theme::StyleAlias;
    ///
    /// assert_eq!(
    ///     StyleAlias::<'static>::PaddingNormal.as_str(),
    ///     "padding_normal"
    /// );
    /// assert_eq!(StyleAlias::<'static>::ShadeLight.as_str(), "shade_light");
    /// ```
    pub fn as_str(&self) -> &str {
        match self {
            StyleAlias::CircleXs => "circle_xs",
            StyleAlias::CircleSm => "circle_sm",
            StyleAlias::CircleMd => "circle_md",
            StyleAlias::CircleLg => "circle_lg",
            StyleAlias::CircleXl => "circle_xl",
            StyleAlias::PaddingNone => "padding_none",
            StyleAlias::PaddingTight => "padding_tight",
            StyleAlias::PaddingNormal => "padding_normal",
            StyleAlias::PaddingWide => "padding_wide",
            StyleAlias::RoundedXs => "rounded_xs",
            StyleAlias::RoundedSm => "rounded_sm",
            StyleAlias::RoundedMd => "rounded_md",
            StyleAlias::RoundedLg => "rounded_lg",
            StyleAlias::RoundedXl => "rounded_xl",
            StyleAlias::Rounded2xl => "rounded_2xl",
            StyleAlias::Rounded3xl => "rounded_3xl",
            StyleAlias::Rounded4xl => "rounded_4xl",
            StyleAlias::FillPale => "fill_pale",
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
    /// assert_eq!(StyleAlias::<'static>::PaddingNormal.custom_id(), None);
    ///
    /// let custom = StyleAlias::Custom("my_style".parse().unwrap());
    /// assert!(custom.custom_id().is_some());
    /// ```
    pub fn custom_id(&self) -> Option<&Id<'id>> {
        if let Self::Custom(id) = self {
            Some(id)
        } else {
            None
        }
    }

    /// Converts this `StyleAlias` into one with a `'static` lifetime.
    ///
    /// This clones the inner `Id` if it's a `Custom` variant.
    pub fn into_static(self) -> StyleAlias<'static> {
        match self {
            StyleAlias::CircleXs => StyleAlias::CircleXs,
            StyleAlias::CircleSm => StyleAlias::CircleSm,
            StyleAlias::CircleMd => StyleAlias::CircleMd,
            StyleAlias::CircleLg => StyleAlias::CircleLg,
            StyleAlias::CircleXl => StyleAlias::CircleXl,
            StyleAlias::PaddingNone => StyleAlias::PaddingNone,
            StyleAlias::PaddingTight => StyleAlias::PaddingTight,
            StyleAlias::PaddingNormal => StyleAlias::PaddingNormal,
            StyleAlias::PaddingWide => StyleAlias::PaddingWide,
            StyleAlias::RoundedXs => StyleAlias::RoundedXs,
            StyleAlias::RoundedSm => StyleAlias::RoundedSm,
            StyleAlias::RoundedMd => StyleAlias::RoundedMd,
            StyleAlias::RoundedLg => StyleAlias::RoundedLg,
            StyleAlias::RoundedXl => StyleAlias::RoundedXl,
            StyleAlias::Rounded2xl => StyleAlias::Rounded2xl,
            StyleAlias::Rounded3xl => StyleAlias::Rounded3xl,
            StyleAlias::Rounded4xl => StyleAlias::Rounded4xl,
            StyleAlias::FillPale => StyleAlias::FillPale,
            StyleAlias::ShadePale => StyleAlias::ShadePale,
            StyleAlias::ShadeLight => StyleAlias::ShadeLight,
            StyleAlias::ShadeMedium => StyleAlias::ShadeMedium,
            StyleAlias::ShadeDark => StyleAlias::ShadeDark,
            StyleAlias::StrokeDashedAnimated => StyleAlias::StrokeDashedAnimated,
            StyleAlias::StrokeDashedAnimatedRequest => StyleAlias::StrokeDashedAnimatedRequest,
            StyleAlias::StrokeDashedAnimatedResponse => StyleAlias::StrokeDashedAnimatedResponse,
            StyleAlias::Custom(id) => StyleAlias::Custom(id.into_static()),
        }
    }
}

impl<'id> From<Id<'id>> for StyleAlias<'id> {
    fn from(id: Id<'id>) -> Self {
        match id.as_str() {
            "circle_xs" => StyleAlias::CircleXs,
            "circle_sm" => StyleAlias::CircleSm,
            "circle_md" => StyleAlias::CircleMd,
            "circle_lg" => StyleAlias::CircleLg,
            "circle_xl" => StyleAlias::CircleXl,
            "padding_none" => StyleAlias::PaddingNone,
            "padding_tight" => StyleAlias::PaddingTight,
            "padding_normal" => StyleAlias::PaddingNormal,
            "padding_wide" => StyleAlias::PaddingWide,
            "rounded_xs" => StyleAlias::RoundedXs,
            "rounded_sm" => StyleAlias::RoundedSm,
            "rounded_md" => StyleAlias::RoundedMd,
            "rounded_lg" => StyleAlias::RoundedLg,
            "rounded_xl" => StyleAlias::RoundedXl,
            "rounded_2xl" => StyleAlias::Rounded2xl,
            "rounded_3xl" => StyleAlias::Rounded3xl,
            "rounded_4xl" => StyleAlias::Rounded4xl,
            "fill_pale" => StyleAlias::FillPale,
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

impl Display for StyleAlias<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl Serialize for StyleAlias<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for StyleAlias<'static> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StyleAliasVisitor)
    }
}

struct StyleAliasVisitor;

impl Visitor<'_> for StyleAliasVisitor {
    type Value = StyleAlias<'static>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(
            "a style alias name such as `circle_xs`, `circle_sm`, `circle_md`, `circle_lg`, \
             `circle_xl`, `padding_none`, `padding_tight`, `padding_normal`, \
             `padding_wide`, `rounded_xs`, `rounded_sm`, `rounded_md`, `rounded_lg`, \
             `rounded_xl`, `rounded_2xl`, `rounded_3xl`, `rounded_4xl`, `fill_pale`, \
             `shade_pale`, `shade_light`, `shade_medium`, `shade_dark`, \
             `stroke_dashed_animated`, `stroke_dashed_animated_request`, \
             `stroke_dashed_animated_response`, or a custom identifier",
        )
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let style_alias = match value {
            "circle_xs" => StyleAlias::CircleXs,
            "circle_sm" => StyleAlias::CircleSm,
            "circle_md" => StyleAlias::CircleMd,
            "circle_lg" => StyleAlias::CircleLg,
            "circle_xl" => StyleAlias::CircleXl,
            "padding_none" => StyleAlias::PaddingNone,
            "padding_tight" => StyleAlias::PaddingTight,
            "padding_normal" => StyleAlias::PaddingNormal,
            "padding_wide" => StyleAlias::PaddingWide,
            "rounded_xs" => StyleAlias::RoundedXs,
            "rounded_sm" => StyleAlias::RoundedSm,
            "rounded_md" => StyleAlias::RoundedMd,
            "rounded_lg" => StyleAlias::RoundedLg,
            "rounded_xl" => StyleAlias::RoundedXl,
            "rounded_2xl" => StyleAlias::Rounded2xl,
            "rounded_3xl" => StyleAlias::Rounded3xl,
            "rounded_4xl" => StyleAlias::Rounded4xl,
            "fill_pale" => StyleAlias::FillPale,
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
