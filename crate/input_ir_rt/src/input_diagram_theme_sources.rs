use disposition_input_model::InputDiagram;
use disposition_input_rt::id_parse;

use crate::ThemeValueSource;

/// Computes where theme field values came from by comparing the user's
/// overlay diagram against the base diagram.
///
/// For each theme map entry, the source is:
///
/// * `UserInput` -- if the key is present in the overlay diagram.
/// * `BaseDiagram` -- if the key is only present in the base diagram.
///
/// The computation is done on-the-fly by checking the overlay diagram.
/// Since the overlay is typically small, this is efficient.
///
/// # Examples
///
/// ```rust,ignore
/// use disposition_input_model::InputDiagram;
/// use disposition_input_ir_rt::{InputDiagramThemeSources, ThemeValueSource};
///
/// let base = InputDiagram::base();
/// let overlay = InputDiagram::new();
/// let sources = InputDiagramThemeSources::new(&base, &overlay);
///
/// // A key only in the base diagram:
/// assert_eq!(
///     sources.base_styles_entry_source("node_defaults"),
///     ThemeValueSource::BaseDiagram,
/// );
/// ```
#[derive(Clone, Debug)]
pub struct InputDiagramThemeSources<'a> {
    /// The base diagram (typically `InputDiagram::base()`).
    base: &'a InputDiagram<'static>,
    /// The user's overlay diagram (before merging).
    overlay: &'a InputDiagram<'static>,
}

impl<'a> InputDiagramThemeSources<'a> {
    /// Creates a new `InputDiagramThemeSources` from a base diagram and
    /// a user overlay diagram.
    ///
    /// # Parameters
    ///
    /// * `base` -- the base diagram, typically from `InputDiagram::base()`.
    /// * `overlay` -- the user's overlay diagram before merging.
    pub fn new(base: &'a InputDiagram<'static>, overlay: &'a InputDiagram<'static>) -> Self {
        Self { base, overlay }
    }

    /// Returns a reference to the base diagram.
    pub fn base(&self) -> &'a InputDiagram<'static> {
        self.base
    }

    /// Returns a reference to the overlay diagram.
    pub fn overlay(&self) -> &'a InputDiagram<'static> {
        self.overlay
    }

    // === Style Aliases === //

    /// Returns the source of a style alias entry.
    ///
    /// `alias_key` is the string form of a `StyleAlias`, e.g.
    /// `"shade_light"` or `"padding_normal"`.
    ///
    /// Returns `UserInput` if the key is present in the overlay's
    /// `theme_default.style_aliases`, otherwise `BaseDiagram`.
    ///
    /// # Panics
    ///
    /// Returns `BaseDiagram` if `alias_key` cannot be parsed into a
    /// valid `StyleAlias` (the key does not exist in either diagram).
    pub fn style_alias_source(&self, alias_key: &str) -> ThemeValueSource {
        let Some(alias) = id_parse::parse_style_alias(alias_key) else {
            return ThemeValueSource::BaseDiagram;
        };
        if self
            .overlay
            .theme_default
            .style_aliases
            .contains_key(&alias)
        {
            ThemeValueSource::UserInput
        } else {
            ThemeValueSource::BaseDiagram
        }
    }

    // === ThemeDefault Base Styles === //

    /// Returns the source for
    /// `theme_default.base_styles[entry_key]`.
    ///
    /// `entry_key` is the string form of an `IdOrDefaults`, e.g.
    /// `"node_defaults"`, `"edge_defaults"`, or a custom entity ID
    /// like `"t_aws"`.
    ///
    /// Returns `UserInput` if the key is present in the overlay's
    /// `theme_default.base_styles`, otherwise `BaseDiagram`.
    pub fn base_styles_entry_source(&self, entry_key: &str) -> ThemeValueSource {
        let Some(id_or_defaults) = id_parse::parse_id_or_defaults(entry_key) else {
            return ThemeValueSource::BaseDiagram;
        };
        if self
            .overlay
            .theme_default
            .base_styles
            .contains_key(&id_or_defaults)
        {
            ThemeValueSource::UserInput
        } else {
            ThemeValueSource::BaseDiagram
        }
    }

