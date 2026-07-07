use std::{
    borrow::Cow,
    fmt::{self, Write},
};

use disposition_input_model::theme::{DarkModeShadeConfig, ThemeAttr};
use disposition_model_common::{entity::EntityType, Map};

use crate::svg_element_classes::{
    EDGE_ARROW_HEAD_SELECTOR, EDGE_BODY_SELECTOR, NODE_CIRCLE_SELECTOR, NODE_WRAPPER_SELECTOR,
};

use super::{css_theme_vars::CssThemeVars, tailwind_color_shade::TailwindColorShade};

use self::shade_computer::ShadeComputer;

mod shade_computer;

const CLASSES_BUFFER_WRITE_FAIL: &str = "Failed to write string to buffer";

/// State for accumulating resolved tailwind class attributes.
///
/// This struct holds a map of [`ThemeAttr`] to their resolved string values,
/// which are then used to generate the appropriate tailwind CSS classes.
#[derive(Clone, Default)]
pub(crate) struct TailwindClassState<'tw_state> {
    /// Map of theme attributes to their resolved values.
    pub(crate) attrs: Map<ThemeAttr, Cow<'tw_state, str>>,
    /// The first entity type of the entity these classes are built for.
    ///
    /// Used to determine the scoping of Stroke/Fill classes (see
    /// [`ScopeTarget`] / [`TailwindClassState::scope_target`]) and whether
    /// outline classes should be prefixed with `[&>.locus]:` (for edge
    /// entities) or applied directly (for nodes).
    pub(crate) entity_type: Option<EntityType>,
}

impl<'tw_state> TailwindClassState<'tw_state> {
    // === Attribute Getters === //

    /// Returns the first present attribute value among `keys`, tried in order.
    ///
    /// Used by the per-state getters to fall back from the state-specific
    /// attribute to the base attribute (and, for colours, the shared shape
    /// colour).
    fn attr_first(&self, keys: &[ThemeAttr]) -> Option<&str> {
        keys.iter()
            .find_map(|key| self.attrs.get(key))
            .map(|value| value.as_ref())
    }

    /// Get the resolved fill color for a state.
    fn get_fill_color(&self, state: HighlightState) -> Option<&str> {
        let state_specific = match state {
            HighlightState::Normal => ThemeAttr::FillColorNormal,
            HighlightState::Focus => ThemeAttr::FillColorFocus,
            HighlightState::Hover => ThemeAttr::FillColorHover,
            HighlightState::Active => ThemeAttr::FillColorActive,
        };
        self.attr_first(&[state_specific, ThemeAttr::FillColor, ThemeAttr::ShapeColor])
    }

    /// Get the resolved fill shade for a state.
    fn get_fill_shade(&self, state: HighlightState) -> Option<&str> {
        let state_specific = match state {
            HighlightState::Normal => ThemeAttr::FillShadeNormal,
            HighlightState::Focus => ThemeAttr::FillShadeFocus,
            HighlightState::Hover => ThemeAttr::FillShadeHover,
            HighlightState::Active => ThemeAttr::FillShadeActive,
        };
        self.attr_first(&[state_specific, ThemeAttr::FillShade])
    }

    /// Get the resolved outline color for a state.
    fn get_outline_color(&self, state: HighlightState) -> Option<&str> {
        let state_specific = match state {
            HighlightState::Normal => ThemeAttr::OutlineColorNormal,
            HighlightState::Focus => ThemeAttr::OutlineColorFocus,
            HighlightState::Hover => ThemeAttr::OutlineColorHover,
            HighlightState::Active => ThemeAttr::OutlineColorActive,
        };
        self.attr_first(&[state_specific, ThemeAttr::OutlineColor])
    }

