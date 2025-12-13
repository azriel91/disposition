use serde::{Deserialize, Serialize};

use crate::theme::{StyleAliases, ThemeStyles};

/// Default theme styles when the diagram has no user interaction.
///
/// Contains `style_aliases` that are available to all the other `theme_*` data,
/// and `base_styles` for specific entities.
///
/// # Example
///
/// ```yaml
/// theme_default:
///   style_aliases:
///     padding_none:
///       padding: "0.0"
///       gap: "0.0"
///     padding_tight:
///       padding: "2.0"
///       gap: "2.0"
///     padding_normal:
///       padding: "4.0"
///       gap: "4.0"
///     padding_wide:
///       padding: "6.0"
///       gap: "6.0"
///     shade_light:
///       fill_shade_hover: "200"
///       fill_shade_normal: "300"
///       fill_shade_focus: "400"
///       fill_shade_active: "500"
///       stroke_shade_hover: "300"
///       stroke_shade_normal: "400"
///       stroke_shade_focus: "500"
///       stroke_shade_active: "600"
///       text_shade: "900"
///   base_styles:
///     node_defaults:
///       style_aliases_applied: [shade_light]
///       shape_color: "slate"
///       stroke_style: "solid"
///       stroke_width: "1"
///       text_color: "neutral"
///       visibility: "visible"
///     edge_defaults:
///       stroke_width: "1"
///       text_color: "neutral"
///       visibility: "visible"
///     t_aws:
///       shape_color: "yellow"
///     t_github:
///       shape_color: "neutral"
///   process_step_selected_styles:
///     node_defaults:
///       style_aliases_applied: [shade_pale, stroke_dashed_animated]
///     edge_defaults:
///       visibility: "visible"
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ThemeDefault {
    /// Style aliases available to all theme data.
    ///
    /// These group common style properties under a single name that can be
    /// referenced using `style_aliases_applied`.
    #[serde(default, skip_serializing_if = "StyleAliases::is_empty")]
    pub style_aliases: StyleAliases,

    /// Base styles for entities when there is no user interaction.
    #[serde(default, skip_serializing_if = "ThemeStyles::is_empty")]
    pub base_styles: ThemeStyles,

    /// Styles applied to entities when a process step is selected.
    #[serde(default, skip_serializing_if = "ThemeStyles::is_empty")]
    pub process_step_selected_styles: ThemeStyles,
}

impl ThemeDefault {
    /// Returns a new `ThemeDefault` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if all fields are at their default values.
    pub fn is_empty(&self) -> bool {
        self.style_aliases.is_empty()
            && self.base_styles.is_empty()
            && self.process_step_selected_styles.is_empty()
    }
}
