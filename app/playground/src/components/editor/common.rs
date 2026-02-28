//! Common helper functions and shared CSS constants for editor page modules.
//!
//! These utilities are used by multiple editor pages to parse string inputs
//! into typed IDs, to rename keys inside theme style maps, and to perform
//! the boilerplate rename of an [`Id`] across all theme / entity maps in an
//! [`InputDiagram`].

use disposition::{
    input_model::{
        process::{ProcessId, ProcessStepId},
        tag::TagId,
        theme::{IdOrDefaults, TagIdOrDefaults, ThemeStyles},
        thing::ThingId,
        InputDiagram,
    },
    model_common::{edge::EdgeGroupId, entity::EntityTypeId, Id},
};

// === Shared CSS constants === //

/// CSS classes shared by all section headings inside editor pages.
pub const SECTION_HEADING: &str = "text-sm font-bold text-gray-300 mt-4 mb-1";

/// CSS classes for the outer wrapper of a key-value row.
pub const ROW_CLASS: &str = "\
    flex flex-row gap-2 items-center \
    pt-1 \
    pb-1 \
    border-t-1 \
    border-b-1 \
    has-[:active]:opacity-40\
";

/// Row-level flex layout (no border/drag styling).
pub const ROW_CLASS_SIMPLE: &str = "flex flex-row gap-2 items-center";

/// CSS classes for text inputs.
pub const INPUT_CLASS: &str = "\
    flex-1 \
    rounded \
    border \
    border-gray-600 \
    bg-gray-800 \
    text-gray-200 \
    px-2 py-1 \
    text-sm \
    font-mono \
    focus:border-blue-400 \
    focus:outline-none\
";

/// CSS classes for ID inputs (with validation colouring).
pub const ID_INPUT_CLASS: &str = "\
    flex-1 \
    rounded \
    border \
    border-gray-600 \
    bg-gray-800 \
    text-gray-200 \
    px-2 py-1 \
    text-sm \
    font-mono \
    focus:border-blue-400 \
    focus:outline-none \
    invalid:bg-red-950 \
    invalid:border-red-400\
";

/// CSS classes for a select / dropdown.
pub const SELECT_CLASS: &str = "\
    rounded \
    border \
    border-gray-600 \
    bg-gray-800 \
    text-gray-200 \
    px-2 py-1 \
    text-sm \
    focus:border-blue-400 \
    focus:outline-none\
";

/// CSS classes for the small "remove" button.
pub const REMOVE_BTN: &str = "\
    text-red-400 \
    hover:text-red-300 \
    text-xs \
    cursor-pointer \
    px-1\
";

/// CSS classes for the "add" button.
pub const ADD_BTN: &str = "\
    mt-1 \
    text-left \
    text-sm \
    text-blue-400 \
    hover:text-blue-300 \
    cursor-pointer \
    select-none\
";

/// CSS classes for a card-like container.
pub const CARD_CLASS: &str = "\
    rounded-lg \
    border \
    border-gray-700 \
    bg-gray-900 \
    p-3 \
    mb-2 \
    flex \
    flex-col \
    gap-2\
";

/// CSS classes for a nested card (e.g. steps within a process).
pub const INNER_CARD_CLASS: &str = "\
    rounded \
    border \
    border-gray-700 \
    bg-gray-850 \
    p-2 \
    flex \
    flex-col \
    gap-1\
";

/// CSS classes for textareas (theme / YAML editors).
pub const TEXTAREA_CLASS: &str = "\
    w-full \
    min-h-24 \
    rounded \
    border \
    border-gray-600 \
    bg-gray-800 \
    text-gray-200 \
    p-2 \
    font-mono \
    text-sm \
    focus:border-blue-400 \
    focus:outline-none\
";

/// CSS classes for the drag handle grip -- braille dots (`⠿`).
pub const DRAG_HANDLE: &str = "\
    text-gray-600 \
    hover:text-gray-400 \
    cursor-grab \
    active:cursor-grabbing \
    select-none \
    leading-none \
    text-sm \
    px-0.5 \
    flex \
    items-center\
";

/// Helper label classes.
pub const LABEL_CLASS: &str = "text-xs text-gray-500 mb-1";

// === ID parsers === //

/// Try to construct an `Id<'static>` from a string, returning `None` if the
/// string is not a valid identifier.
pub fn parse_id(s: &str) -> Option<Id<'static>> {
    Id::new(s).ok().map(|id| id.into_static())
}

/// Try to construct a `ThingId<'static>` from a string, returning `None` if
/// the string is not a valid identifier.
pub fn parse_thing_id(s: &str) -> Option<ThingId<'static>> {
    Id::new(s).ok().map(|id| ThingId::from(id.into_static()))
}

/// Try to construct an `EdgeGroupId<'static>` from a string, returning `None`
/// if the string is not a valid identifier.
pub fn parse_edge_group_id(s: &str) -> Option<EdgeGroupId<'static>> {
    EdgeGroupId::new(s).ok().map(|id| id.into_static())
}

/// Try to construct a `TagId<'static>` from a string, returning `None` if the
/// string is not a valid identifier.
pub fn parse_tag_id(s: &str) -> Option<TagId<'static>> {
    Id::new(s).ok().map(|id| TagId::from(id.into_static()))
}

/// Try to construct a `ProcessId<'static>` from a string, returning `None` if
/// the string is not a valid identifier.
pub fn parse_process_id(s: &str) -> Option<ProcessId<'static>> {
    Id::new(s).ok().map(|id| ProcessId::from(id.into_static()))
}

/// Try to construct a `ProcessStepId<'static>` from a string, returning `None`
/// if the string is not a valid identifier.
pub fn parse_process_step_id(s: &str) -> Option<ProcessStepId<'static>> {
    Id::new(s)
        .ok()
        .map(|id| ProcessStepId::from(id.into_static()))
}

/// Try to construct an `EntityTypeId<'static>` from a string, returning
/// `None` if the string is not a valid identifier.
///
/// Valid values: `"type_organisation"`, `"type_custom_1"`.
pub fn parse_entity_type_id(s: &str) -> Option<EntityTypeId<'static>> {
    Id::new(s)
        .ok()
        .map(|id| EntityTypeId::from(id.into_static()))
}

/// Parse a string into a `TagIdOrDefaults<'static>`.
///
/// Returns `TagIdOrDefaults::TagDefaults` for the literal `"tag_defaults"`,
/// otherwise attempts to parse as a custom `TagId`.
///
/// Valid values: `"tag_defaults"`, `"tag_app_development"`.
pub fn parse_tag_id_or_defaults(s: &str) -> Option<TagIdOrDefaults<'static>> {
    match s {
        "tag_defaults" => Some(TagIdOrDefaults::TagDefaults),
        other => Id::new(other)
            .ok()
            .map(|id| TagIdOrDefaults::Custom(TagId::from(id.into_static()))),
    }
}

// === Theme style rename helper === //

/// Replaces an [`IdOrDefaults::Id`] key that matches `id_old` with `id_new`
/// inside a [`ThemeStyles`] map.
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
/// Many editor pages need to propagate a rename of an entity-level ID through
/// `entity_descs`, `entity_tooltips`, `entity_types`, and all theme style
/// maps. This function performs that common boilerplate so that each page's
/// rename function only needs to handle the domain-specific maps (e.g.
/// `things`, `tags`, `processes`, etc.) and then call this for the shared
/// fields.
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
