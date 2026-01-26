use disposition_input_model::{
    entity::{EntityDescs, EntityTypes},
    process::Processes,
    tag::{TagNames, TagThings},
    theme::{
        StyleAliases, ThemeDefault, ThemeStyles, ThemeTagThingsFocus, ThemeThingDependenciesStyles,
        ThemeTypesStyles,
    },
    thing::{ThingCopyText, ThingDependencies, ThingHierarchy, ThingInteractions, ThingNames},
    InputDiagram,
};
use disposition_model_common::{entity::EntityTooltips, theme::Css};

/// Merges an input diagram over another.
#[derive(Clone, Copy, Debug)]
pub struct InputDiagramMerger;

impl InputDiagramMerger {
    /// Merges an overlay `InputDiagram` over a base `InputDiagram`.
    ///
    /// The merge strategy is:
    /// - For map-like fields: overlay values override base values for the same
    ///   key, base values without overlay counterparts are preserved.
    /// - For nested structures (like `ThemeDefault`): each sub-field is merged
    ///   recursively.
    /// - For `css`: the overlay value completely replaces the base value if
    ///   non-empty.
    ///
    /// # Parameters
    ///
    /// * `base_diagram` - The base diagram providing default values (typically
    ///   from `InputDiagram::base()`).
    /// * `overlay_diagram` - The overlay diagram with user-specified values
    ///   that take precedence.
    ///
    /// # Returns
    ///
    /// A new `InputDiagram` containing the merged result.
    pub fn merge<'f, 'id>(
        base_diagram: InputDiagram<'static>,
        overlay_diagram: &'f InputDiagram<'id>,
    ) -> InputDiagram<'id>
    where
        'id: 'f,
    {
        let things = Self::merge_thing_names(base_diagram.things, &overlay_diagram.things);
        let thing_copy_text = Self::merge_thing_copy_text(
            base_diagram.thing_copy_text,
            &overlay_diagram.thing_copy_text,
        );
        let thing_hierarchy = Self::merge_thing_hierarchy(
            base_diagram.thing_hierarchy,
            &overlay_diagram.thing_hierarchy,
        );
        let thing_dependencies = Self::merge_thing_dependencies(
            base_diagram.thing_dependencies,
            &overlay_diagram.thing_dependencies,
        );
        let thing_interactions = Self::merge_thing_interactions(
            base_diagram.thing_interactions,
            &overlay_diagram.thing_interactions,
        );
        let processes = Self::merge_processes(base_diagram.processes, &overlay_diagram.processes);
        let tags = Self::merge_tag_names(base_diagram.tags, &overlay_diagram.tags);
        let tag_things =
            Self::merge_tag_things(base_diagram.tag_things, &overlay_diagram.tag_things);
        let entity_descs =
            Self::merge_entity_descs(base_diagram.entity_descs, &overlay_diagram.entity_descs);
        let entity_tooltips = Self::merge_entity_tooltips(
            base_diagram.entity_tooltips,
            &overlay_diagram.entity_tooltips,
        );
        let entity_types =
            Self::merge_entity_types(base_diagram.entity_types, &overlay_diagram.entity_types);
        let theme_default =
            Self::merge_theme_default(base_diagram.theme_default, &overlay_diagram.theme_default);
        let theme_types_styles = Self::merge_theme_types_styles(
            base_diagram.theme_types_styles,
            &overlay_diagram.theme_types_styles,
        );
        let theme_thing_dependencies_styles = Self::merge_theme_thing_dependencies_styles(
            base_diagram.theme_thing_dependencies_styles,
            &overlay_diagram.theme_thing_dependencies_styles,
        );
        let theme_tag_things_focus = Self::merge_theme_tag_things_focus(
            base_diagram.theme_tag_things_focus,
            &overlay_diagram.theme_tag_things_focus,
        );
        let css = Self::merge_css(base_diagram.css, &overlay_diagram.css);

        InputDiagram {
            things,
            thing_copy_text,
            thing_hierarchy,
            thing_dependencies,
            thing_interactions,
            processes,
            tags,
            tag_things,
            entity_descs,
            entity_tooltips,
            entity_types,
            theme_default,
            theme_types_styles,
            theme_thing_dependencies_styles,
            theme_tag_things_focus,
            css,
        }
    }

