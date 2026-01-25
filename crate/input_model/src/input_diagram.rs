use disposition_model_common::{
    entity::{EntityTooltips, EntityType},
    theme::Css,
    Map,
};
use serde::{Deserialize, Serialize};

use crate::{
    entity::{EntityDescs, EntityTypes},
    process::Processes,
    tag::{TagNames, TagThings},
    theme::{
        CssClassPartials, IdOrDefaults, StyleAlias, StyleAliases, TagIdOrDefaults, ThemeAttr,
        ThemeDefault, ThemeStyles, ThemeTagThingsFocus, ThemeThingDependenciesStyles,
        ThemeTypesStyles,
    },
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

impl InputDiagram<'static> {
    /// Returns the base `InputDiagram` with default theme settings.
    ///
    /// This contains standard style aliases, base styles, type styles, tag
    /// focus styles, and CSS animations as sensible defaults.
    ///
    /// # Maintainers
    ///
    /// A YAML representation of this diagram is available in
    /// `workspace_tests/src/base_diagram.yaml`.
    ///
    /// Make sure to update `base_diagram.yaml` when updating this file.
    #[allow(clippy::too_many_lines)]
    pub fn base() -> Self {
        let style_aliases = base_style_aliases();
        let base_styles = base_theme_styles();
        let process_step_selected_styles = base_process_step_selected_styles();

        let theme_default = ThemeDefault {
            style_aliases,
            base_styles,
            process_step_selected_styles,
        };

        let theme_types_styles = base_theme_types_styles();
        let theme_tag_things_focus = base_theme_tag_things_focus();

        let css = Css::from_string(String::from(
            "@keyframes stroke-dashoffset-move {\n  \
               0%   { stroke-dasharray: 3; stroke-dashoffset: 30; }\n  \
               100% { stroke-dasharray: 3; stroke-dashoffset: 0; }\n\
             }\n\
             @keyframes stroke-dashoffset-move-request {\n  \
               0%   { stroke-dashoffset: 0; }\n  \
               100% { stroke-dashoffset: 228; }\n\
             }\n\
             @keyframes stroke-dashoffset-move-response {\n  \
               0%   { stroke-dashoffset: 0; }\n  \
               100% { stroke-dashoffset: -248; }\n\
             }",
        ));

        Self {
            things: ThingNames::default(),
            thing_copy_text: ThingCopyText::default(),
            thing_hierarchy: ThingHierarchy::default(),
            thing_dependencies: ThingDependencies::default(),
            thing_interactions: ThingInteractions::default(),
            processes: Processes::default(),
            tags: TagNames::default(),
            tag_things: TagThings::default(),
            entity_descs: EntityDescs::default(),
            entity_tooltips: EntityTooltips::default(),
            entity_types: EntityTypes::default(),
            theme_default,
            theme_types_styles,
            theme_thing_dependencies_styles: ThemeThingDependenciesStyles::default(),
            theme_tag_things_focus,
            css,
        }
    }
}

