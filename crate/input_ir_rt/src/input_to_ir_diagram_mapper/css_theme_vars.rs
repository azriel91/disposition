use std::fmt::Write;

use disposition_model_common::Set;

use super::tailwind_colors::tailwind_color_lookup;

/// A CSS variable entry representing a light/dark color pair.
///
/// # Examples
///
/// For a light shade of `"100"` and dark shade of `"900"` on color `"blue"`,
/// the variable name would be `"--tw-blue-100-900"` with light value
/// `"oklch(93.2% 0.032 255.585)"` and dark value
/// `"oklch(37.9% 0.146 265.522)"`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct CssThemeVar {
    /// The CSS variable name, e.g. `"--tw-blue-100-900"`.
    var_name: String,
    /// The oklch value for light mode, e.g. `"oklch(93.2% 0.032 255.585)"`.
    light_value: &'static str,
    /// The oklch value for dark mode, e.g. `"oklch(37.9% 0.146 265.522)"`.
    dark_value: &'static str,
}

/// Collects CSS variable definitions for dark/light mode color pairs.
///
/// Instead of generating `dark:` prefixed tailwind classes (which rely on
/// `@media (prefers-color-scheme: dark)` and conflict when a surrounding
/// website has its own light/dark toggle), this struct tracks CSS variables
/// whose values change based on the user's preferred color scheme.
///
/// Each unique `(color, light_shade, dark_shade)` combination produces one
/// CSS variable. The variable is defined in an `svg { ... }` block for
/// light mode and overridden inside
/// `@media (prefers-color-scheme: dark) { svg { ... } }` for dark mode.
///
/// Elements then reference these variables via tailwind's
/// `fill-(--var-name)` / `stroke-(--var-name)` syntax instead of using
/// separate light and dark class pairs.
#[derive(Clone, Debug, Default)]
pub(crate) struct CssThemeVars {
    /// Deduplicated set of CSS variable entries.
    vars: Set<CssThemeVar>,
}

impl CssThemeVars {
    /// Register a color+shade pair and return the CSS variable name.
    ///
    /// If the `(color, light_shade, dark_shade)` combination has already been
    /// registered, the existing variable name is returned without creating a
    /// duplicate.
    ///
    /// If either the light or dark shade cannot be looked up in the tailwind
    /// color table, returns `None` (the caller should fall back to emitting
    /// the class directly without a variable).
    ///
    /// # Parameters
    ///
    /// * `color`: The tailwind color name, e.g. `"blue"`, `"slate"`.
    /// * `light_shade`: The shade for light mode, e.g. `"100"`.
    /// * `dark_shade`: The shade for dark mode, e.g. `"900"`.
    ///
    /// # Returns
    ///
    /// The CSS variable name without the `var(...)` wrapper, e.g.
    /// `"--tw-blue-100-900"`.
    pub(crate) fn register(
        &mut self,
        color: &str,
        light_shade: &str,
        dark_shade: &str,
    ) -> Option<String> {
        let light_value = tailwind_color_lookup(color, light_shade)?;
        let dark_value = tailwind_color_lookup(color, dark_shade)?;

        let var_name = format!("--tw-{color}-{light_shade}-{dark_shade}");

        self.vars.insert(CssThemeVar {
            var_name: var_name.clone(),
            light_value,
            dark_value,
        });

        Some(var_name)
    }

    /// Returns `true` if no variables have been registered.
    pub(crate) fn is_empty(&self) -> bool {
        self.vars.is_empty()
    }

    /// Generate the CSS text containing variable definitions for both light
    /// and dark modes.
    ///
    /// The output looks like:
    ///
    /// ```css
    /// svg {
    ///   --tw-blue-100-900: oklch(93.2% 0.032 255.585);
    ///   --tw-neutral-900-100: oklch(20.5% 0 0);
    /// }
    /// @media (prefers-color-scheme: dark) {
    ///   svg {
    ///     --tw-blue-100-900: oklch(37.9% 0.146 265.522);
    ///     --tw-neutral-900-100: oklch(97% 0 0);
    ///   }
    /// }
    /// ```
    ///
    /// Returns an empty string when no variables have been registered.
    pub(crate) fn to_css(&self) -> String {
        if self.vars.is_empty() {
            return String::new();
        }

        // Sort variables by name for deterministic output.
        let mut sorted_vars: Vec<&CssThemeVar> = self.vars.iter().collect();
        sorted_vars.sort_by_key(|css_theme_var| &css_theme_var.var_name);

        let mut css = String::with_capacity(sorted_vars.len() * 80);

        // === Light mode variables === //
        css.push_str("svg {\n");
        for css_theme_var in &sorted_vars {
            writeln!(
                css,
                "  {}: {};",
                css_theme_var.var_name, css_theme_var.light_value
            )
            .expect("Failed to write CSS variable");
        }
        css.push_str("}\n");

        // === Dark mode variables === //
        css.push_str("@media (prefers-color-scheme: dark) {\n  svg {\n");
        for css_theme_var in &sorted_vars {
            writeln!(
                css,
                "    {}: {};",
                css_theme_var.var_name, css_theme_var.dark_value
            )
            .expect("Failed to write CSS variable");
        }
        css.push_str("  }\n}");

        css
    }
}

