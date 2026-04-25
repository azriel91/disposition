use std::{
    borrow::Cow,
    fmt::{self, Write},
};

use disposition_input_model::theme::{DarkModeShadeConfig, ThemeAttr};
use disposition_model_common::{entity::EntityType, Map};

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
    /// The first entity type of the entity these classes are built for.
    ///
    /// Used to determine whether outline classes should be prefixed with
    /// `[&>.locus]:` (for edge entities) or applied directly (for nodes).
    pub(crate) entity_type: Option<EntityType>,
}

impl<'tw_state> TailwindClassState<'tw_state> {
    /// Convert stroke style to stroke-dasharray value.
    fn stroke_style_to_dasharray(style: &str) -> Option<&str> {
        match style {
            // `stroke-dasharray: none` in CSS produces a solid line.
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

    /// Get the resolved outline color for a state.
    fn get_outline_color(&self, state: HighlightState) -> Option<&str> {
        let (state_specific, base) = match state {
            HighlightState::Normal => (ThemeAttr::OutlineColorNormal, ThemeAttr::OutlineColor),
            HighlightState::Focus => (ThemeAttr::OutlineColorFocus, ThemeAttr::OutlineColor),
            HighlightState::Hover => (ThemeAttr::OutlineColorHover, ThemeAttr::OutlineColor),
            HighlightState::Active => (ThemeAttr::OutlineColorActive, ThemeAttr::OutlineColor),
        };

        self.attrs
            .get(&state_specific)
            .or_else(|| self.attrs.get(&base))
            .map(|c| c.as_ref())
    }

    /// Get the resolved outline shade for a state.
    fn get_outline_shade(&self, state: HighlightState) -> Option<&str> {
        let (state_specific, base) = match state {
            HighlightState::Normal => (ThemeAttr::OutlineShadeNormal, ThemeAttr::OutlineShade),
            HighlightState::Focus => (ThemeAttr::OutlineShadeFocus, ThemeAttr::OutlineShade),
            HighlightState::Hover => (ThemeAttr::OutlineShadeHover, ThemeAttr::OutlineShade),
            HighlightState::Active => (ThemeAttr::OutlineShadeActive, ThemeAttr::OutlineShade),
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

    /// Get the resolved stroke style for a state.
    ///
    /// Looks up the state-specific attribute first (e.g.
    /// [`ThemeAttr::StrokeStyleHover`]) and falls back to the base
    /// [`ThemeAttr::StrokeStyle`] if the state-specific one is absent.
    fn get_stroke_style(&self, state: HighlightState) -> Option<&str> {
        let (state_specific, base) = match state {
            HighlightState::Normal => (ThemeAttr::StrokeStyleNormal, ThemeAttr::StrokeStyle),
            HighlightState::Focus => (ThemeAttr::StrokeStyleFocus, ThemeAttr::StrokeStyle),
            HighlightState::Hover => (ThemeAttr::StrokeStyleHover, ThemeAttr::StrokeStyle),
            HighlightState::Active => (ThemeAttr::StrokeStyleActive, ThemeAttr::StrokeStyle),
        };

        self.attrs
            .get(&state_specific)
            .or_else(|| self.attrs.get(&base))
            .map(|c| c.as_ref())
    }

    /// Get the resolved outline style for a state.
    ///
    /// Looks up the state-specific attribute first (e.g.
    /// [`ThemeAttr::OutlineStyleHover`]) and falls back to the base
    /// [`ThemeAttr::OutlineStyle`] if the state-specific one is absent.
    fn get_outline_style(&self, state: HighlightState) -> Option<&str> {
        let (state_specific, base) = match state {
            HighlightState::Normal => (ThemeAttr::OutlineStyleNormal, ThemeAttr::OutlineStyle),
            HighlightState::Focus => (ThemeAttr::OutlineStyleFocus, ThemeAttr::OutlineStyle),
            HighlightState::Hover => (ThemeAttr::OutlineStyleHover, ThemeAttr::OutlineStyle),
            HighlightState::Active => (ThemeAttr::OutlineStyleActive, ThemeAttr::OutlineStyle),
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
        peer_prefix_maybe: &str,
        css_theme_vars: &mut CssThemeVars,
        dark_mode_shade_config: DarkModeShadeConfig,
    ) {
        // Visibility
        if let Some(visibility) = self.attrs.get(&ThemeAttr::Visibility) {
            writeln!(classes, "{peer_prefix_maybe}{visibility}").expect(CLASSES_BUFFER_WRITE_FAIL);
        }

        let stroke_style_normal = self.get_stroke_style(HighlightState::Normal);
        let stroke_style_hover = self.get_stroke_style(HighlightState::Hover);
        let stroke_style_focus = self.get_stroke_style(HighlightState::Focus);
        let stroke_style_active = self.get_stroke_style(HighlightState::Active);

        // Stroke dasharray from stroke_style (per-state, with base fallback)
        for (state_modifier, stroke_style) in [
            ("", stroke_style_normal),
            ("hover:", stroke_style_hover),
            ("focus:", stroke_style_focus),
            ("active:", stroke_style_active),
        ] {
            if let Some(style) = stroke_style
                && let Some(dasharray) = Self::stroke_style_to_dasharray(style)
            {
                writeln!(
                    classes,
                    "{peer_prefix_maybe}{state_modifier}[stroke-dasharray:{dasharray}]"
                )
                .expect(CLASSES_BUFFER_WRITE_FAIL);
            }
        }

        // Stroke width
        if let Some(width) = self.attrs.get(&ThemeAttr::StrokeWidth) {
            writeln!(classes, "{peer_prefix_maybe}stroke-{width}")
                .expect(CLASSES_BUFFER_WRITE_FAIL);
        }

        if let Some(opacity) = self.attrs.get(&ThemeAttr::Opacity) {
            writeln!(classes, "{peer_prefix_maybe}opacity-{opacity}")
                .expect(CLASSES_BUFFER_WRITE_FAIL);
        }
        if let Some(animate) = self.attrs.get(&ThemeAttr::Animate) {
            writeln!(classes, "{peer_prefix_maybe}animate-{animate}")
                .expect(CLASSES_BUFFER_WRITE_FAIL);
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
            peer_prefix_maybe,
            None,
            "hover:",
            ColorTarget::Fill,
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
            peer_prefix_maybe,
            None,
            "",
            ColorTarget::Fill,
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
            peer_prefix_maybe,
            None,
            "focus:",
            ColorTarget::Fill,
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
            peer_prefix_maybe,
            None,
            "active:",
            ColorTarget::Fill,
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
        //
        // Skip color/shade classes for states where stroke_style is "none": in
        // SVG specifying a stroke color will draw the stroke regardless of
        // style, unlike HTML where `border-style: none` prevents drawing.

        for (state_modifier, color, shade, stroke_style) in [
            (
                "hover:",
                stroke_color_hover,
                stroke_shade_hover,
                stroke_style_hover,
            ),
            (
                "",
                stroke_color_normal,
                stroke_shade_normal,
                stroke_style_normal,
            ),
            (
                "focus:",
                stroke_color_focus,
                stroke_shade_focus,
                stroke_style_focus,
            ),
            (
                "active:",
                stroke_color_active,
                stroke_shade_active,
                stroke_style_active,
            ),
        ] {
            if stroke_style != Some("none") {
                Self::write_shifted_shade_class(
                    classes,
                    css_theme_vars,
                    peer_prefix_maybe,
                    None,
                    state_modifier,
                    ColorTarget::Stroke,
                    dark_mode_shade_config,
                    color,
                    shade,
                    stroke_shade_normal,
                    stroke_shade_hover,
                    stroke_shade_focus,
                    stroke_shade_active,
                );
            }
        }

        // === Outline classes === //
        //
        // For edge entities the outline classes target `.locus` children
        // via the `[&>.locus]:` arbitrary-variant prefix. For all other
        // entities the classes are applied directly.

        let is_edge = self.entity_type.as_ref().is_some_and(EntityType::is_edge);
        let locus_selector_prefix = if is_edge { Some("[&>.locus]:") } else { None };
        let locus_selector_prefix_str = locus_selector_prefix.unwrap_or("");

        let outline_style_normal = self.get_outline_style(HighlightState::Normal);
        let outline_style_hover = self.get_outline_style(HighlightState::Hover);
        let outline_style_focus = self.get_outline_style(HighlightState::Focus);
        let outline_style_active = self.get_outline_style(HighlightState::Active);

        // Outline style (base applies to all states; per-state variants override)
        //
        // For non-edge entities, the standard `outline-{style}` tailwind class is
        // used (e.g. `outline-solid`, `outline-dashed`). For edge entities, the SVG
        // `<path>` outline does not support CSS `outline-style`; instead,
        // `stroke_style_to_dasharray` converts the style to a `stroke-dasharray`
        // value applied to the `.locus` path element.
        let write_outline_style = if is_edge {
            Self::write_outline_style_edge
        } else {
            Self::write_outline_style_node
        };
        for (state_modifier, outline_style) in [
            ("", outline_style_normal),
            ("hover:", outline_style_hover),
            ("focus:", outline_style_focus),
            ("active:", outline_style_active),
        ] {
            if let Some(style) = outline_style {
                write_outline_style(
                    classes,
                    peer_prefix_maybe,
                    locus_selector_prefix,
                    state_modifier,
                    style,
                );
            }
        }

        // Outline width
        if let Some(width) = self.attrs.get(&ThemeAttr::OutlineWidth) {
            if is_edge {
                writeln!(
                    classes,
                    "{peer_prefix_maybe}{locus_selector_prefix_str}stroke-{width}"
                )
                .expect(CLASSES_BUFFER_WRITE_FAIL);
            } else {
                writeln!(
                    classes,
                    "{peer_prefix_maybe}{locus_selector_prefix_str}outline-{width}"
                )
                .expect(CLASSES_BUFFER_WRITE_FAIL);
            }
        }

        // Outline color and shade (similar to stroke, using "outline" as the property)
        //
        // When a shade is also available, `write_shifted_shade_class` is used for
        // dark-mode support. When only a color is specified (no shade), an arbitrary
        // CSS property class `[outline-color:{color}]` is written instead.
        //
        // For edge entities, the `.edge_locus` `<path>` element is styled via SVG
        // `stroke` rather than CSS `outline`, so `"stroke"` is used as the property
        // and `[stroke:{color}]` as the color-only fallback.
        let outline_color_target = if is_edge {
            ColorTarget::Stroke
        } else {
            ColorTarget::Outline
        };
        let outline_color_css_prop = if is_edge { "stroke" } else { "outline-color" };

        let outline_color_hover = self.get_outline_color(HighlightState::Hover);
        let outline_shade_hover = self.get_outline_shade(HighlightState::Hover);
        let outline_color_normal = self.get_outline_color(HighlightState::Normal);
        let outline_shade_normal = self.get_outline_shade(HighlightState::Normal);
        let outline_color_focus = self.get_outline_color(HighlightState::Focus);
        let outline_shade_focus = self.get_outline_shade(HighlightState::Focus);
        let outline_color_active = self.get_outline_color(HighlightState::Active);
        let outline_shade_active = self.get_outline_shade(HighlightState::Active);

        // When outline_style is "none" for a state:
        // - For edge entities: emit `[stroke:none]` on the `.locus` prefix so the SVG
        //   stroke is explicitly cleared for that state. Without this, a stroke color
        //   inherited or set by another state would still be visible, because in SVG
        //   there is no equivalent of `border-style: none` to suppress drawing.
        // - For non-edge entities: skip entirely, since CSS `outline-style: none`
        //   already prevents the outline from being drawn.
        for (state_modifier, color, shade, outline_style) in [
            (
                "hover:",
                outline_color_hover,
                outline_shade_hover,
                outline_style_hover,
            ),
            (
                "",
                outline_color_normal,
                outline_shade_normal,
                outline_style_normal,
            ),
            (
                "focus:",
                outline_color_focus,
                outline_shade_focus,
                outline_style_focus,
            ),
            (
                "active:",
                outline_color_active,
                outline_shade_active,
                outline_style_active,
            ),
        ] {
            if outline_style == Some("none") {
                if is_edge {
                    writeln!(
                        classes,
                        "{peer_prefix_maybe}{state_modifier}{locus_selector_prefix_str}[stroke:none]"
                    )
                    .expect(CLASSES_BUFFER_WRITE_FAIL);
                }
                continue;
            }
            if shade.is_some() {
                Self::write_shifted_shade_class(
                    classes,
                    css_theme_vars,
                    peer_prefix_maybe,
                    locus_selector_prefix,
                    state_modifier,
                    outline_color_target,
                    dark_mode_shade_config,
                    color,
                    shade,
                    outline_shade_normal,
                    outline_shade_hover,
                    outline_shade_focus,
                    outline_shade_active,
                );
            } else if let Some(color) = color {
                writeln!(
                    classes,
                    "{peer_prefix_maybe}{state_modifier}{locus_selector_prefix_str}[{outline_color_css_prop}:{color}]"
                )
                .expect(CLASSES_BUFFER_WRITE_FAIL);
            }
        }

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

    /// Writes outline style classes for edge entities, which uses
    /// `stroke-dasharray` to simulate an outline.
    fn write_outline_style_edge(
        classes: &mut String,
        peer_prefix: &str,
        subelement_selector_prefix: Option<&str>,
        state_modifier: &str,
        outline_style: &str,
    ) {
        let subelement_selector_prefix = subelement_selector_prefix.unwrap_or("");
        if let Some(dasharray) = Self::stroke_style_to_dasharray(outline_style) {
            writeln!(
                classes,
                "{peer_prefix}{state_modifier}{subelement_selector_prefix}[stroke-dasharray:{dasharray}]"
            )
            .expect(CLASSES_BUFFER_WRITE_FAIL);
        }
    }

    /// Writes outline style classes for node entities, which uses the `outline`
    /// tailwind classes.
    fn write_outline_style_node(
        classes: &mut String,
        peer_prefix: &str,
        subelement_selector_prefix: Option<&str>,
        state_modifier: &str,
        outline_style: &str,
    ) {
        let subelement_selector_prefix = subelement_selector_prefix.unwrap_or("");
        writeln!(
            classes,
            "{peer_prefix}{state_modifier}{subelement_selector_prefix}outline-{outline_style}"
        )
        .expect(CLASSES_BUFFER_WRITE_FAIL);
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
    /// * `color_target`: The color target, e.g. `"fill"`, `"stroke"`,
    ///   `"outline"`.
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
        peer_prefix: &str,
        subelement_selector_prefix: Option<&str>,
        state_modifier: &str,
        color_target: ColorTarget,
        dark_mode_shade_config: DarkModeShadeConfig,
        color: Option<&str>,
        shade: Option<&str>,
        shade_normal: Option<&str>,
        shade_hover: Option<&str>,
        shade_focus: Option<&str>,
        shade_active: Option<&str>,
    ) {
        let subelement_selector_prefix = subelement_selector_prefix.unwrap_or("");
        if let Some((color, shade)) = color.zip(shade) {
            write!(
                classes,
                "{peer_prefix}{state_modifier}{subelement_selector_prefix}"
            )
            .expect(CLASSES_BUFFER_WRITE_FAIL);
            match dark_mode_shade_config {
                DarkModeShadeConfig::Disable => {
                    // No dark mode -- emit plain tailwind class.
                    color_target
                        .write_color_shade(classes, color, shade)
                        .expect(CLASSES_BUFFER_WRITE_FAIL);
                }
                DarkModeShadeConfig::Invert => {
                    let dark_shade = Self::shade_inverted(shade);
                    if let Some(var_name) = css_theme_vars.register(color, shade, dark_shade) {
                        color_target
                            .write_css_var(classes, &var_name)
                            .expect(CLASSES_BUFFER_WRITE_FAIL);
                    } else {
                        color_target
                            .write_color_shade(classes, color, shade)
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
                        color_target
                            .write_css_var(classes, &var_name)
                            .expect(CLASSES_BUFFER_WRITE_FAIL);
                    } else {
                        color_target
                            .write_color_shade(classes, color, shade)
                            .expect(CLASSES_BUFFER_WRITE_FAIL);
                    }
                }
            }
            writeln!(classes).expect(CLASSES_BUFFER_WRITE_FAIL);
        }
    }
}

/// States for fill, stroke, and outline colors.
#[derive(Clone, Copy)]
pub(crate) enum HighlightState {
    Normal,
    Focus,
    Hover,
    Active,
}

/// Whether it is the fill, stroke, or outline color.
#[derive(Clone, Copy)]
pub(crate) enum ColorTarget {
    Fill,
    Stroke,
    Outline,
}

impl ColorTarget {
    pub(crate) fn write_color_shade(
        self,
        buffer: &mut String,
        color: &str,
        shade: &str,
    ) -> fmt::Result {
        match self {
            ColorTarget::Fill => write!(buffer, "fill-{color}-{shade}"),
            ColorTarget::Stroke => write!(buffer, "stroke-{color}-{shade}"),
            ColorTarget::Outline => write!(buffer, "outline-{color}-{shade}"),
        }
    }

    /// Writes the CSS var for the color target to the buffer.
    ///
    /// This is needed because tailwind v4 docs say to use the same
    /// `outline-[..]` syntax for arbitrary widths and arbitrary colors.
    ///
    /// For the outline widths, it should generate `outline-width: ..px` and for
    /// colors, it should generate `outline-color: var(--color-)`.
    ///
    /// However, in practice when we use that syntax, encre-css only generates
    /// the `outline-width: ..` CSS style for `outline-[..]` and the color var
    /// is not generated.
    pub(crate) fn write_css_var(self, buffer: &mut String, var_name: &str) -> fmt::Result {
        match self {
            ColorTarget::Fill => write!(buffer, "fill-[var({var_name})]"),
            ColorTarget::Stroke => write!(buffer, "stroke-[var({var_name})]"),
            ColorTarget::Outline => write!(buffer, "[outline-color:var({var_name})]"),
        }
    }
}
