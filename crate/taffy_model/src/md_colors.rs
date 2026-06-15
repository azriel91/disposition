//! Markdown span colour choices, expressed as Tailwind theme-variable colour
//! pairs.
//!
//! These colours are registered with the `CssThemeVars` system in `input_ir_rt`
//! so their values flip between light and dark per the configured
//! `DarkModeCssSelector`, and they are referenced from markdown spans via
//! `fill-[var(--tw-{color}-{light}-{dark})]` Tailwind classes. Keeping the
//! colour parts here as the single source of truth ensures the registered CSS
//! variable name and the referencing class always agree.

/// A Tailwind colour name plus its light- and dark-mode shades.
///
/// Used to build a `--tw-{color}-{light}-{dark}` CSS theme variable (via
/// `CssThemeVars::register`) and the `fill-[var(...)]` Tailwind class that
/// references it.
///
/// # Examples
///
/// ```rust
/// use disposition_taffy_model::MdColor;
///
/// let md_color = MdColor {
///     color: "blue",
///     shade_light: "700",
///     shade_dark: "400",
/// };
/// assert_eq!(md_color.fill_class(), "fill-[var(--tw-blue-700-400)]");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MdColor {
    /// The Tailwind colour name, e.g. `"blue"`, `"neutral"`.
    pub color: &'static str,
    /// The shade used in light mode, e.g. `"700"`.
    pub shade_light: &'static str,
    /// The shade used in dark mode, e.g. `"400"`.
    pub shade_dark: &'static str,
}

impl MdColor {
    /// Returns the `fill-[var(--tw-{color}-{light}-{dark})]` Tailwind class
    /// that references this colour's CSS theme variable.
    pub fn fill_class(&self) -> String {
        format!(
            "fill-[var(--tw-{}-{}-{})]",
            self.color, self.shade_light, self.shade_dark
        )
    }
}

/// Inline-code background fill: a neutral grey (light `neutral-200`, dark
/// `neutral-700`), closely matching the previous `#e8e8e8` / `#3a3a3a`.
pub const MD_CODE_BG_COLOR: MdColor = MdColor {
    color: "neutral",
    shade_light: "200",
    shade_dark: "700",
};

/// Link text fill: blue with good contrast in both modes (light `blue-700`,
/// dark `blue-400`).
pub const MD_LINK_COLOR: MdColor = MdColor {
    color: "blue",
    shade_light: "700",
    shade_dark: "400",
};
