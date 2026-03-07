//! Mutation helpers for the style aliases section.
//!
//! This module is a thin Signal-aware wrapper around
//! [`disposition_input_rt::style_aliases_section_ops::StyleAliasesSectionOps`].
//! Each method acquires a write guard on the [`Signal`] and delegates to the
//! framework-agnostic implementation.

use dioxus::signals::{Signal, WritableExt};
use disposition::input_model::InputDiagram;

/// Groups mutation helpers for the style aliases section.
pub(crate) struct StyleAliasesSectionOps;

impl StyleAliasesSectionOps {
    /// Renames a style alias key across the entire [`InputDiagram`].
    ///
    /// 1. Replaces the key in `theme_default.style_aliases`.
    /// 2. Walks every `ThemeStyles` map reachable from the diagram
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
    /// * `input_diagram`: signal holding the diagram to mutate.
    /// * `alias_old_str`: the old alias name string, e.g. `"shade_light"`.
    /// * `alias_new_str`: the new alias name string, e.g. `"shade_pale"`.
    pub(crate) fn style_alias_rename(
        mut input_diagram: Signal<InputDiagram<'static>>,
        alias_old_str: &str,
        alias_new_str: &str,
    ) {
        disposition_input_rt::style_aliases_section_ops::StyleAliasesSectionOps::style_alias_rename(
            &mut input_diagram.write(),
            alias_old_str,
            alias_new_str,
        );
    }
}