    /// Get the resolved outline shade for a state.
    fn get_outline_shade(&self, state: HighlightState) -> Option<&str> {
        let state_specific = match state {
            HighlightState::Normal => ThemeAttr::OutlineShadeNormal,
            HighlightState::Focus => ThemeAttr::OutlineShadeFocus,
            HighlightState::Hover => ThemeAttr::OutlineShadeHover,
            HighlightState::Active => ThemeAttr::OutlineShadeActive,
        };
        self.attr_first(&[state_specific, ThemeAttr::OutlineShade])
    }

    /// Get the resolved stroke color for a state.
    fn get_stroke_color(&self, state: HighlightState) -> Option<&str> {
        let state_specific = match state {
            HighlightState::Normal => ThemeAttr::StrokeColorNormal,
            HighlightState::Focus => ThemeAttr::StrokeColorFocus,
            HighlightState::Hover => ThemeAttr::StrokeColorHover,
            HighlightState::Active => ThemeAttr::StrokeColorActive,
        };
        self.attr_first(&[
            state_specific,
            ThemeAttr::StrokeColor,
            ThemeAttr::ShapeColor,
        ])
    }

    /// Get the resolved stroke shade for a state.
    fn get_stroke_shade(&self, state: HighlightState) -> Option<&str> {
        let state_specific = match state {
            HighlightState::Normal => ThemeAttr::StrokeShadeNormal,
            HighlightState::Focus => ThemeAttr::StrokeShadeFocus,
            HighlightState::Hover => ThemeAttr::StrokeShadeHover,
            HighlightState::Active => ThemeAttr::StrokeShadeActive,
        };
        self.attr_first(&[state_specific, ThemeAttr::StrokeShade])
    }

    /// Get the resolved stroke style for a state.
    ///
    /// Looks up the state-specific attribute first (e.g.
    /// [`ThemeAttr::StrokeStyleHover`]) and falls back to the base
    /// [`ThemeAttr::StrokeStyle`] if the state-specific one is absent.
    fn get_stroke_style(&self, state: HighlightState) -> Option<&str> {
        let state_specific = match state {
            HighlightState::Normal => ThemeAttr::StrokeStyleNormal,
            HighlightState::Focus => ThemeAttr::StrokeStyleFocus,
            HighlightState::Hover => ThemeAttr::StrokeStyleHover,
            HighlightState::Active => ThemeAttr::StrokeStyleActive,
        };
        self.attr_first(&[state_specific, ThemeAttr::StrokeStyle])
    }

    /// Determines which sub-element(s) Stroke/Fill-derived classes should be
    /// scoped to, based on `entity_type`. See [`ScopeTarget`].
    fn scope_target(&self) -> ScopeTarget {
        match self.entity_type.as_ref() {
            Some(entity_type) if entity_type.is_edge() => ScopeTarget::Edge,
            Some(entity_type) if entity_type.is_node() => ScopeTarget::Node,
            _ => ScopeTarget::Unscoped,
        }
    }

    /// Get the resolved outline style for a state.
    ///
    /// Looks up the state-specific attribute first (e.g.
    /// [`ThemeAttr::OutlineStyleHover`]) and falls back to the base
    /// [`ThemeAttr::OutlineStyle`] if the state-specific one is absent.
    fn get_outline_style(&self, state: HighlightState) -> Option<&str> {
        let state_specific = match state {
            HighlightState::Normal => ThemeAttr::OutlineStyleNormal,
            HighlightState::Focus => ThemeAttr::OutlineStyleFocus,
            HighlightState::Hover => ThemeAttr::OutlineStyleHover,
            HighlightState::Active => ThemeAttr::OutlineStyleActive,
        };
        self.attr_first(&[state_specific, ThemeAttr::OutlineStyle])
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
        self.write_peer_classes_visibility(classes, peer_prefix_maybe);
        self.write_peer_classes_stroke_dasharray(classes, peer_prefix_maybe);
        self.write_peer_classes_stroke_width(classes, peer_prefix_maybe);
        self.write_peer_classes_opacity_animate(classes, peer_prefix_maybe);
        self.write_peer_classes_fill(
            classes,
            peer_prefix_maybe,
            css_theme_vars,
            dark_mode_shade_config,
        );
        self.write_peer_classes_stroke(
            classes,
            peer_prefix_maybe,
            css_theme_vars,
            dark_mode_shade_config,
        );
        self.write_peer_classes_outline(
            classes,
            peer_prefix_maybe,
            css_theme_vars,
            dark_mode_shade_config,
        );
        self.write_peer_classes_text(classes, css_theme_vars, dark_mode_shade_config);
        self.write_peer_classes_extra(classes);
    }

