use std::{borrow::Cow, fmt::Write};

use disposition_input_model::theme::ThemeAttr;
use disposition_model_common::Map;

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

    /// Write tailwind classes to the given string.
    pub(crate) fn write_classes(&self, classes: &mut String) {
        self.write_peer_classes(classes, "");
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
    pub(crate) fn write_peer_classes(&self, classes: &mut String, prefix: &str) {
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

        // Fill classes with peer prefix
        if let Some((fill_color_hover, fill_shade_hover)) = fill_color_hover.zip(fill_shade_hover) {
            writeln!(
                classes,
                "{prefix}hover:fill-{fill_color_hover}-{fill_shade_hover}"
            )
            .expect(CLASSES_BUFFER_WRITE_FAIL);
        }
        if let Some((fill_color_normal, fill_shade_normal)) =
            fill_color_normal.zip(fill_shade_normal)
        {
            writeln!(
                classes,
                "{prefix}fill-{fill_color_normal}-{fill_shade_normal}"
            )
            .expect(CLASSES_BUFFER_WRITE_FAIL);
        }
        if let Some((fill_color_focus, fill_shade_focus)) = fill_color_focus.zip(fill_shade_focus) {
            writeln!(
                classes,
                "{prefix}focus:fill-{fill_color_focus}-{fill_shade_focus}"
            )
            .expect(CLASSES_BUFFER_WRITE_FAIL);
        }
        if let Some((fill_color_active, fill_shade_active)) =
            fill_color_active.zip(fill_shade_active)
        {
            writeln!(
                classes,
                "{prefix}active:fill-{fill_color_active}-{fill_shade_active}"
            )
            .expect(CLASSES_BUFFER_WRITE_FAIL);
        }

        // Stroke classes with peer prefix
        if let Some((stroke_color_hover, stroke_shade_hover)) =
            stroke_color_hover.zip(stroke_shade_hover)
        {
            writeln!(
                classes,
                "{prefix}hover:stroke-{stroke_color_hover}-{stroke_shade_hover}"
            )
            .expect(CLASSES_BUFFER_WRITE_FAIL);
        }
        if let Some((stroke_color_normal, stroke_shade_normal)) =
            stroke_color_normal.zip(stroke_shade_normal)
        {
            writeln!(
                classes,
                "{prefix}stroke-{stroke_color_normal}-{stroke_shade_normal}"
            )
            .expect(CLASSES_BUFFER_WRITE_FAIL);
        }
        if let Some((stroke_color_focus, stroke_shade_focus)) =
            stroke_color_focus.zip(stroke_shade_focus)
        {
            writeln!(
                classes,
                "{prefix}focus:stroke-{stroke_color_focus}-{stroke_shade_focus}"
            )
            .expect(CLASSES_BUFFER_WRITE_FAIL);
        }
        if let Some((stroke_color_active, stroke_shade_active)) =
            stroke_color_active.zip(stroke_shade_active)
        {
            writeln!(
                classes,
                "{prefix}active:stroke-{stroke_color_active}-{stroke_shade_active}"
            )
            .expect(CLASSES_BUFFER_WRITE_FAIL);
        }

        // Text classes
        let text_color = self.attrs.get(&ThemeAttr::TextColor).map(|c| c.as_ref());
        let text_shade = self.attrs.get(&ThemeAttr::TextShade).map(|c| c.as_ref());
        if let Some((text_color, text_shade)) = text_color.zip(text_shade) {
            writeln!(classes, "[&>text]:fill-{text_color}-{text_shade}")
                .expect(CLASSES_BUFFER_WRITE_FAIL);
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
