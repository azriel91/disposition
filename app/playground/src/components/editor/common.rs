//! Common helper functions shared across editor page modules.
//!
//! These utilities are used by multiple editor pages to parse string inputs
//! into typed IDs and to rename keys inside theme style maps.

use disposition::{
    input_model::{
        process::{ProcessId, ProcessStepId},
        tag::TagId,
        theme::{IdOrDefaults, ThemeStyles},
        thing::ThingId,
    },
    model_common::{edge::EdgeGroupId, Id},
};

// ---------------------------------------------------------------------------
// ID parsers
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Theme style rename helper
// ---------------------------------------------------------------------------

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
