//! Categories of dynamic map-key completions, mapped from schema def names.

/// A category of map *key* that a container expects, derived from the schema
/// type of the container (its `$ref` def name).
///
/// Unlike [`IdCategory`], which describes the ID type in a *value* position,
/// this describes what to offer when the cursor is typing a *key* inside one of
/// the `InputDiagram` maps whose keys are dynamic IDs (e.g. a `ThingNames` map
/// keyed by `ThingId`). The JSON schema models these as plain
/// `additionalProperties` and drops the key type, so the only signal is the
/// container's def name.
///
/// [`IdCategory`]: crate::completion::id_category::IdCategory
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyCategory {
    /// Keys are `ThingId`s -- `thing_names`, `thing_copy_text`, `thing_descs`,
    /// `thing_layouts`. Offers the things defined in the document.
    ThingId,
    /// Keys are dependency `EdgeGroupId`s -- `thing_dependencies`. Offers an
    /// `edge_dep__<thing_id_0>_<thing_id_1>` template.
    EdgeGroupDep,
    /// Keys are interaction `EdgeGroupId`s -- `thing_interactions`. Offers an
    /// `edge_ix__<thing_id_0>_<thing_id_1>` template.
    EdgeGroupInteraction,
    /// Keys are new `TagId`s -- `tags`. Offers a `tag_example` placeholder.
    TagName,
    /// Keys are existing `TagId`s -- `tag_things`. Offers the tags defined in
    /// the document.
    TagId,
    /// Keys are edge IDs -- `edge_descs`, `edge_labels`. Offers
    /// `<edge_group_id>__0` for each edge group defined in the document.
    EdgeId,
    /// Keys are entity IDs -- `entity_tooltips`, `entity_types`. Offers thing,
    /// process, process-step, and edge IDs defined in the document.
    Entity,
    /// Keys are `StyleAlias`es -- `style_aliases`. Offers the built-in style
    /// aliases plus a `style_alias_custom` placeholder.
    StyleAlias,
    /// Keys are `IdOrDefaults` -- any `ThemeStyles` map. Offers
    /// `node_defaults`, `edge_defaults`, plus thing, edge-group, and edge
    /// IDs.
    ThemeStyles,
    /// Keys are `TagIdOrDefaults` -- `theme_tag_things_focus`. Offers
    /// `tag_defaults` plus the tags defined in the document.
    TagFocus,
    /// Keys are `EntityType`s -- `theme_types_styles`. Offers the built-in
    /// entity types.
    EntityType,
}

impl KeyCategory {
    /// Maps a schema `$defs` name to the key category it constrains, if any.
    pub fn from_ref_name(ref_name: &str) -> Option<KeyCategory> {
        match ref_name {
            "ThingNames" | "ThingCopyText" | "ThingDescs" | "ThingLayouts" => {
                Some(KeyCategory::ThingId)
            }
            "ThingDependencies" => Some(KeyCategory::EdgeGroupDep),
            "ThingInteractions" => Some(KeyCategory::EdgeGroupInteraction),
            "TagNames" => Some(KeyCategory::TagName),
            "TagThings" => Some(KeyCategory::TagId),
            "EdgeDescs" | "EdgeLabels" => Some(KeyCategory::EdgeId),
            "EntityTooltips" | "EntityTypes" => Some(KeyCategory::Entity),
            "StyleAliases" => Some(KeyCategory::StyleAlias),
            "ThemeStyles" => Some(KeyCategory::ThemeStyles),
            "ThemeTagThingsFocus" => Some(KeyCategory::TagFocus),
            "ThemeTypesStyles" => Some(KeyCategory::EntityType),
            _ => None,
        }
    }
}
