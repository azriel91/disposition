use std::{borrow::Cow, fmt::Write};

use disposition_input_model::theme::{DarkModeShadeConfig, ThemeAttr};
use disposition_model_common::Map;

use super::{css_theme_vars::CssThemeVars, tailwind_color_shade::TailwindColorShade};

const CLASSES_BUFFER_WRITE_FAIL: &str = "Failed to write string to buffer";

/// State for accumulating resolved tailwind class attributes.
///
/// This struct holds a map of [`ThemeAttr`] to their resolved string values,
/// which are then used to generate the appropriate tailwind CSS classes.
#[derive(Default)]
pub(crate) struct TailwindClassState<'tw_state> {
    /// Map of theme attributes to their resolved values.
    pub(crate) attrs: Map<ThemeAttr, Cow<'tw_state, str>>,
}

impl<'tw_state> TailwindClassState<'tw_state> {
    /// Convert stroke style to stroke-dasharray value.
    fn stroke_style_to_dasharray(style: &str) -> Option<&str> {
        match style {
            "solid" => Some("none"),
            "dashed" => Some("3"),
            "dotted" => Some("2"),
            s if s.starts_with("dasharray:") => Some(&s["dasharray:".len()..]),
            _ => None,
        }
    }

    /// Invert a tailwind shade number for dark mode.
    ///
    /// This is used for **text** colours where the dark-mode shade is the
    /// mirror image of the light-mode shade.
    ///
    /// Uses the following mapping:
    ///
    /// * `50` <-> `950`
    /// * `100` <-> `900`
    /// * `200` <-> `800`
    /// * `300` <-> `700`
    /// * `400` <-> `600`
    /// * `500` <-> `500`
    fn shade_inverted(shade: &str) -> &str {
        match shade {
            "50" => "950",
            "100" => "900",
            "200" => "800",
            "300" => "700",
            "400" => "600",
            "500" => "500",
            "600" => "400",
            "700" => "300",
            "800" => "200",
            "900" => "100",
            "950" => "50",
            other => other,
        }
    }