fn base_style_aliases() -> StyleAliases<'static> {
    [
        // padding_none
        (
            StyleAlias::PaddingNone,
            css_class_partials(
                vec![],
                vec![(ThemeAttr::Padding, "0.0"), (ThemeAttr::Gap, "0.0")],
            ),
        ),
        // padding_tight
        (
            StyleAlias::PaddingTight,
            css_class_partials(
                vec![],
                vec![(ThemeAttr::Padding, "2.0"), (ThemeAttr::Gap, "2.0")],
            ),
        ),
        // padding_normal
        (
            StyleAlias::PaddingNormal,
            css_class_partials(
                vec![],
                vec![(ThemeAttr::Padding, "4.0"), (ThemeAttr::Gap, "4.0")],
            ),
        ),
        // padding_wide
        (
            StyleAlias::PaddingWide,
            css_class_partials(
                vec![],
                vec![(ThemeAttr::Padding, "6.0"), (ThemeAttr::Gap, "6.0")],
            ),
        ),
        // rounded_xs
        (
            StyleAlias::RoundedXs,
            css_class_partials(
                vec![],
                vec![
                    (ThemeAttr::RadiusTopLeft, "2.0"),
                    (ThemeAttr::RadiusTopRight, "2.0"),
                    (ThemeAttr::RadiusBottomLeft, "2.0"),
                    (ThemeAttr::RadiusBottomRight, "2.0"),
                ],
            ),
        ),
        // rounded_sm
        (
            StyleAlias::RoundedSm,
            css_class_partials(
                vec![],
                vec![
                    (ThemeAttr::RadiusTopLeft, "4.0"),
                    (ThemeAttr::RadiusTopRight, "4.0"),
                    (ThemeAttr::RadiusBottomLeft, "4.0"),
                    (ThemeAttr::RadiusBottomRight, "4.0"),
                ],
            ),
        ),
        // rounded_md
        (
            StyleAlias::RoundedMd,
            css_class_partials(
                vec![],
                vec![
                    (ThemeAttr::RadiusTopLeft, "6.0"),
                    (ThemeAttr::RadiusTopRight, "6.0"),
                    (ThemeAttr::RadiusBottomLeft, "6.0"),
                    (ThemeAttr::RadiusBottomRight, "6.0"),
                ],
            ),
        ),
        // rounded_lg
        (
            StyleAlias::RoundedLg,
            css_class_partials(
                vec![],
                vec![
                    (ThemeAttr::RadiusTopLeft, "8.0"),
                    (ThemeAttr::RadiusTopRight, "8.0"),
                    (ThemeAttr::RadiusBottomLeft, "8.0"),
                    (ThemeAttr::RadiusBottomRight, "8.0"),
                ],
            ),
        ),
        // rounded_xl
        (
            StyleAlias::RoundedXl,
            css_class_partials(
                vec![],
                vec![
                    (ThemeAttr::RadiusTopLeft, "12.0"),
                    (ThemeAttr::RadiusTopRight, "12.0"),
                    (ThemeAttr::RadiusBottomLeft, "12.0"),
                    (ThemeAttr::RadiusBottomRight, "12.0"),
                ],
            ),
        ),
        // rounded_2xl
        (
            StyleAlias::Rounded2xl,
            css_class_partials(
                vec![],
                vec![
                    (ThemeAttr::RadiusTopLeft, "16.0"),
                    (ThemeAttr::RadiusTopRight, "16.0"),
                    (ThemeAttr::RadiusBottomLeft, "16.0"),
                    (ThemeAttr::RadiusBottomRight, "16.0"),
                ],
            ),
        ),
        // rounded_3xl
        (
            StyleAlias::Rounded3xl,
            css_class_partials(
                vec![],
                vec![
                    (ThemeAttr::RadiusTopLeft, "24.0"),
                    (ThemeAttr::RadiusTopRight, "24.0"),
                    (ThemeAttr::RadiusBottomLeft, "24.0"),
                    (ThemeAttr::RadiusBottomRight, "24.0"),
                ],
            ),
        ),
        // rounded_4xl
        (
            StyleAlias::Rounded4xl,
            css_class_partials(
                vec![],
                vec![
                    (ThemeAttr::RadiusTopLeft, "32.0"),
                    (ThemeAttr::RadiusTopRight, "32.0"),
                    (ThemeAttr::RadiusBottomLeft, "32.0"),
                    (ThemeAttr::RadiusBottomRight, "32.0"),
                ],
            ),
        ),
        // fill_pale
        (
            StyleAlias::FillPale,
            css_class_partials(
                vec![],
                vec![
                    (ThemeAttr::FillShadeHover, "50"),
                    (ThemeAttr::FillShadeNormal, "100"),
                    (ThemeAttr::FillShadeFocus, "200"),
                    (ThemeAttr::FillShadeActive, "300"),
                    (ThemeAttr::TextShade, "800"),
                ],
            ),
        ),
        // shade_pale
        (
            StyleAlias::ShadePale,
            css_class_partials(
                vec![],
                vec![
                    (ThemeAttr::FillShadeHover, "50"),
                    (ThemeAttr::FillShadeNormal, "100"),
                    (ThemeAttr::FillShadeFocus, "200"),
                    (ThemeAttr::FillShadeActive, "300"),
                    (ThemeAttr::StrokeShadeHover, "100"),
                    (ThemeAttr::StrokeShadeNormal, "200"),
                    (ThemeAttr::StrokeShadeFocus, "300"),
                    (ThemeAttr::StrokeShadeActive, "400"),
                    (ThemeAttr::TextShade, "800"),
                ],
            ),
        ),
        // shade_light
        (
            StyleAlias::ShadeLight,
            css_class_partials(
                vec![],
                vec![
                    (ThemeAttr::FillShadeHover, "200"),
                    (ThemeAttr::FillShadeNormal, "300"),
                    (ThemeAttr::FillShadeFocus, "400"),
                    (ThemeAttr::FillShadeActive, "500"),
                    (ThemeAttr::StrokeShadeHover, "300"),
                    (ThemeAttr::StrokeShadeNormal, "400"),
                    (ThemeAttr::StrokeShadeFocus, "500"),
                    (ThemeAttr::StrokeShadeActive, "600"),
                    (ThemeAttr::TextShade, "900"),
                ],
            ),
        ),
        // shade_medium
        (
            StyleAlias::ShadeMedium,
            css_class_partials(
                vec![],
                vec![
                    (ThemeAttr::FillShadeHover, "400"),
                    (ThemeAttr::FillShadeNormal, "500"),
                    (ThemeAttr::FillShadeFocus, "600"),
                    (ThemeAttr::FillShadeActive, "700"),
                    (ThemeAttr::StrokeShadeHover, "500"),
                    (ThemeAttr::StrokeShadeNormal, "600"),
                    (ThemeAttr::StrokeShadeFocus, "700"),
                    (ThemeAttr::StrokeShadeActive, "800"),
                    (ThemeAttr::TextShade, "950"),
                ],
            ),
        ),
        // shade_dark
        (
            StyleAlias::ShadeDark,
            css_class_partials(
                vec![],
                vec![
                    (ThemeAttr::FillShadeHover, "600"),
                    (ThemeAttr::FillShadeNormal, "700"),
                    (ThemeAttr::FillShadeFocus, "800"),
                    (ThemeAttr::FillShadeActive, "900"),
                    (ThemeAttr::StrokeShadeHover, "700"),
                    (ThemeAttr::StrokeShadeNormal, "800"),
                    (ThemeAttr::StrokeShadeFocus, "900"),
                    (ThemeAttr::StrokeShadeActive, "950"),
                    (ThemeAttr::TextShade, "950"),
                ],
            ),
        ),
        // stroke_dashed_animated
        (
            StyleAlias::StrokeDashedAnimated,
            css_class_partials(
                vec![],
                vec![
                    (ThemeAttr::StrokeStyle, "dashed"),
                    (ThemeAttr::StrokeWidth, "2"),
                    (
                        ThemeAttr::Animate,
                        "[stroke-dashoffset-move_2s_linear_infinite]",
                    ),
                ],
            ),
        ),
        // stroke_dashed_animated_request
        (
            StyleAlias::StrokeDashedAnimatedRequest,
            css_class_partials(
                vec![],
                vec![(
                    ThemeAttr::Animate,
                    "[stroke-dashoffset-move-request_2s_linear_infinite]",
                )],
            ),
        ),
        // stroke_dashed_animated_response
        (
            StyleAlias::StrokeDashedAnimatedResponse,
            css_class_partials(
                vec![],
                vec![(
                    ThemeAttr::Animate,
                    "[stroke-dashoffset-move-response_2s_linear_infinite]",
                )],
            ),
        ),
    ]
    .into_iter()
    .collect()
}