    // === ThemeDefault Process Step Selected Styles === //

    /// Returns the source for
    /// `theme_default.process_step_selected_styles[entry_key]`.
    ///
    /// `entry_key` is the string form of an `IdOrDefaults`, e.g.
    /// `"node_defaults"` or `"edge_defaults"`.
    ///
    /// Returns `UserInput` if the key is present in the overlay's
    /// `theme_default.process_step_selected_styles`, otherwise
    /// `BaseDiagram`.
    pub fn process_step_selected_styles_entry_source(&self, entry_key: &str) -> ThemeValueSource {
        let Some(id_or_defaults) = id_parse::parse_id_or_defaults(entry_key) else {
            return ThemeValueSource::BaseDiagram;
        };
        if self
            .overlay
            .theme_default
            .process_step_selected_styles
            .contains_key(&id_or_defaults)
        {
            ThemeValueSource::UserInput
        } else {
            ThemeValueSource::BaseDiagram
        }
    }

    // === Theme Types Styles (Outer Key) === //

    /// Returns the source for `theme_types_styles[type_key]` (outer
    /// key).
    ///
    /// `type_key` is the string form of an `EntityTypeId`, e.g.
    /// `"type_thing_default"` or `"type_organisation"`.
    ///
    /// Returns `UserInput` if the key is present in the overlay's
    /// `theme_types_styles`, otherwise `BaseDiagram`.
    pub fn types_styles_key_source(&self, type_key: &str) -> ThemeValueSource {
        let Some(entity_type_id) = id_parse::parse_entity_type_id(type_key) else {
            return ThemeValueSource::BaseDiagram;
        };
        if self
            .overlay
            .theme_types_styles
            .contains_key(&entity_type_id)
        {
            ThemeValueSource::UserInput
        } else {
            ThemeValueSource::BaseDiagram
        }
    }

    // === Theme Types Styles (Inner Entry) === //

    /// Returns the source for
    /// `theme_types_styles[type_key][entry_key]` (inner entry).
    ///
    /// `type_key` is the string form of an `EntityTypeId`, e.g.
    /// `"type_thing_default"`.
    /// `entry_key` is the string form of an `IdOrDefaults`, e.g.
    /// `"node_defaults"`.
    ///
    /// Returns `UserInput` if the overlay's
    /// `theme_types_styles[type_key]` map contains `entry_key`,
    /// otherwise `BaseDiagram`.
    pub fn types_styles_entry_source(&self, type_key: &str, entry_key: &str) -> ThemeValueSource {
        let Some(entity_type_id) = id_parse::parse_entity_type_id(type_key) else {
            return ThemeValueSource::BaseDiagram;
        };
        let Some(id_or_defaults) = id_parse::parse_id_or_defaults(entry_key) else {
            return ThemeValueSource::BaseDiagram;
        };
        let theme_styles = self.overlay.theme_types_styles.get(&entity_type_id);
        match theme_styles {
            Some(theme_styles) if theme_styles.contains_key(&id_or_defaults) => {
                ThemeValueSource::UserInput
            }
            _ => ThemeValueSource::BaseDiagram,
        }
    }

    // === Theme Thing Dependencies Styles (Included) === //