    /// Writes the visibility class, if set.
    fn write_peer_classes_visibility(&self, classes: &mut String, peer_prefix_maybe: &str) {
        if let Some(visibility) = self.attrs.get(&ThemeAttr::Visibility) {
            writeln!(classes, "{peer_prefix_maybe}{visibility}").expect(CLASSES_BUFFER_WRITE_FAIL);
        }
    }

    /// Writes the per-state `stroke-dasharray` classes derived from the
    /// resolved stroke style (with base fallback), scoped the same way as
    /// stroke colour/width (see [`ScopeTarget`]) so the dasharray does not
    /// leak onto unrelated sibling paths.
    fn write_peer_classes_stroke_dasharray(&self, classes: &mut String, peer_prefix_maybe: &str) {
        let scope_target = self.scope_target();

        [
            HighlightState::Normal,
            HighlightState::Hover,
            HighlightState::Focus,
            HighlightState::Active,
        ]
        .into_iter()
        .for_each(|state| {
            if let Some(style) = self.get_stroke_style(state)
                && let Some(dasharray) = ShadeComputer::stroke_style_to_dasharray(style)
            {
                let state_modifier = state.modifier();
                scope_target.stroke_selector_prefixes().iter().for_each(
                    |&subelement_selector_prefix| {
                        let subelement_selector_prefix = subelement_selector_prefix.unwrap_or("");
                        writeln!(
                            classes,
                            "{peer_prefix_maybe}{state_modifier}{subelement_selector_prefix}[stroke-dasharray:{dasharray}]"
                        )
                        .expect(CLASSES_BUFFER_WRITE_FAIL);
                    },
                );
            }
        });
    }

    /// Writes the stroke-width class, if set, scoped the same way as stroke
    /// colour/dasharray (see [`ScopeTarget`]).
    fn write_peer_classes_stroke_width(&self, classes: &mut String, peer_prefix_maybe: &str) {
        let Some(width) = self.attrs.get(&ThemeAttr::StrokeWidth) else {
            return;
        };
        let scope_target = self.scope_target();
        scope_target
            .stroke_selector_prefixes()
            .iter()
            .for_each(|&subelement_selector_prefix| {
                let subelement_selector_prefix = subelement_selector_prefix.unwrap_or("");
                writeln!(
                    classes,
                    "{peer_prefix_maybe}{subelement_selector_prefix}stroke-{width}"
                )
                .expect(CLASSES_BUFFER_WRITE_FAIL);
            });
    }

    /// Writes the opacity and animation classes, if set.
    ///
    /// Unlike Stroke/Fill, these legitimately apply to the whole `<g>` (all
    /// descendants), so they remain unscoped.
    fn write_peer_classes_opacity_animate(&self, classes: &mut String, peer_prefix_maybe: &str) {
        if let Some(opacity) = self.attrs.get(&ThemeAttr::Opacity) {
            writeln!(classes, "{peer_prefix_maybe}opacity-{opacity}")
                .expect(CLASSES_BUFFER_WRITE_FAIL);
        }
        if let Some(animate) = self.attrs.get(&ThemeAttr::Animate) {
            writeln!(classes, "{peer_prefix_maybe}animate-{animate}")
                .expect(CLASSES_BUFFER_WRITE_FAIL);
        }
    }

