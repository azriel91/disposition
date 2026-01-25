use disposition_model_common::{entity::EntityTooltips, theme::Css};
use serde::{Deserialize, Serialize};

use crate::{
    entity::{EntityDescs, EntityTypes},
    process::Processes,
    tag::{TagNames, TagThings},
    theme::{ThemeDefault, ThemeTagThingsFocus, ThemeThingDependenciesStyles, ThemeTypesStyles},
    thing::{ThingCopyText, ThingDependencies, ThingHierarchy, ThingInteractions, ThingNames},
};

/// The kinds of diagrams that can be generated.
///
/// This is the root data structure for diagram input, containing all
/// configuration for things, their relationships, processes, tags, styling,
/// and themes.
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(bound(deserialize = "\
    ThemeDefault<'id>: Deserialize<'de>, \
    ThemeTypesStyles<'id>: Deserialize<'de>, \
    ThemeThingDependenciesStyles<'id>: Deserialize<'de>, \
    ThemeTagThingsFocus<'id>: Deserialize<'de>\
"))]
pub struct InputDiagram<'id> {
    /// Things in the diagram and their display labels.
    #[serde(default, skip_serializing_if = "ThingNames::is_empty")]
    pub things: ThingNames<'id>,

    /// Text to copy to clipboard when a thing's copy button is clicked.
    ///
    /// This allows things to have different copy text than their display label.
    #[serde(default, skip_serializing_if = "ThingCopyText::is_empty")]
    pub thing_copy_text: ThingCopyText<'id>,

    /// Hierarchy of `thing`s as a recursive tree structure.
    ///
    /// This defines the nesting of things, which affects visual containment
    /// in the diagram.
    #[serde(default, skip_serializing_if = "ThingHierarchy::is_empty")]
    pub thing_hierarchy: ThingHierarchy<'id>,

    /// Dependencies between things (static relationships).
    ///
    /// When B depends on A, it means A must exist before B.
    /// Changes to A means B is out of date.
    #[serde(default, skip_serializing_if = "ThingDependencies::is_empty")]
    pub thing_dependencies: ThingDependencies<'id>,

    /// Interactions between things (communication between applications).
    ///
    /// Has the same structure as dependencies but represents runtime
    /// communication rather than static dependencies.
    #[serde(default, skip_serializing_if = "ThingInteractions::is_empty")]
    pub thing_interactions: ThingInteractions<'id>,

    /// Processes are groupings of interactions between things sequenced over
    /// time.
    #[serde(default, skip_serializing_if = "Processes::is_empty")]
    pub processes: Processes<'id>,

    /// Tags are labels that can be associated with things, so that the things
    /// can be highlighted when the tag is focused.
    #[serde(default, skip_serializing_if = "TagNames::is_empty")]
    pub tags: TagNames<'id>,

    /// Things associated with each tag.
    #[serde(default, skip_serializing_if = "TagThings::is_empty")]
    pub tag_things: TagThings<'id>,

    /// Descriptions to render next to entities (things, edges, edge groups).
    #[serde(default, skip_serializing_if = "EntityDescs::is_empty")]
    pub entity_descs: EntityDescs<'id>,

    /// Descriptions for entities (nodes, edges, and edge groups).
    ///
    /// Contains text (typically markdown) that provides additional context
    /// about entities in the diagram, such as process steps.
    #[serde(default, skip_serializing_if = "EntityTooltips::is_empty")]
    pub entity_tooltips: EntityTooltips<'id>,

    /// Additional `type`s attached to entities for common styling.
    ///
    /// Each entity can have multiple types, allowing styles to be stacked.
    /// These types are appended to the entity's computed default type.
    #[serde(default, skip_serializing_if = "EntityTypes::is_empty")]
    pub entity_types: EntityTypes<'id>,

    /// Default theme styles when the diagram has no user interaction.
    #[serde(default, skip_serializing_if = "ThemeDefault::is_empty")]
    pub theme_default: ThemeDefault<'id>,

    /// Styles applied to things / edges of a particular `type`.
    #[serde(default, skip_serializing_if = "ThemeTypesStyles::is_empty")]
    pub theme_types_styles: ThemeTypesStyles<'id>,

    /// Styles when a `thing` is focused to show its dependencies.
    #[serde(
        default,
        skip_serializing_if = "ThemeThingDependenciesStyles::is_empty"
    )]
    pub theme_thing_dependencies_styles: ThemeThingDependenciesStyles<'id>,

    /// Styles when a tag is focused.
    ///
    /// The `tag_defaults` key applies styles to all tags uniformly.
    /// Specific tag IDs can be used to override defaults for particular tags.
    #[serde(default, skip_serializing_if = "ThemeTagThingsFocus::is_empty")]
    pub theme_tag_things_focus: ThemeTagThingsFocus<'id>,

    /// Additional CSS to place in the SVG's inline `<styles>` section.
    #[serde(default, skip_serializing_if = "Css::is_empty")]
    pub css: Css,
}

impl<'id> InputDiagram<'id> {
    /// Returns a new `InputDiagram` with default values.
    pub fn new() -> Self {
        Self::default()
    }
}