    /// Returns the source for
    /// `theme_thing_dependencies_styles.things_included_styles[entry_key]`.
    ///
    /// `entry_key` is the string form of an `IdOrDefaults`, e.g.
    /// `"node_defaults"` or `"edge_defaults"`.
    ///
    /// Returns `UserInput` if the key is present in the overlay's
    /// `theme_thing_dependencies_styles.things_included_styles`,
    /// otherwise `BaseDiagram`.
    pub fn dependencies_included_entry_source(&self, entry_key: &str) -> ThemeValueSource {
        let Some(id_or_defaults) = id_parse::parse_id_or_defaults(entry_key) else {
            return ThemeValueSource::BaseDiagram;
        };
        if self
            .overlay
            .theme_thing_dependencies_styles
            .things_included_styles
            .contains_key(&id_or_defaults)
        {
            ThemeValueSource::UserInput
        } else {
            ThemeValueSource::BaseDiagram
        }
    }

    // === Theme Thing Dependencies Styles (Excluded) === //

    /// Returns the source for
    /// `theme_thing_dependencies_styles.things_excluded_styles[entry_key]`.
    ///
    /// `entry_key` is the string form of an `IdOrDefaults`, e.g.
    /// `"node_excluded_defaults"`.
    ///
    /// Returns `UserInput` if the key is present in the overlay's
    /// `theme_thing_dependencies_styles.things_excluded_styles`,
    /// otherwise `BaseDiagram`.
    pub fn dependencies_excluded_entry_source(&self, entry_key: &str) -> ThemeValueSource {
        let Some(id_or_defaults) = id_parse::parse_id_or_defaults(entry_key) else {
            return ThemeValueSource::BaseDiagram;
        };
        if self
            .overlay
            .theme_thing_dependencies_styles
            .things_excluded_styles
            .contains_key(&id_or_defaults)
        {
            ThemeValueSource::UserInput
        } else {
            ThemeValueSource::BaseDiagram
        }
    }

    // === Theme Tag Things Focus (Outer Key) === //

    /// Returns the source for `theme_tag_things_focus[tag_key]`
    /// (outer key).
    ///
    /// `tag_key` is the string form of a `TagIdOrDefaults`, e.g.
    /// `"tag_defaults"` or `"tag_app_development"`.
    ///
    /// Returns `UserInput` if the key is present in the overlay's
    /// `theme_tag_things_focus`, otherwise `BaseDiagram`.
    pub fn tag_focus_key_source(&self, tag_key: &str) -> ThemeValueSource {
        let Some(tag_id_or_defaults) = id_parse::parse_tag_id_or_defaults(tag_key) else {
            return ThemeValueSource::BaseDiagram;
        };
        if self
            .overlay
            .theme_tag_things_focus
            .contains_key(&tag_id_or_defaults)
        {
            ThemeValueSource::UserInput
        } else {
            ThemeValueSource::BaseDiagram
        }
    }

    // === Theme Tag Things Focus (Inner Entry) === //

    /// Returns the source for
    /// `theme_tag_things_focus[tag_key][entry_key]` (inner entry).
    ///
    /// `tag_key` is the string form of a `TagIdOrDefaults`, e.g.
    /// `"tag_defaults"`.
    /// `entry_key` is the string form of an `IdOrDefaults`, e.g.
    /// `"node_defaults"` or `"node_excluded_defaults"`.
    ///
    /// Returns `UserInput` if the overlay's
    /// `theme_tag_things_focus[tag_key]` map contains `entry_key`,
    /// otherwise `BaseDiagram`.
    pub fn tag_focus_entry_source(&self, tag_key: &str, entry_key: &str) -> ThemeValueSource {
        let Some(tag_id_or_defaults) = id_parse::parse_tag_id_or_defaults(tag_key) else {
            return ThemeValueSource::BaseDiagram;
        };
        let Some(id_or_defaults) = id_parse::parse_id_or_defaults(entry_key) else {
            return ThemeValueSource::BaseDiagram;
        };
        let theme_styles = self.overlay.theme_tag_things_focus.get(&tag_id_or_defaults);
        match theme_styles {
            Some(theme_styles) if theme_styles.contains_key(&id_or_defaults) => {
                ThemeValueSource::UserInput
            }
            _ => ThemeValueSource::BaseDiagram,
        }
    }
}