    /// Writes the per-state fill colour/shade classes.
    ///
    /// Fill uses shade shifting for dark mode -- the relative ordering of
    /// highlight-state shades is preserved.
    fn write_peer_classes_fill(
        &self,
        classes: &mut String,
        peer_prefix_maybe: &str,
        css_theme_vars: &mut CssThemeVars,
        dark_mode_shade_config: DarkModeShadeConfig,
    ) {
        let fill_shades = HighlightShades::resolve(|state| self.get_fill_shade(state));
        let scope_target = self.scope_target();

        [
            HighlightState::Hover,
            HighlightState::Normal,
            HighlightState::Focus,
            HighlightState::Active,
        ]
        .into_iter()
        .for_each(|state| {
            scope_target
                .fill_selector_prefixes()
                .iter()
                .for_each(|&subelement_selector_prefix| {
                    Self::write_shifted_shade_class(
                        classes,
                        css_theme_vars,
                        ShadeClassSpec {
                            peer_prefix: peer_prefix_maybe,
                            subelement_selector_prefix,
                            state_modifier: state.modifier(),
                            color_target: ColorTarget::Fill,
                            dark_mode_shade_config,
                            color: self.get_fill_color(state),
                            shade: self.get_fill_shade(state),
                            shades: fill_shades,
                        },
                    );
                });
        });
    }

    /// Writes the per-state stroke colour/shade classes.
    ///
    /// Stroke also uses shade shifting for dark mode. Colour/shade classes are
    /// skipped for states where the stroke style is `"none"`: in SVG specifying
    /// a stroke colour draws the stroke regardless of style, unlike HTML where
    /// `border-style: none` prevents drawing.
    fn write_peer_classes_stroke(
        &self,
        classes: &mut String,
        peer_prefix_maybe: &str,
        css_theme_vars: &mut CssThemeVars,
        dark_mode_shade_config: DarkModeShadeConfig,
    ) {
        let stroke_shades = HighlightShades::resolve(|state| self.get_stroke_shade(state));
        let scope_target = self.scope_target();

        [
            HighlightState::Hover,
            HighlightState::Normal,
            HighlightState::Focus,
            HighlightState::Active,
        ]
        .into_iter()
        .for_each(|state| {
            if self.get_stroke_style(state) != Some("none") {
                scope_target.stroke_selector_prefixes().iter().for_each(
                    |&subelement_selector_prefix| {
                        Self::write_shifted_shade_class(
                            classes,
                            css_theme_vars,
                            ShadeClassSpec {
                                peer_prefix: peer_prefix_maybe,
                                subelement_selector_prefix,
                                state_modifier: state.modifier(),
                                color_target: ColorTarget::Stroke,
                                dark_mode_shade_config,
                                color: self.get_stroke_color(state),
                                shade: self.get_stroke_shade(state),
                                shades: stroke_shades,
                            },
                        );
                    },
                );
            }
        });
    }

    /// Writes the outline style, width, and colour/shade classes.
    ///
    /// For edge entities the outline classes target `.locus` children via the
    /// `[&>.locus]:` arbitrary-variant prefix. For all other entities the
    /// classes are applied directly.
    fn write_peer_classes_outline(
        &self,
        classes: &mut String,
        peer_prefix_maybe: &str,
        css_theme_vars: &mut CssThemeVars,
        dark_mode_shade_config: DarkModeShadeConfig,
    ) {
        let is_edge = self.entity_type.as_ref().is_some_and(EntityType::is_edge);
        let outline_ctx = OutlineWriteCtx {
            is_edge,
            locus_selector_prefix: if is_edge { Some("[&>.locus]:") } else { None },
        };

        self.write_peer_classes_outline_style(classes, peer_prefix_maybe, outline_ctx);
        self.write_peer_classes_outline_width(classes, peer_prefix_maybe, outline_ctx);
        self.write_peer_classes_outline_color(
            classes,
            peer_prefix_maybe,
            css_theme_vars,
            dark_mode_shade_config,
            outline_ctx,
        );
    }

