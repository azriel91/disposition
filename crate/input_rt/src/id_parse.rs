//! Strongly-typed ID parsers.
//!
//! Each function attempts to construct a domain-specific ID from a plain
//! string, returning `None` when the string is not a valid identifier.
//! These are extracted from the playground editor so that mutation helpers
//! in this crate can validate user input without depending on dioxus.

use disposition_input_model::{
    process::{ProcessId, ProcessStepId},
    tag::TagId,
    theme::{IdOrDefaults, StyleAlias, TagIdOrDefaults},
    thing::ThingId,
};
use disposition_model_common::{edge::EdgeGroupId, entity::EntityTypeId, Id};

/// Try to construct an `Id<'static>` from a string, returning `None` if the
/// string is not a valid identifier.
///
/// Valid values: `"entity_0"`, `"my_thing"`.
pub fn parse_id(s: &str) -> Option<Id<'static>> {
    Id::new(s).ok().map(|id| id.into_static())
}

/// Try to construct a `ThingId<'static>` from a string, returning `None` if
/// the string is not a valid identifier.
///
/// Valid values: `"t_aws"`, `"thing_0"`.
pub fn parse_thing_id(s: &str) -> Option<ThingId<'static>> {
    Id::new(s).ok().map(|id| ThingId::from(id.into_static()))
}

/// Try to construct an `EdgeGroupId<'static>` from a string, returning `None`
/// if the string is not a valid identifier.
///
/// Valid values: `"edge_0"`, `"dep_network"`.
pub fn parse_edge_group_id(s: &str) -> Option<EdgeGroupId<'static>> {
    EdgeGroupId::new(s).ok().map(|id| id.into_static())
}

/// Try to construct a `TagId<'static>` from a string, returning `None` if the
/// string is not a valid identifier.
///
/// Valid values: `"tag_0"`, `"tag_app_development"`.
pub fn parse_tag_id(s: &str) -> Option<TagId<'static>> {
    Id::new(s).ok().map(|id| TagId::from(id.into_static()))
}

/// Try to construct a `ProcessId<'static>` from a string, returning `None` if
/// the string is not a valid identifier.
///
/// Valid values: `"proc_0"`, `"proc_app_dev"`.
pub fn parse_process_id(s: &str) -> Option<ProcessId<'static>> {
    Id::new(s).ok().map(|id| ProcessId::from(id.into_static()))
}

/// Try to construct a `ProcessStepId<'static>` from a string, returning `None`
/// if the string is not a valid identifier.
///
/// Valid values: `"proc_app_dev_step_0"`, `"step_build"`.
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

/// Parse a string into an `IdOrDefaults<'static>`.
///
/// Returns built-in variants for the recognized sentinel strings, otherwise
/// attempts to parse as a custom entity `Id`.
///
/// Recognized built-in keys:
///
/// * `"node_defaults"`: `IdOrDefaults::NodeDefaults`
/// * `"node_excluded_defaults"`: `IdOrDefaults::NodeExcludedDefaults`
/// * `"edge_defaults"`: `IdOrDefaults::EdgeDefaults`
///
/// Valid values: `"node_defaults"`, `"edge_defaults"`, `"t_aws"`.
pub fn parse_id_or_defaults(s: &str) -> Option<IdOrDefaults<'static>> {
    match s {
        "node_defaults" => Some(IdOrDefaults::NodeDefaults),
        "node_excluded_defaults" => Some(IdOrDefaults::NodeExcludedDefaults),
        "edge_defaults" => Some(IdOrDefaults::EdgeDefaults),
        other => Id::new(other)
            .ok()
            .map(|id| IdOrDefaults::Id(id.into_static())),
    }
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

/// Try to construct a `StyleAlias<'static>` from a string, returning
/// `None` if the string is not a valid identifier.
///
/// Valid values: `"shade_light"`, `"padding_normal"`, `"my_custom_alias"`.
pub fn parse_style_alias(s: &str) -> Option<StyleAlias<'static>> {
    Id::new(s)
        .ok()
        .map(|id| StyleAlias::from(id.into_static()).into_static())
}