fn base_theme_styles() -> ThemeStyles<'static> {
    [
        // node_defaults
        (
            IdOrDefaults::NodeDefaults,
            css_class_partials(
                vec![StyleAlias::ShadeLight, StyleAlias::PaddingNormal],
                vec![
                    (ThemeAttr::ShapeColor, "slate"),
                    (ThemeAttr::StrokeStyle, "solid"),
                    (ThemeAttr::StrokeWidth, "2"),
                    (ThemeAttr::TextColor, "neutral"),
                    (ThemeAttr::Visibility, "visible"),
                ],
            ),
        ),
        // edge_defaults
        (
            IdOrDefaults::EdgeDefaults,
            css_class_partials(vec![], vec![(ThemeAttr::TextColor, "neutral")]),
        ),
    ]
    .into_iter()
    .collect()
}

fn base_process_step_selected_styles() -> ThemeStyles<'static> {
    [
        // node_defaults
        (
            IdOrDefaults::NodeDefaults,
            css_class_partials(
                vec![StyleAlias::FillPale, StyleAlias::StrokeDashedAnimated],
                vec![],
            ),
        ),
        // edge_defaults
        (
            IdOrDefaults::EdgeDefaults,
            css_class_partials(vec![], vec![(ThemeAttr::Visibility, "visible")]),
        ),
    ]
    .into_iter()
    .collect()
}