// === Tests === //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_returns_var_name_for_known_color() {
        let mut css_theme_vars = CssThemeVars::default();
        let var_name = css_theme_vars.register("blue", "100", "900");
        assert_eq!(var_name, Some("--tw-blue-100-900".to_string()));
    }

    #[test]
    fn register_returns_none_for_unknown_color() {
        let mut css_theme_vars = CssThemeVars::default();
        let var_name = css_theme_vars.register("nonexistent", "100", "900");
        assert_eq!(var_name, None);
    }

    #[test]
    fn register_returns_none_for_unknown_shade() {
        let mut css_theme_vars = CssThemeVars::default();
        let var_name = css_theme_vars.register("blue", "100", "999");
        assert_eq!(var_name, None);
    }

    #[test]
    fn register_deduplicates_same_combination() {
        let mut css_theme_vars = CssThemeVars::default();
        css_theme_vars.register("blue", "100", "900");
        css_theme_vars.register("blue", "100", "900");
        assert_eq!(css_theme_vars.vars.len(), 1);
    }

    #[test]
    fn register_tracks_different_combinations() {
        let mut css_theme_vars = CssThemeVars::default();
        css_theme_vars.register("blue", "100", "900");
        css_theme_vars.register("blue", "200", "800");
        css_theme_vars.register("red", "100", "900");
        assert_eq!(css_theme_vars.vars.len(), 3);
    }

    #[test]
    fn is_empty_returns_true_when_no_vars_registered() {
        let css_theme_vars = CssThemeVars::default();
        assert!(css_theme_vars.is_empty());
    }

    #[test]
    fn is_empty_returns_false_after_registration() {
        let mut css_theme_vars = CssThemeVars::default();
        css_theme_vars.register("blue", "100", "900");
        assert!(!css_theme_vars.is_empty());
    }

    #[test]
    fn to_css_returns_empty_string_when_no_vars() {
        let css_theme_vars = CssThemeVars::default();
        assert_eq!(css_theme_vars.to_css(), "");
    }

    #[test]
    fn to_css_generates_light_and_dark_blocks() {
        let mut css_theme_vars = CssThemeVars::default();
        css_theme_vars.register("blue", "100", "900");

        let css = css_theme_vars.to_css();

        assert!(
            css.contains("svg {"),
            "should contain light mode svg block: {css}"
        );
        assert!(
            css.contains("--tw-blue-100-900: oklch(93.2% 0.032 255.585);"),
            "should contain light mode value: {css}"
        );
        assert!(
            css.contains("@media (prefers-color-scheme: dark)"),
            "should contain dark mode media query: {css}"
        );
        assert!(
            css.contains("--tw-blue-100-900: oklch(37.9% 0.146 265.522);"),
            "should contain dark mode value: {css}"
        );
    }

    #[test]
    fn to_css_sorts_variables_by_name() {
        let mut css_theme_vars = CssThemeVars::default();
        css_theme_vars.register("red", "100", "900");
        css_theme_vars.register("blue", "100", "900");

        let css = css_theme_vars.to_css();

        let blue_pos = css.find("--tw-blue").expect("blue var should exist");
        let red_pos = css.find("--tw-red").expect("red var should exist");
        assert!(
            blue_pos < red_pos,
            "blue should come before red alphabetically"
        );
    }

    #[test]
    fn to_css_handles_same_shade_for_light_and_dark() {
        let mut css_theme_vars = CssThemeVars::default();
        css_theme_vars.register("blue", "500", "500");

        let css = css_theme_vars.to_css();

        assert!(
            css.contains("--tw-blue-500-500: oklch(62.3% 0.214 259.815);"),
            "should use the 500 shade value for both: {css}"
        );
    }
}
