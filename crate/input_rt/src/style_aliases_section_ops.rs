//! Mutation helpers for the style aliases section.
//!
//! Provides the `StyleAliasesSectionOps` type that groups operations for
//! renaming a `StyleAlias` key in `theme_default.style_aliases` and
//! propagating that rename across every `style_aliases_applied` list in the
//! `InputDiagram`.
//!
//! All methods operate on `&mut InputDiagram<'static>` instead of a
//! framework-specific signal type, making them testable without a UI runtime.

use disposition_input_model::{
    theme::{StyleAlias, ThemeStyles},
    InputDiagram,
};

use crate::id_parse::parse_style_alias;

/// Groups mutation helpers for the style aliases section.
pub struct StyleAliasesSectionOps;

impl StyleAliasesSectionOps {
    /// Renames a style alias key across the entire [`InputDiagram`].
    ///
    /// 1. Replaces the key in `theme_default.style_aliases`.
    /// 2. Walks every [`ThemeStyles`] map reachable from the diagram
    ///    (`theme_default.base_styles`,
    ///    `theme_default.process_step_selected_styles`, `theme_types_styles`,
    ///    `theme_thing_dependencies_styles`, `theme_tag_things_focus`) and
    ///    replaces matching entries inside each
    ///    `CssClassPartials.style_aliases_applied` vector.
    /// 3. Also walks the `style_aliases_applied` vectors inside
    ///    `theme_default.style_aliases` values themselves, since one alias
    ///    definition can reference another.
    ///
    /// # Parameters
    ///
    /// * `input_diagram`: the diagram to mutate.
    /// * `alias_old_str`: the old alias name string, e.g. `"shade_light"`.
    /// * `alias_new_str`: the new alias name string, e.g. `"shade_pale"`.
    pub fn style_alias_rename(
        input_diagram: &mut InputDiagram<'static>,
        alias_old_str: &str,
        alias_new_str: &str,
    ) {
        if alias_old_str == alias_new_str {
            return;
        }

        let alias_old = match parse_style_alias(alias_old_str) {
            Some(a) => a,
            None => return,
        };
        let alias_new = match parse_style_alias(alias_new_str) {
            Some(a) => a,
            None => return,
        };

        // === 1. Rename the key in style_aliases === //
        if input_diagram
            .theme_default
            .style_aliases
            .contains_key(&alias_new)
        {
            // Target key already exists -- bail out to avoid data loss.
            return;
        }
        if let Some(idx) = input_diagram
            .theme_default
            .style_aliases
            .get_index_of(&alias_old)
        {
            input_diagram
                .theme_default
                .style_aliases
                .replace_index(idx, alias_new.clone())
                .expect(
                    "Expected new key to be unique; \
                     checked for availability above",
                );
        } else {
            // Old key not found -- nothing to rename.
            return;
        }

        // === 2. Rename inside style_aliases values === //
        for css_partials in input_diagram.theme_default.style_aliases.values_mut() {
            Self::rename_in_applied(
                &mut css_partials.style_aliases_applied,
                &alias_old,
                &alias_new,
            );
        }

        // === 3. Rename inside theme_default.base_styles === //
        Self::rename_in_theme_styles(
            &mut input_diagram.theme_default.base_styles,
            &alias_old,
            &alias_new,
        );

        // === 4. Rename inside theme_default.process_step_selected_styles === //
        Self::rename_in_theme_styles(
            &mut input_diagram.theme_default.process_step_selected_styles,
            &alias_old,
            &alias_new,
        );

        // === 5. Rename inside theme_types_styles === //
        for theme_styles in input_diagram.theme_types_styles.values_mut() {
            Self::rename_in_theme_styles(theme_styles, &alias_old, &alias_new);
        }

        // === 6. Rename inside theme_thing_dependencies_styles === //
        Self::rename_in_theme_styles(
            &mut input_diagram
                .theme_thing_dependencies_styles
                .things_included_styles,
            &alias_old,
            &alias_new,
        );
        Self::rename_in_theme_styles(
            &mut input_diagram
                .theme_thing_dependencies_styles
                .things_excluded_styles,
            &alias_old,
            &alias_new,
        );

        // === 7. Rename inside theme_tag_things_focus === //
        for theme_styles in input_diagram.theme_tag_things_focus.values_mut() {
            Self::rename_in_theme_styles(theme_styles, &alias_old, &alias_new);
        }
    }

    /// Replaces all occurrences of `old` with `new` in a single
    /// `style_aliases_applied` vector.
    fn rename_in_applied(
        applied: &mut Vec<StyleAlias<'static>>,
        old: &StyleAlias<'static>,
        new: &StyleAlias<'static>,
    ) {
        for alias in applied.iter_mut() {
            if alias == old {
                *alias = new.clone();
            }
        }
    }

    /// Walks every `CssClassPartials` value inside a [`ThemeStyles`] map and
    /// replaces matching entries in each `style_aliases_applied` vector.
    fn rename_in_theme_styles(
        theme_styles: &mut ThemeStyles<'static>,
        old: &StyleAlias<'static>,
        new: &StyleAlias<'static>,
    ) {
        for css_partials in theme_styles.values_mut() {
            Self::rename_in_applied(&mut css_partials.style_aliases_applied, old, new);
        }
    }
}