    /// Compute the dark-mode shade for a fill or stroke shade by shifting
    /// rather than inverting.
    ///
    /// The shift preserves the relative ordering of highlight-state shades so
    /// that, for example, `hover < normal < focus < active` in light mode is
    /// still `hover < normal < focus < active` in dark mode.
    ///
    /// # Shift direction
    ///
    /// The direction is determined by the `normal` shade of the group:
    ///
    /// * Normal shade `<= _400` -- the group is on the light end, so the
    ///   dark-mode shift goes **darker** (toward `_950`).
    /// * Normal shade `>= _600` -- the group is on the dark end, so the
    ///   dark-mode shift goes **lighter** (toward `_50`).
    /// * Normal shade `== _500` -- the tie-breaker examines the other shades in
    ///   the group: if they lean darker (majority index > 5), the dark-mode
    ///   shift goes lighter; if they lean lighter (majority index < 5), the
    ///   dark-mode shift goes darker. When exactly tied, the shift goes darker.
    ///
    /// # Parameters
    ///
    /// * `shade`: The shade string to shift, e.g. `"100"`, `"700"`.
    /// * `shade_normal`: The shade string for `HighlightState::Normal`.
    /// * `shade_hover`: The shade string for `HighlightState::Hover`.
    /// * `shade_focus`: The shade string for `HighlightState::Focus`.
    /// * `shade_active`: The shade string for `HighlightState::Active`.
    ///
    /// # Returns
    ///
    /// The shifted shade as a `&'static str`, or the original `shade` if it
    /// cannot be parsed as a known tailwind shade.
    fn shade_shifted<'a>(
        shade: &'a str,
        levels: u8,
        shade_normal: Option<&str>,
        shade_hover: Option<&str>,
        shade_focus: Option<&str>,
        shade_active: Option<&str>,
    ) -> &'a str {
        let Some(shade_parsed) = TailwindColorShade::from_str(shade) else {
            return shade;
        };

        let shift_darker =
            Self::shade_shift_is_darker(shade_normal, shade_hover, shade_focus, shade_active);

        let dark_shade = if shift_darker {
            shade_parsed.darker(levels)
        } else {
            shade_parsed.lighter(levels)
        };

        dark_shade.as_str()
    }

    /// Determine whether the dark-mode shift direction should go darker.
    ///
    /// Returns `true` when the shift should go darker (light shades in light
    /// mode become darker in dark mode), `false` when the shift should go
    /// lighter.
    ///
    /// # Parameters
    ///
    /// * `shade_normal`: The shade string for `HighlightState::Normal`.
    /// * `shade_hover`: The shade string for `HighlightState::Hover`.
    /// * `shade_focus`: The shade string for `HighlightState::Focus`.
    /// * `shade_active`: The shade string for `HighlightState::Active`.
    fn shade_shift_is_darker(
        shade_normal: Option<&str>,
        shade_hover: Option<&str>,
        shade_focus: Option<&str>,
        shade_active: Option<&str>,
    ) -> bool {
        let normal = shade_normal.and_then(TailwindColorShade::from_str);

        match normal {
            Some(n) if n < TailwindColorShade::_500 => true,
            Some(n) if n > TailwindColorShade::_500 => false,
            Some(_) => {
                // Normal is exactly _500 -- look at the other shades.
                // If the majority leans dark (index > 5), shift lighter.
                // If the majority leans light (index < 5), shift darker.
                // Ties go darker.
                let mid = TailwindColorShade::_500.index();
                let mut light_count: u32 = 0;
                let mut dark_count: u32 = 0;

                for shade_str in [shade_hover, shade_focus, shade_active]
                    .into_iter()
                    .flatten()
                {
                    if let Some(s) = TailwindColorShade::from_str(shade_str) {
                        let idx = s.index();
                        if idx < mid {
                            light_count += 1;
                        } else if idx > mid {
                            dark_count += 1;
                        }
                    }
                }

                if dark_count > light_count {
                    // Shades lean dark in light mode, so shift lighter for
                    // dark mode.
                    false
                } else {
                    // Shades lean light (or tied) in light mode, so shift
                    // darker for dark mode.
                    true
                }
            }
            // No normal shade available -- fall back to shifting darker.
            None => true,
        }
    }

    // === Attribute Getters === //

    /// Get the resolved fill color for a state.
    fn get_fill_color(&self, state: HighlightState) -> Option<&str> {
        let (state_specific, base, shape) = match state {
            HighlightState::Normal => (
                ThemeAttr::FillColorNormal,
                ThemeAttr::FillColor,
                ThemeAttr::ShapeColor,
            ),
            HighlightState::Focus => (
                ThemeAttr::FillColorFocus,
                ThemeAttr::FillColor,
                ThemeAttr::ShapeColor,
            ),
            HighlightState::Hover => (
                ThemeAttr::FillColorHover,
                ThemeAttr::FillColor,
                ThemeAttr::ShapeColor,
            ),
            HighlightState::Active => (
                ThemeAttr::FillColorActive,
                ThemeAttr::FillColor,
                ThemeAttr::ShapeColor,
            ),
        };

        self.attrs
            .get(&state_specific)
            .or_else(|| self.attrs.get(&base))
            .or_else(|| self.attrs.get(&shape))
            .map(|c| c.as_ref())
    }

    /// Get the resolved fill shade for a state.
    fn get_fill_shade(&self, state: HighlightState) -> Option<&str> {
        let (state_specific, base) = match state {
            HighlightState::Normal => (ThemeAttr::FillShadeNormal, ThemeAttr::FillShade),
            HighlightState::Focus => (ThemeAttr::FillShadeFocus, ThemeAttr::FillShade),
            HighlightState::Hover => (ThemeAttr::FillShadeHover, ThemeAttr::FillShade),
            HighlightState::Active => (ThemeAttr::FillShadeActive, ThemeAttr::FillShade),
        };

        self.attrs
            .get(&state_specific)
            .or_else(|| self.attrs.get(&base))
            .map(|c| c.as_ref())
    }

    /// Get the resolved stroke color for a state.
    fn get_stroke_color(&self, state: HighlightState) -> Option<&str> {
        let (state_specific, base, shape) = match state {
            HighlightState::Normal => (
                ThemeAttr::StrokeColorNormal,
                ThemeAttr::StrokeColor,
                ThemeAttr::ShapeColor,
            ),
            HighlightState::Focus => (
                ThemeAttr::StrokeColorFocus,
                ThemeAttr::StrokeColor,
                ThemeAttr::ShapeColor,
            ),
            HighlightState::Hover => (
                ThemeAttr::StrokeColorHover,
                ThemeAttr::StrokeColor,
                ThemeAttr::ShapeColor,
            ),
            HighlightState::Active => (
                ThemeAttr::StrokeColorActive,
                ThemeAttr::StrokeColor,
                ThemeAttr::ShapeColor,
            ),
        };

        self.attrs
            .get(&state_specific)
            .or_else(|| self.attrs.get(&base))
            .or_else(|| self.attrs.get(&shape))
            .map(|c| c.as_ref())
    }

    /// Get the resolved stroke shade for a state.
    fn get_stroke_shade(&self, state: HighlightState) -> Option<&str> {
        let (state_specific, base) = match state {
            HighlightState::Normal => (ThemeAttr::StrokeShadeNormal, ThemeAttr::StrokeShade),
            HighlightState::Focus => (ThemeAttr::StrokeShadeFocus, ThemeAttr::StrokeShade),
            HighlightState::Hover => (ThemeAttr::StrokeShadeHover, ThemeAttr::StrokeShade),
            HighlightState::Active => (ThemeAttr::StrokeShadeActive, ThemeAttr::StrokeShade),
        };

        self.attrs
            .get(&state_specific)
            .or_else(|| self.attrs.get(&base))
            .map(|c| c.as_ref())
    }

    // === Class Writers === //

    /// Write tailwind classes to the given string.
    ///
    /// Colour classes use CSS variables registered in `css_theme_vars` so that
    /// light/dark mode is handled through variable overrides rather than
    /// `dark:` prefixed tailwind classes.
    pub(crate) fn write_classes(
        &self,
        classes: &mut String,
        css_theme_vars: &mut CssThemeVars,
        dark_mode_shade_config: DarkModeShadeConfig,
    ) {
        self.write_peer_classes(classes, "", css_theme_vars, dark_mode_shade_config);
    }

    /// Write peer-prefixed classes to the given string for tag/step
    /// highlighting.
    ///
    /// This method determines what classes to write based on the attributes
    /// present in the state:
    ///
    /// - If only [`ThemeAttr::Opacity`] is set (no fill/stroke shade normals or
    ///   animation), writes only the opacity class.
    /// - If [`ThemeAttr::Animate`] or fill/stroke shade normals are set, writes
    ///   the animation class (if present) followed by full fill/stroke peer
    ///   classes.
    ///
    /// Each class that contains a colour shade registers a CSS variable in
    /// `css_theme_vars` with both the light and dark oklch values. The
    /// `dark_mode_shade_config` parameter controls how dark-mode shades are
    /// computed: disabled, inverted, or shifted by a number of levels.
    pub(crate) fn write_peer_classes(
        &self,
        classes: &mut String,
        prefix: &str,
        css_theme_vars: &mut CssThemeVars,
        dark_mode_shade_config: DarkModeShadeConfig,
    ) {
        // Visibility
        if let Some(visibility) = self.attrs.get(&ThemeAttr::Visibility) {
            writeln!(classes, "{prefix}{visibility}").expect(CLASSES_BUFFER_WRITE_FAIL);
        }

        // Stroke dasharray from stroke_style
        if let Some(style) = self.attrs.get(&ThemeAttr::StrokeStyle)
            && let Some(dasharray) = Self::stroke_style_to_dasharray(style)
        {
            writeln!(classes, "{prefix}[stroke-dasharray:{dasharray}]")
                .expect(CLASSES_BUFFER_WRITE_FAIL);
        }

        // Stroke width
        if let Some(width) = self.attrs.get(&ThemeAttr::StrokeWidth) {
            writeln!(classes, "{prefix}stroke-{width}").expect(CLASSES_BUFFER_WRITE_FAIL);
        }

        if let Some(opacity) = self.attrs.get(&ThemeAttr::Opacity) {
            writeln!(classes, "{prefix}opacity-{opacity}").expect(CLASSES_BUFFER_WRITE_FAIL);
        }
        if let Some(animate) = self.attrs.get(&ThemeAttr::Animate) {
            writeln!(classes, "{prefix}animate-{animate}").expect(CLASSES_BUFFER_WRITE_FAIL);
        }

        let fill_color_hover = self.get_fill_color(HighlightState::Hover);
        let fill_shade_hover = self.get_fill_shade(HighlightState::Hover);
        let fill_color_normal = self.get_fill_color(HighlightState::Normal);
        let fill_shade_normal = self.get_fill_shade(HighlightState::Normal);
        let fill_color_focus = self.get_fill_color(HighlightState::Focus);
        let fill_shade_focus = self.get_fill_shade(HighlightState::Focus);
        let fill_color_active = self.get_fill_color(HighlightState::Active);
        let fill_shade_active = self.get_fill_shade(HighlightState::Active);

        let stroke_color_hover = self.get_stroke_color(HighlightState::Hover);
        let stroke_shade_hover = self.get_stroke_shade(HighlightState::Hover);
        let stroke_color_normal = self.get_stroke_color(HighlightState::Normal);
        let stroke_shade_normal = self.get_stroke_shade(HighlightState::Normal);
        let stroke_color_focus = self.get_stroke_color(HighlightState::Focus);
        let stroke_shade_focus = self.get_stroke_shade(HighlightState::Focus);
        let stroke_color_active = self.get_stroke_color(HighlightState::Active);
        let stroke_shade_active = self.get_stroke_shade(HighlightState::Active);

        // === Fill classes === //
        // Fill uses shade shifting for dark mode -- the relative ordering of
        // highlight-state shades is preserved.

        Self::write_shifted_shade_class(
            classes,
            css_theme_vars,
            prefix,
            "hover:",
            "fill",
            dark_mode_shade_config,
            fill_color_hover,
            fill_shade_hover,
            fill_shade_normal,
            fill_shade_hover,
            fill_shade_focus,
            fill_shade_active,
        );
        Self::write_shifted_shade_class(
            classes,
            css_theme_vars,
            prefix,
            "",
            "fill",
            dark_mode_shade_config,
            fill_color_normal,
            fill_shade_normal,
            fill_shade_normal,
            fill_shade_hover,
            fill_shade_focus,
            fill_shade_active,
        );
        Self::write_shifted_shade_class(
            classes,
            css_theme_vars,
            prefix,
            "focus:",
            "fill",
            dark_mode_shade_config,
            fill_color_focus,
            fill_shade_focus,
            fill_shade_normal,
            fill_shade_hover,
            fill_shade_focus,
            fill_shade_active,
        );
        Self::write_shifted_shade_class(
            classes,
            css_theme_vars,
            prefix,
            "active:",
            "fill",
            dark_mode_shade_config,
            fill_color_active,
            fill_shade_active,
            fill_shade_normal,
            fill_shade_hover,
            fill_shade_focus,
            fill_shade_active,
        );

        // === Stroke classes === //
        // Stroke also uses shade shifting for dark mode.

        Self::write_shifted_shade_class(
            classes,
            css_theme_vars,
            prefix,
            "hover:",
            "stroke",
            dark_mode_shade_config,
            stroke_color_hover,
            stroke_shade_hover,
            stroke_shade_normal,
            stroke_shade_hover,
            stroke_shade_focus,
            stroke_shade_active,
        );
        Self::write_shifted_shade_class(
            classes,
            css_theme_vars,
            prefix,
            "",
            "stroke",
            dark_mode_shade_config,
            stroke_color_normal,
            stroke_shade_normal,
            stroke_shade_normal,
            stroke_shade_hover,
            stroke_shade_focus,
            stroke_shade_active,
        );
        Self::write_shifted_shade_class(
            classes,
            css_theme_vars,
            prefix,
            "focus:",
            "stroke",
            dark_mode_shade_config,
            stroke_color_focus,
            stroke_shade_focus,
            stroke_shade_normal,
            stroke_shade_hover,
            stroke_shade_focus,
            stroke_shade_active,
        );
        Self::write_shifted_shade_class(
            classes,
            css_theme_vars,
            prefix,
            "active:",
            "stroke",
            dark_mode_shade_config,
            stroke_color_active,
            stroke_shade_active,
            stroke_shade_normal,
            stroke_shade_hover,
            stroke_shade_focus,
            stroke_shade_active,
        );

        // === Text classes === //
        // Text uses shade inversion for dark mode.
        // The `[&>text]` selector is not prefixed with the peer prefix because
        // text colour does not change based on peer state.

        let text_color = self.attrs.get(&ThemeAttr::TextColor).map(|c| c.as_ref());
        let text_shade = self.attrs.get(&ThemeAttr::TextShade).map(|c| c.as_ref());
        if let Some((text_color, text_shade)) = text_color.zip(text_shade) {
            match dark_mode_shade_config {
                DarkModeShadeConfig::Disable => {
                    writeln!(classes, "[&>text]:fill-{text_color}-{text_shade}")
                        .expect(CLASSES_BUFFER_WRITE_FAIL);
                }
                DarkModeShadeConfig::Invert | DarkModeShadeConfig::Shift { .. } => {
                    let dark_shade = Self::shade_inverted(text_shade);
                    if let Some(var_name) =
                        css_theme_vars.register(text_color, text_shade, dark_shade)
                    {
                        writeln!(classes, "[&>text]:fill-[var({var_name})]")
                            .expect(CLASSES_BUFFER_WRITE_FAIL);
                    } else {
                        writeln!(classes, "[&>text]:fill-{text_color}-{text_shade}")
                            .expect(CLASSES_BUFFER_WRITE_FAIL);
                    }
                }
            }
        }
    }

    /// Write a shade class for fill or stroke, handling all three dark-mode
    /// configurations.
    ///
    /// The behaviour depends on `dark_mode_shade_config`:
    ///
    /// * [`DarkModeShadeConfig::Disable`] -- emit a plain tailwind class with
    ///   no dark-mode support.
    /// * [`DarkModeShadeConfig::Invert`] -- compute the dark-mode shade by
    ///   inverting (mirroring around 500) and register a CSS variable.
    /// * [`DarkModeShadeConfig::Shift`] -- compute the dark-mode shade by
    ///   shifting all highlight-state shades by the configured number of
    ///   levels. This preserves the relative ordering so that, for example,
    ///   `hover < normal < focus < active` in light mode remains `hover <
    ///   normal < focus < active` in dark mode.
    ///
    /// When the colour and shade are known in the tailwind colour table, a CSS
    /// variable is registered in `css_theme_vars` with both the light-mode and
    /// dark-mode oklch values. The emitted class then references the variable,
    /// e.g. `fill-[var(--tw-blue-100-500)]`.
    ///
    /// If the colour is not found in the lookup table the original tailwind
    /// class (e.g. `fill-blue-100`) is emitted without dark mode support.
    ///
    /// # Parameters
    ///
    /// * `classes`: The string buffer to write to.
    /// * `css_theme_vars`: Collector for CSS variable definitions.
    /// * `prefix`: The peer prefix for the class, e.g.
    ///   `"peer-[:focus-within]/tag:"` or `""`.
    /// * `state_modifier`: The highlight state modifier, e.g. `"hover:"`,
    ///   `"focus:"`, `"active:"`, or `""` for normal.
    /// * `property`: `"fill"` or `"stroke"`.
    /// * `color`: The resolved colour name, e.g. `"yellow"`, `"slate"`.
    /// * `shade`: The resolved shade value for this state, e.g. `"100"`.
    /// * `dark_mode_shade_config`: Controls how dark-mode shades are computed.
    /// * `shade_normal`: The shade for `HighlightState::Normal` (used to
    ///   determine shift direction).
    /// * `shade_hover`: The shade for `HighlightState::Hover` (used as
    ///   tie-breaker when normal is `_500`).
    /// * `shade_focus`: The shade for `HighlightState::Focus` (used as
    ///   tie-breaker when normal is `_500`).
    /// * `shade_active`: The shade for `HighlightState::Active` (used as
    ///   tie-breaker when normal is `_500`).
    #[allow(clippy::too_many_arguments)]
    fn write_shifted_shade_class(
        classes: &mut String,
        css_theme_vars: &mut CssThemeVars,
        prefix: &str,
        state_modifier: &str,
        property: &str,
        dark_mode_shade_config: DarkModeShadeConfig,
        color: Option<&str>,
        shade: Option<&str>,
        shade_normal: Option<&str>,
        shade_hover: Option<&str>,
        shade_focus: Option<&str>,
        shade_active: Option<&str>,
    ) {
        if let Some((color, shade)) = color.zip(shade) {
            match dark_mode_shade_config {
                DarkModeShadeConfig::Disable => {
                    // No dark mode -- emit plain tailwind class.
                    writeln!(
                        classes,
                        "{prefix}{state_modifier}{property}-{color}-{shade}"
                    )
                    .expect(CLASSES_BUFFER_WRITE_FAIL);
                }
                DarkModeShadeConfig::Invert => {
                    let dark_shade = Self::shade_inverted(shade);
                    if let Some(var_name) = css_theme_vars.register(color, shade, dark_shade) {
                        writeln!(
                            classes,
                            "{prefix}{state_modifier}{property}-[var({var_name})]"
                        )
                        .expect(CLASSES_BUFFER_WRITE_FAIL);
                    } else {
                        writeln!(
                            classes,
                            "{prefix}{state_modifier}{property}-{color}-{shade}"
                        )
                        .expect(CLASSES_BUFFER_WRITE_FAIL);
                    }
                }
                DarkModeShadeConfig::Shift { levels } => {
                    let dark_shade = Self::shade_shifted(
                        shade,
                        levels,
                        shade_normal,
                        shade_hover,
                        shade_focus,
                        shade_active,
                    );
                    if let Some(var_name) = css_theme_vars.register(color, shade, dark_shade) {
                        writeln!(
                            classes,
                            "{prefix}{state_modifier}{property}-[var({var_name})]"
                        )
                        .expect(CLASSES_BUFFER_WRITE_FAIL);
                    } else {
                        writeln!(
                            classes,
                            "{prefix}{state_modifier}{property}-{color}-{shade}"
                        )
                        .expect(CLASSES_BUFFER_WRITE_FAIL);
                    }
                }
            }
        }
    }
}

/// States for fill and stroke colors.
#[derive(Clone, Copy)]
pub(crate) enum HighlightState {
    Normal,
    Focus,
    Hover,
    Active,
}