    fn merge_thing_names<'id>(
        base: ThingNames<'static>,
        overlay: &ThingNames<'id>,
    ) -> ThingNames<'id> {
        let mut result = base;
        overlay.iter().for_each(|(key, value)| {
            result.insert(key.clone(), value.clone());
        });
        result
    }

    fn merge_thing_copy_text<'id>(
        base: ThingCopyText<'static>,
        overlay: &ThingCopyText<'id>,
    ) -> ThingCopyText<'id> {
        let mut result = base;
        overlay.iter().for_each(|(key, value)| {
            result.insert(key.clone(), value.clone());
        });
        result
    }

    fn merge_thing_hierarchy<'id>(
        base: ThingHierarchy<'static>,
        overlay: &ThingHierarchy<'id>,
    ) -> ThingHierarchy<'id> {
        // For thing_hierarchy, overlay completely replaces base for matching top-level
        // keys.
        //
        // Base keys not in overlay are preserved
        let mut result = base;
        overlay.iter().for_each(|(key, value)| {
            result.insert(key.clone(), value.clone());
        });
        result
    }

    fn merge_thing_dependencies<'id>(
        base: ThingDependencies<'static>,
        overlay: &ThingDependencies<'id>,
    ) -> ThingDependencies<'id> {
        let mut result = base;
        overlay.iter().for_each(|(key, value)| {
            result.insert(key.clone(), value.clone());
        });
        result
    }

    fn merge_thing_interactions<'id>(
        base: ThingInteractions<'static>,
        overlay: &ThingInteractions<'id>,
    ) -> ThingInteractions<'id> {
        let mut result = base;
        overlay.iter().for_each(|(key, value)| {
            result.insert(key.clone(), value.clone());
        });
        result
    }

    fn merge_processes<'id>(base: Processes<'static>, overlay: &Processes<'id>) -> Processes<'id> {
        let mut result = base;
        overlay.iter().for_each(|(key, value)| {
            result.insert(key.clone(), value.clone());
        });
        result
    }

    fn merge_tag_names<'id>(base: TagNames<'static>, overlay: &TagNames<'id>) -> TagNames<'id> {
        let mut result = base;
        overlay.iter().for_each(|(key, value)| {
            result.insert(key.clone(), value.clone());
        });
        result
    }

    fn merge_tag_things<'id>(base: TagThings<'static>, overlay: &TagThings<'id>) -> TagThings<'id> {
        let mut result = base;
        overlay.iter().for_each(|(key, value)| {
            result.insert(key.clone(), value.clone());
        });
        result
    }

    fn merge_entity_descs<'id>(
        base: EntityDescs<'static>,
        overlay: &EntityDescs<'id>,
    ) -> EntityDescs<'id> {
        let mut result = base;
        overlay.iter().for_each(|(key, value)| {
            result.insert(key.clone(), value.clone());
        });
        result
    }

    fn merge_entity_tooltips<'id>(
        base: EntityTooltips<'static>,
        overlay: &EntityTooltips<'id>,
    ) -> EntityTooltips<'id> {
        let mut result = base;
        overlay.iter().for_each(|(key, value)| {
            result.insert(key.clone(), value.clone());
        });
        result
    }

    fn merge_entity_types<'id>(
        base: EntityTypes<'static>,
        overlay: &EntityTypes<'id>,
    ) -> EntityTypes<'id> {
        let mut result = base;
        overlay.iter().for_each(|(key, value)| {
            result.insert(key.clone(), value.clone());
        });
        result
    }

    fn merge_theme_default<'id>(
        base: ThemeDefault<'static>,
        overlay: &ThemeDefault<'id>,
    ) -> ThemeDefault<'id> {
        let style_aliases = Self::merge_style_aliases(base.style_aliases, &overlay.style_aliases);
        let base_styles = Self::merge_theme_styles(base.base_styles, &overlay.base_styles);
        let process_step_selected_styles = Self::merge_theme_styles(
            base.process_step_selected_styles,
            &overlay.process_step_selected_styles,
        );

        ThemeDefault {
            style_aliases,
            base_styles,
            process_step_selected_styles,
        }
    }

    fn merge_style_aliases<'id>(
        base: StyleAliases<'static>,
        overlay: &StyleAliases<'id>,
    ) -> StyleAliases<'id> {
        let mut result = base;
        overlay.iter().for_each(|(key, value)| {
            result.insert(key.clone(), value.clone());
        });
        result
    }

    fn merge_theme_styles<'id>(
        base: ThemeStyles<'static>,
        overlay: &ThemeStyles<'id>,
    ) -> ThemeStyles<'id> {
        let mut result = base;
        overlay.iter().for_each(|(key, value)| {
            result.insert(key.clone(), value.clone());
        });
        result
    }

    fn merge_theme_types_styles<'id>(
        base: ThemeTypesStyles<'static>,
        overlay: &ThemeTypesStyles<'id>,
    ) -> ThemeTypesStyles<'id> {
        let mut result = base;
        overlay.iter().for_each(|(key, value)| {
            result.insert(key.clone(), value.clone());
        });
        result
    }

    fn merge_theme_thing_dependencies_styles<'id>(
        base: ThemeThingDependenciesStyles<'static>,
        overlay: &ThemeThingDependenciesStyles<'id>,
    ) -> ThemeThingDependenciesStyles<'id> {
        let things_included_styles =
            Self::merge_theme_styles(base.things_included_styles, &overlay.things_included_styles);
        let things_excluded_styles =
            Self::merge_theme_styles(base.things_excluded_styles, &overlay.things_excluded_styles);

        ThemeThingDependenciesStyles {
            things_included_styles,
            things_excluded_styles,
        }
    }

    fn merge_theme_tag_things_focus<'id>(
        base: ThemeTagThingsFocus<'static>,
        overlay: &ThemeTagThingsFocus<'id>,
    ) -> ThemeTagThingsFocus<'id> {
        let mut result = base;
        overlay.iter().for_each(|(key, value)| {
            result.insert(key.clone(), value.clone());
        });
        result
    }

    fn merge_css(base: Css, overlay: &Css) -> Css {
        // If overlay has CSS, use it; otherwise use base
        if overlay.is_empty() {
            base
        } else {
            overlay.clone()
        }
    }
}