    /// Writes the per-state outline style classes.
    ///
    /// For non-edge entities, the standard `outline-{style}` tailwind class is
    /// used (e.g. `outline-solid`, `outline-dashed`). For edge entities, the
    /// SVG `<path>` outline does not support CSS `outline-style`; instead the
    /// style is converted to a `stroke-dasharray` value applied to the `.locus`
    /// path element.
    fn write_peer_classes_outline_style(
        &self,
        classes: &mut String,
        peer_prefix_maybe: &str,
        outline_ctx: OutlineWriteCtx<'_>,
    ) {
        let write_outline_style = if outline_ctx.is_edge {
            Self::write_outline_style_edge
        } else {
            Self::write_outline_style_node
        };
        [
            HighlightState::Normal,
            HighlightState::Hover,
            HighlightState::Focus,
            HighlightState::Active,
        ]
        .into_iter()
        .for_each(|state| {
            if let Some(style) = self.get_outline_style(state) {
                write_outline_style(
                    classes,
                    peer_prefix_maybe,
                    outline_ctx.locus_selector_prefix,
                    state.modifier(),
                    style,
                );
            }
        });
    }

    /// Writes the outline width class, if set. For edge entities the width is
    /// applied as an SVG `stroke` width on the `.locus` element.
    fn write_peer_classes_outline_width(
        &self,
        classes: &mut String,
        peer_prefix_maybe: &str,
        outline_ctx: OutlineWriteCtx<'_>,
    ) {
        let Some(width) = self.attrs.get(&ThemeAttr::OutlineWidth) else {
            return;
        };
        let locus_selector_prefix_str = outline_ctx.locus_selector_prefix.unwrap_or("");
        if outline_ctx.is_edge {
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

    /// Writes the per-state outline colour/shade classes.
    ///
    /// When a shade is available, `write_shifted_shade_class` provides
    /// dark-mode support. When only a colour is specified, an arbitrary CSS
    /// property class (`[outline-color:{color}]`, or `[stroke:{color}]` for
    /// edges) is written.
    ///
    /// When the outline style is `"none"` for a state: for edge entities
    /// `[stroke:none]` is emitted on the `.locus` prefix so the SVG stroke is
    /// explicitly cleared (SVG has no `border-style: none` equivalent); for
    /// non-edge entities the state is skipped, since CSS `outline-style: none`
    /// already prevents the outline from drawing.
    fn write_peer_classes_outline_color(
        &self,
        classes: &mut String,
        peer_prefix_maybe: &str,
        css_theme_vars: &mut CssThemeVars,
        dark_mode_shade_config: DarkModeShadeConfig,
        outline_ctx: OutlineWriteCtx<'_>,
    ) {
        let outline_color_target = if outline_ctx.is_edge {
            ColorTarget::Stroke
        } else {
            ColorTarget::Outline
        };
        let outline_color_css_prop = if outline_ctx.is_edge {
            "stroke"
        } else {
            "outline-color"
        };
        let locus_selector_prefix_str = outline_ctx.locus_selector_prefix.unwrap_or("");
        let outline_shades = HighlightShades::resolve(|state| self.get_outline_shade(state));

        [
            HighlightState::Hover,
            HighlightState::Normal,
            HighlightState::Focus,
            HighlightState::Active,
        ]
        .into_iter()
        .for_each(|state| {
            let state_modifier = state.modifier();
            if self.get_outline_style(state) == Some("none") {
                if outline_ctx.is_edge {
                    writeln!(
                        classes,
                        "{peer_prefix_maybe}{state_modifier}{locus_selector_prefix_str}[stroke:none]"
                    )
                    .expect(CLASSES_BUFFER_WRITE_FAIL);
                }
                return;
            }

            let shade = self.get_outline_shade(state);
            let color = self.get_outline_color(state);
            if shade.is_some() {
                Self::write_shifted_shade_class(
                    classes,
                    css_theme_vars,
                    ShadeClassSpec {
                        peer_prefix: peer_prefix_maybe,
                        subelement_selector_prefix: outline_ctx.locus_selector_prefix,
                        state_modifier,
                        color_target: outline_color_target,
                        dark_mode_shade_config,
                        color,
                        shade,
                        shades: outline_shades,
                    },
                );
            } else if let Some(color) = color {
                writeln!(
                    classes,
                    "{peer_prefix_maybe}{state_modifier}{locus_selector_prefix_str}[{outline_color_css_prop}:{color}]"
                )
                .expect(CLASSES_BUFFER_WRITE_FAIL);
            }
        });
    }

    /// Writes the text colour class. Text uses shade inversion for dark mode,
    /// and the `[&>text]` selector is not peer-prefixed because text colour
    /// does not change based on peer state.
    fn write_peer_classes_text(
        &self,
        classes: &mut String,
        css_theme_vars: &mut CssThemeVars,
        dark_mode_shade_config: DarkModeShadeConfig,
    ) {
        let text_color = self.attrs.get(&ThemeAttr::TextColor).map(|c| c.as_ref());
        let text_shade = self.attrs.get(&ThemeAttr::TextShade).map(|c| c.as_ref());
        if let Some((text_color, text_shade)) = text_color.zip(text_shade) {
            match dark_mode_shade_config {
                DarkModeShadeConfig::Disable => {
                    writeln!(classes, "[&>text]:fill-{text_color}-{text_shade}")
                        .expect(CLASSES_BUFFER_WRITE_FAIL);
                }
                DarkModeShadeConfig::Invert | DarkModeShadeConfig::Shift { .. } => {
                    let dark_shade = ShadeComputer::shade_inverted(text_shade);
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

    /// Writes the user-specified extra classes, if set.
    fn write_peer_classes_extra(&self, classes: &mut String) {
        if let Some(extra_classes) = self.attrs.get(&ThemeAttr::Extra) {
            classes.push_str(extra_classes.as_ref());
            classes.push('\n');
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
        if let Some(dasharray) = ShadeComputer::stroke_style_to_dasharray(outline_style) {
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
    /// * `spec`: The class position, colour, and shade context -- see
    ///   [`ShadeClassSpec`].
    fn write_shifted_shade_class(
        classes: &mut String,
        css_theme_vars: &mut CssThemeVars,
        spec: ShadeClassSpec<'_>,
    ) {
        let ShadeClassSpec {
            peer_prefix,
            subelement_selector_prefix,
            state_modifier,
            color_target,
            dark_mode_shade_config,
            color,
            shade,
            shades,
        } = spec;
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
                    let dark_shade = ShadeComputer::shade_inverted(shade);
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
                    let dark_shade = ShadeComputer::shade_shifted(
                        shade,
                        levels,
                        shades.normal,
                        shades.hover,
                        shades.focus,
                        shades.active,
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

impl HighlightState {
    /// Returns the tailwind state-variant modifier for this state, e.g.
    /// `"hover:"`, `"focus:"`, `"active:"`, or `""` for normal.
    fn modifier(self) -> &'static str {
        match self {
            HighlightState::Normal => "",
            HighlightState::Hover => "hover:",
            HighlightState::Focus => "focus:",
            HighlightState::Active => "active:",
        }
    }
}

/// The resolved shade for each highlight state of a single colour target.
///
/// Used to determine the dark-mode shift direction (and the `_500`
/// tie-breaker) when shifting shades -- see [`ShadeComputer::shade_shifted`].
#[derive(Clone, Copy)]
struct HighlightShades<'a> {
    normal: Option<&'a str>,
    hover: Option<&'a str>,
    focus: Option<&'a str>,
    active: Option<&'a str>,
}

impl<'a> HighlightShades<'a> {
    /// Resolves the shade for each highlight state via `shade_for`.
    fn resolve(shade_for: impl Fn(HighlightState) -> Option<&'a str>) -> Self {
        HighlightShades {
            normal: shade_for(HighlightState::Normal),
            hover: shade_for(HighlightState::Hover),
            focus: shade_for(HighlightState::Focus),
            active: shade_for(HighlightState::Active),
        }
    }
}

/// The sub-element(s) that Stroke/Fill-derived classes should be scoped to,
/// determined from [`TailwindClassState::entity_type`] via
/// [`TailwindClassState::scope_target`].
///
/// Mirrors the existing `.locus` scoping used for Outline classes (see
/// [`OutlineWriteCtx`]), generalized to Stroke/Fill and to nodes as well as
/// edges.
#[derive(Clone, Copy)]
enum ScopeTarget {
    /// Not a node or edge (halo / halo-outline / label-desc-bg / custom /
    /// container-inbuilt entity types, or no entity type at all) -- these
    /// represent rendering-only style keys or standalone `<path>` elements
    /// with no sibling shape paths to leak onto, so classes are written
    /// unscoped, exactly as before this scoping was introduced.
    Unscoped,
    /// A node (thing / tag / process / process step). It is not known at
    /// class-resolution time whether the node will use a circle shape, so
    /// both `.wrapper` and `.circle` selectors are emitted.
    Node,
    /// An edge. Stroke scopes to `.edge_body` (the line); Fill scopes to
    /// `.arrow_head` (the arrow head fill), matching the `ThemeAttr` doc
    /// comments.
    Edge,
}

impl ScopeTarget {
    /// Selector prefixes for Stroke-derived classes (colour, dasharray,
    /// width).
    fn stroke_selector_prefixes(self) -> &'static [Option<&'static str>] {
        match self {
            ScopeTarget::Unscoped => &[None],
            ScopeTarget::Node => &[Some(NODE_WRAPPER_SELECTOR), Some(NODE_CIRCLE_SELECTOR)],
            ScopeTarget::Edge => &[Some(EDGE_BODY_SELECTOR), Some(EDGE_ARROW_HEAD_SELECTOR)],
        }
    }

    /// Selector prefixes for Fill-derived classes (colour).
    fn fill_selector_prefixes(self) -> &'static [Option<&'static str>] {
        match self {
            ScopeTarget::Unscoped => &[None],
            ScopeTarget::Node => &[Some(NODE_WRAPPER_SELECTOR), Some(NODE_CIRCLE_SELECTOR)],
            ScopeTarget::Edge => &[Some(EDGE_ARROW_HEAD_SELECTOR)],
        }
    }
}

/// Immutable context for writing outline classes, distinguishing edge entities
/// (which target the `.locus` child element) from other entities.
#[derive(Clone, Copy)]
struct OutlineWriteCtx<'a> {
    /// Whether the entity is an edge.
    is_edge: bool,
    /// The `[&>.locus]:` arbitrary-variant prefix for edge entities, or `None`.
    locus_selector_prefix: Option<&'a str>,
}

/// The class position, colour, and shade context for writing a single shifted
/// shade class via [`TailwindClassState::write_shifted_shade_class`].
#[derive(Clone, Copy)]
struct ShadeClassSpec<'a> {
    /// The peer prefix for the class, e.g. `"peer-[:focus-within]/tag:"` or
    /// `""`.
    peer_prefix: &'a str,
    /// The sub-element selector prefix, e.g. `Some("[&>.locus]:")` for edges.
    subelement_selector_prefix: Option<&'a str>,
    /// The highlight state modifier, e.g. `"hover:"`, `"focus:"`, `"active:"`,
    /// or `""` for normal.
    state_modifier: &'a str,
    /// The color target -- fill, stroke, or outline.
    color_target: ColorTarget,
    /// Controls how dark-mode shades are computed.
    dark_mode_shade_config: DarkModeShadeConfig,
    /// The resolved colour name for this state, e.g. `"yellow"`, `"slate"`.
    color: Option<&'a str>,
    /// The resolved shade value for this state, e.g. `"100"`.
    shade: Option<&'a str>,
    /// The resolved shade for every highlight state (used to determine the
    /// dark-mode shift direction).
    shades: HighlightShades<'a>,
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