fn base_theme_tag_things_focus() -> ThemeTagThingsFocus<'static> {
    [
        // tag_defaults
        (
            TagIdOrDefaults::TagDefaults,
            [
                (
                    IdOrDefaults::NodeDefaults,
                    css_class_partials(
                        vec![StyleAlias::FillPale, StyleAlias::StrokeDashedAnimated],
                        vec![],
                    ),
                ),
                (
                    IdOrDefaults::NodeExcludedDefaults,
                    css_class_partials(vec![], vec![(ThemeAttr::Opacity, "75")]),
                ),
            ]
            .into_iter()
            .collect(),
        ),
    ]
    .into_iter()
    .collect()
}

fn base_theme_types_styles() -> ThemeTypesStyles<'static> {
    [
        // type_thing_default
        (
            EntityType::ThingDefault.into_id().into(),
            [(
                IdOrDefaults::NodeDefaults,
                css_class_partials(
                    vec![StyleAlias::RoundedSm, StyleAlias::ShadeLight],
                    vec![
                        (ThemeAttr::StrokeStyle, "solid"),
                        (ThemeAttr::ShapeColor, "slate"),
                        (ThemeAttr::StrokeWidth, "2"),
                    ],
                ),
            )]
            .into_iter()
            .collect(),
        ),
        // type_tag_default
        (
            EntityType::TagDefault.into_id().into(),
            [(
                IdOrDefaults::NodeDefaults,
                css_class_partials(
                    vec![StyleAlias::RoundedSm, StyleAlias::ShadeMedium],
                    vec![
                        (ThemeAttr::StrokeStyle, "solid"),
                        (ThemeAttr::ShapeColor, "emerald"),
                        (ThemeAttr::StrokeWidth, "2"),
                    ],
                ),
            )]
            .into_iter()
            .collect(),
        ),
        // type_process_default
        (
            EntityType::ProcessDefault.into_id().into(),
            [(
                IdOrDefaults::NodeDefaults,
                css_class_partials(
                    vec![StyleAlias::RoundedSm, StyleAlias::ShadeMedium],
                    vec![
                        (ThemeAttr::StrokeStyle, "solid"),
                        (ThemeAttr::ShapeColor, "blue"),
                        (ThemeAttr::StrokeWidth, "2"),
                    ],
                ),
            )]
            .into_iter()
            .collect(),
        ),
        // type_process_step_default
        (
            EntityType::ProcessStepDefault.into_id().into(),
            [(
                IdOrDefaults::NodeDefaults,
                css_class_partials(
                    vec![StyleAlias::RoundedSm, StyleAlias::ShadeMedium],
                    vec![
                        (ThemeAttr::StrokeStyle, "solid"),
                        (ThemeAttr::ShapeColor, "sky"),
                        (ThemeAttr::StrokeWidth, "2"),
                        (ThemeAttr::Visibility, "invisible"),
                    ],
                ),
            )]
            .into_iter()
            .collect(),
        ),
        // type_dependency_edge_sequence_default
        (
            EntityType::DependencyEdgeSequenceDefault.into_id().into(),
            [(
                IdOrDefaults::EdgeDefaults,
                css_class_partials(
                    vec![StyleAlias::ShadeDark],
                    vec![
                        (ThemeAttr::StrokeStyle, "solid"),
                        (ThemeAttr::ShapeColor, "neutral"),
                        (ThemeAttr::StrokeWidth, "2"),
                        (ThemeAttr::Visibility, "visible"),
                    ],
                ),
            )]
            .into_iter()
            .collect(),
        ),
        // type_dependency_edge_cyclic_default
        (
            EntityType::DependencyEdgeCyclicDefault.into_id().into(),
            [(
                IdOrDefaults::EdgeDefaults,
                css_class_partials(
                    vec![StyleAlias::ShadeDark],
                    vec![
                        (ThemeAttr::StrokeStyle, "solid"),
                        (ThemeAttr::ShapeColor, "neutral"),
                        (ThemeAttr::StrokeWidth, "2"),
                        (ThemeAttr::Visibility, "visible"),
                    ],
                ),
            )]
            .into_iter()
            .collect(),
        ),
        // type_dependency_edge_symmetric_default
        (
            EntityType::DependencyEdgeSymmetricDefault.into_id().into(),
            [(
                IdOrDefaults::EdgeDefaults,
                css_class_partials(
                    vec![StyleAlias::ShadeDark],
                    vec![
                        (ThemeAttr::StrokeStyle, "solid"),
                        (ThemeAttr::ShapeColor, "neutral"),
                        (ThemeAttr::StrokeWidth, "2"),
                        (ThemeAttr::Visibility, "visible"),
                    ],
                ),
            )]
            .into_iter()
            .collect(),
        ),
        // type_dependency_edge_sequence_forward_default
        (
            EntityType::DependencyEdgeSequenceForwardDefault
                .into_id()
                .into(),
            [(
                IdOrDefaults::EdgeDefaults,
                css_class_partials(vec![], vec![(ThemeAttr::StrokeWidth, "2")]),
            )]
            .into_iter()
            .collect(),
        ),
        // type_dependency_edge_cyclic_forward_default
        (
            EntityType::DependencyEdgeCyclicForwardDefault
                .into_id()
                .into(),
            [(
                IdOrDefaults::EdgeDefaults,
                css_class_partials(vec![], vec![(ThemeAttr::StrokeWidth, "2")]),
            )]
            .into_iter()
            .collect(),
        ),
        // type_dependency_edge_symmetric_forward_default
        (
            EntityType::DependencyEdgeSymmetricForwardDefault
                .into_id()
                .into(),
            [(
                IdOrDefaults::EdgeDefaults,
                css_class_partials(vec![], vec![(ThemeAttr::StrokeWidth, "2")]),
            )]
            .into_iter()
            .collect(),
        ),
        // type_dependency_edge_symmetric_reverse_default
        (
            EntityType::DependencyEdgeSymmetricReverseDefault
                .into_id()
                .into(),
            [(
                IdOrDefaults::EdgeDefaults,
                css_class_partials(vec![], vec![(ThemeAttr::StrokeWidth, "2")]),
            )]
            .into_iter()
            .collect(),
        ),
        // type_interaction_edge_sequence_default
        (
            EntityType::InteractionEdgeSequenceDefault.into_id().into(),
            [(
                IdOrDefaults::EdgeDefaults,
                css_class_partials(
                    vec![StyleAlias::ShadeDark],
                    vec![
                        (ThemeAttr::ShapeColor, "violet"),
                        (ThemeAttr::StrokeWidth, "2"),
                        (
                            ThemeAttr::StrokeStyle,
                            "dasharray:0,80,12,2,4,2,2,2,1,2,1,120",
                        ),
                        (ThemeAttr::Visibility, "invisible"),
                    ],
                ),
            )]
            .into_iter()
            .collect(),
        ),
        // type_interaction_edge_cyclic_default
        (
            EntityType::InteractionEdgeCyclicDefault.into_id().into(),
            [(
                IdOrDefaults::EdgeDefaults,
                css_class_partials(
                    vec![StyleAlias::ShadeDark],
                    vec![
                        (ThemeAttr::ShapeColor, "violet"),
                        (ThemeAttr::StrokeWidth, "2"),
                        (
                            ThemeAttr::StrokeStyle,
                            "dasharray:0,80,12,2,4,2,2,2,1,2,1,120",
                        ),
                        (ThemeAttr::Visibility, "invisible"),
                    ],
                ),
            )]
            .into_iter()
            .collect(),
        ),
        // type_interaction_edge_symmetric_default
        (
            EntityType::InteractionEdgeSymmetricDefault.into_id().into(),
            [(
                IdOrDefaults::EdgeDefaults,
                css_class_partials(
                    vec![StyleAlias::ShadeDark],
                    vec![
                        (ThemeAttr::ShapeColor, "violet"),
                        (ThemeAttr::StrokeWidth, "2"),
                        (ThemeAttr::Visibility, "invisible"),
                    ],
                ),
            )]
            .into_iter()
            .collect(),
        ),
        // type_interaction_edge_sequence_forward_default
        (
            EntityType::InteractionEdgeSequenceForwardDefault
                .into_id()
                .into(),
            [(
                IdOrDefaults::EdgeDefaults,
                css_class_partials(vec![StyleAlias::StrokeDashedAnimatedRequest], vec![]),
            )]
            .into_iter()
            .collect(),
        ),
        // type_interaction_edge_cyclic_forward_default
        (
            EntityType::InteractionEdgeCyclicForwardDefault
                .into_id()
                .into(),
            [(
                IdOrDefaults::EdgeDefaults,
                css_class_partials(vec![StyleAlias::StrokeDashedAnimatedRequest], vec![]),
            )]
            .into_iter()
            .collect(),
        ),
        // type_interaction_edge_symmetric_forward_default
        (
            EntityType::InteractionEdgeSymmetricForwardDefault
                .into_id()
                .into(),
            [(
                IdOrDefaults::EdgeDefaults,
                css_class_partials(
                    vec![StyleAlias::StrokeDashedAnimatedRequest],
                    vec![(
                        ThemeAttr::StrokeStyle,
                        "dasharray:0,80,12,2,4,2,2,2,1,2,1,120",
                    )],
                ),
            )]
            .into_iter()
            .collect(),
        ),
        // type_interaction_edge_symmetric_reverse_default
        (
            EntityType::InteractionEdgeSymmetricReverseDefault
                .into_id()
                .into(),
            [(
                IdOrDefaults::EdgeDefaults,
                css_class_partials(
                    vec![StyleAlias::StrokeDashedAnimatedResponse],
                    vec![(
                        ThemeAttr::StrokeStyle,
                        "dasharray:0,120,1,2,1,2,2,2,4,2,8,2,20,80",
                    )],
                ),
            )]
            .into_iter()
            .collect(),
        ),
    ]
    .into_iter()
    .collect()
}

/// Creates a `CssClassPartials` with style aliases and partials.
fn css_class_partials(
    style_aliases_applied: Vec<StyleAlias<'static>>,
    partials: Vec<(ThemeAttr, &'static str)>,
) -> CssClassPartials<'static> {
    let partials = partials
        .into_iter()
        .map(|(theme_attr, value)| (theme_attr, String::from(value)))
        .collect::<Map<ThemeAttr, String>>();

    CssClassPartials {
        style_aliases_applied,
        partials,
    }
}
