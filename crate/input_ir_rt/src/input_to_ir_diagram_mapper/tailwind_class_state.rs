use std::{borrow::Cow, fmt::Write};

use disposition_input_model::theme::ThemeAttr;
use disposition_model_common::Map;

use super::css_theme_vars::CssThemeVars;

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
    pub(crate) fn write_classes(&self, classes: &mut String, css_theme_vars: &mut CssThemeVars) {
        self.write_peer_classes(classes, "", css_theme_vars);
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
    /// `css_theme_vars` with both the light and dark (shade-inverted) oklch
    /// values.  The element then references the variable via the
    /// `fill-[var(--tw-...)]` / `stroke-[var(--tw-...)]` syntax so that the
    /// active colour changes automatically when the user's preferred colour
    /// scheme changes.
    pub(crate) fn write_peer_classes(
        &self,
        classes: &mut String,
        prefix: &str,
        css_theme_vars: &mut CssThemeVars,
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

        Self::write_shade_class(
            classes,
            css_theme_vars,
            prefix,
            "hover:",
            "fill",
            fill_color_hover,
            fill_shade_hover,
        );
        Self::write_shade_class(
            classes,
            css_theme_vars,
            prefix,
            "",
            "fill",
            fill_color_normal,
            fill_shade_normal,
        );
        Self::write_shade_class(
            classes,
            css_theme_vars,
            prefix,
            "focus:",
            "fill",
            fill_color_focus,
            fill_shade_focus,
        );
        Self::write_shade_class(
            classes,
            css_theme_vars,
            prefix,
            "active:",
            "fill",
            fill_color_active,
            fill_shade_active,
        );

        // === Stroke classes === //

        Self::write_shade_class(
            classes,
            css_theme_vars,
            prefix,
            "hover:",
            "stroke",
            stroke_color_hover,
            stroke_shade_hover,
        );
        Self::write_shade_class(
            classes,
            css_theme_vars,
            prefix,
            "",
            "stroke",
            stroke_color_normal,
            stroke_shade_normal,
        );
        Self::write_shade_class(
            classes,
            css_theme_vars,
            prefix,
            "focus:",
            "stroke",
            stroke_color_focus,
            stroke_shade_focus,
        );
        Self::write_shade_class(
            classes,
            css_theme_vars,
            prefix,
            "active:",
            "stroke",
            stroke_color_active,
            stroke_shade_active,
        );

        // === Text classes === //
        // Text uses shade inversion for dark mode.
        // The `[&>text]` selector is not prefixed with the peer prefix because
        // text colour does not change based on peer state.

        let text_color = self.attrs.get(&ThemeAttr::TextColor).map(|c| c.as_ref());
        let text_shade = self.attrs.get(&ThemeAttr::TextShade).map(|c| c.as_ref());
        if let Some((text_color, text_shade)) = text_color.zip(text_shade) {
            let dark_shade = Self::shade_inverted(text_shade);
            if let Some(var_name) = css_theme_vars.register(text_color, text_shade, dark_shade) {
                writeln!(classes, "[&>text]:fill-[var({var_name})]")
                    .expect(CLASSES_BUFFER_WRITE_FAIL);
            } else {
                // Fallback: colour not in the tailwind lookup table, emit
                // the original class without dark mode support.
                writeln!(classes, "[&>text]:fill-{text_color}-{text_shade}")
                    .expect(CLASSES_BUFFER_WRITE_FAIL);
            }
        }
    }

    /// Write a shade class that references a CSS variable for dark/light mode.
    ///
    /// When the colour and shade are known in the tailwind colour table, a CSS
    /// variable is registered in `css_theme_vars` with both the light-mode and
    /// dark-mode (shade-inverted) oklch values.  The emitted class then
    /// references the variable, e.g. `fill-[var(--tw-blue-100-900)]`, so that
    /// a single class works for both colour schemes.
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
    /// * `shade`: The resolved shade value, e.g. `"100"`, `"300"`.
    fn write_shade_class(
        classes: &mut String,
        css_theme_vars: &mut CssThemeVars,
        prefix: &str,
        state_modifier: &str,
        property: &str,
        color: Option<&str>,
        shade: Option<&str>,
    ) {
        if let Some((color, shade)) = color.zip(shade) {
            let dark_shade = Self::shade_inverted(shade);
            if let Some(var_name) = css_theme_vars.register(color, shade, dark_shade) {
                writeln!(
                    classes,
                    "{prefix}{state_modifier}{property}-[var({var_name})]"
                )
                .expect(CLASSES_BUFFER_WRITE_FAIL);
            } else {
                // Fallback: colour not in the tailwind lookup table, emit
                // the original class without dark mode support.
                writeln!(
                    classes,
                    "{prefix}{state_modifier}{property}-{color}-{shade}"
                )
                .expect(CLASSES_BUFFER_WRITE_FAIL);
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
