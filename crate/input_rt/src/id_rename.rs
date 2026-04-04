//! Shared rename-across-diagram helpers.
//!
//! Many editor pages need to propagate a rename of an entity-level ID through
//! `entity_descs`, `entity_tooltips`, `entity_types`, and all theme style
//! maps. The functions in this module perform that common boilerplate so that
//! each page's rename function only needs to handle the domain-specific maps
//! (e.g. `things`, `tags`, `processes`, etc.) and then call
//! [`id_rename_in_input_diagram`] for the shared fields.

use disposition_input_model::{
    theme::{IdOrDefaults, ThemeStyles},
    InputDiagram,
};
use disposition_model_common::Id;

// === Theme style rename helper === //

/// Replaces an [`IdOrDefaults::Id`] key that matches `id_old` with `id_new`
/// inside a [`ThemeStyles`] map.
///
/// If the old key is not present the map is left unchanged.
pub fn rename_id_in_theme_styles(
    theme_styles: &mut ThemeStyles<'static>,
    id_old: &Id<'static>,
    id_new: &Id<'static>,
) {
    let key_old = IdOrDefaults::Id(id_old.clone());
    if let Some(index) = theme_styles.get_index_of(&key_old) {
        let key_new = IdOrDefaults::Id(id_new.clone());
        let _result = theme_styles.replace_index(index, key_new);
    }
}

// === Shared rename-across-diagram helper === //

/// Renames an [`Id`] across all entity and theme maps in an
/// [`InputDiagram`].
///
/// Specifically this renames the key in:
///
/// * `entity_descs`
/// * `entity_tooltips`
/// * `entity_types`
/// * `theme_default.base_styles`
/// * `theme_default.process_step_selected_styles`
/// * every value in `theme_types_styles`
/// * `theme_thing_dependencies_styles.things_included_styles`
/// * `theme_thing_dependencies_styles.things_excluded_styles`
/// * every value in `theme_tag_things_focus`
///
/// Domain-specific maps (`things`, `tags`, `processes`, etc.) are NOT
/// handled here -- each caller is responsible for those.
///
/// # Parameters
///
/// * `input_diagram`: mutable reference to the [`InputDiagram`] being edited.
/// * `id_old`: the old [`Id`] being replaced.
/// * `id_new`: the new [`Id`] to insert in its place.
pub fn id_rename_in_input_diagram(
    input_diagram: &mut InputDiagram<'static>,
    id_old: &Id<'static>,
    id_new: &Id<'static>,
) {
    let InputDiagram {
        things: _,
        thing_copy_text: _,
        thing_hierarchy: _,
        thing_layouts: _,
        thing_dependencies: _,
        thing_interactions: _,
        processes: _,
        tags: _,
        tag_things: _,
        entity_descs,
        entity_tooltips,
        entity_types,
        theme_default,
        theme_types_styles,
        theme_thing_dependencies_styles,
        theme_tag_things_focus,
        rank_dir: _,
        css: _,
    } = input_diagram;

    // entity_descs / entity_tooltips / entity_types: keys are Id.
    if let Some(index) = entity_descs.get_index_of(id_old) {
        let _result = entity_descs.replace_index(index, id_new.clone());
    }
    if let Some(index) = entity_tooltips.get_index_of(id_old) {
        let _result = entity_tooltips.replace_index(index, id_new.clone());
    }
    if let Some(index) = entity_types.get_index_of(id_old) {
        let _result = entity_types.replace_index(index, id_new.clone());
    }

    // theme_default: rename in base_styles and process_step_selected_styles.
    rename_id_in_theme_styles(&mut theme_default.base_styles, id_old, id_new);
    rename_id_in_theme_styles(
        &mut theme_default.process_step_selected_styles,
        id_old,
        id_new,
    );

    // theme_types_styles: rename in each ThemeStyles value.
    theme_types_styles.values_mut().for_each(|theme_styles| {
        rename_id_in_theme_styles(theme_styles, id_old, id_new);
    });

    // theme_thing_dependencies_styles: rename in both ThemeStyles fields.
    rename_id_in_theme_styles(
        &mut theme_thing_dependencies_styles.things_included_styles,
        id_old,
        id_new,
    );
    rename_id_in_theme_styles(
        &mut theme_thing_dependencies_styles.things_excluded_styles,
        id_old,
        id_new,
    );

    // theme_tag_things_focus: rename in each ThemeStyles value.
    theme_tag_things_focus
        .values_mut()
        .for_each(|theme_styles| {
            rename_id_in_theme_styles(theme_styles, id_old, id_new);
        });
}
