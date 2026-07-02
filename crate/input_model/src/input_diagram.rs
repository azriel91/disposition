use disposition_model_common::{
    entity::{EntityTooltips, EntityType},
    theme::Css,
    Map, RenderOptions,
};
use serde::{Deserialize, Serialize};

use crate::{
    edge::{EdgeDescs, EdgeLabels},
    entity::EntityTypes,
    process::Processes,
    tag::{TagNames, TagThings},
    theme::{
        CssClassPartials, DarkModeConfig, IdOrDefaults, StyleAlias, StyleAliases, TagIdOrDefaults,
        ThemeAttr, ThemeDefault, ThemeStyles, ThemeTagThingsFocus, ThemeThingDependenciesStyles,
        ThemeTypesStyles,
    },
    thing::{
        ThingCopyText, ThingDependencies, ThingDescs, ThingHierarchy, ThingInteractions,
        ThingLayouts, ThingNames,
    },
};

/// The root data structure for diagram input.
///
/// An `InputDiagram` describes *what* to draw and *how* to style it. The fields
/// fall into a few groups that work together:
///
/// ## How the pieces fit together
///
/// * **Things and hierarchy** -- `things` is the single source of truth for
///   which nodes exist, as a recursive nesting tree. Labels, descriptions, and
///   layout overrides are keyed by `ThingId` in `thing_names`, `thing_descs`,
///   and `thing_layouts`.
///
/// * **Edges and edge groups** -- relationships are declared as *edge groups*
///   in `thing_dependencies` (static "depends on") and `thing_interactions`
///   (runtime communication). Each group has a `kind` (`sequence`, `symmetric`,
///   `cyclic`) and a list of `things`; individual edges within a group get an
///   ID of `<edge_group_id>__<index>`.
///
/// * **Entity types (shared styling)** -- `entity_types` attaches one or more
///   reusable `type_*` ids to *any* entity, **both things and edge groups**.
///   The look of each type is then defined once in `theme_types_styles`, so a
///   whole category of nodes and edges can be styled together. Every entity
///   also carries a built-in default type (e.g. `type_thing_default`) that
///   `theme_types_styles` can target.
///
/// * **Tags and focus** -- `tags` names labels and `tag_things` lists the
///   things in each tag. When a tag is focused in the viewer, the things it
///   contains are highlighted and the rest are dimmed. The two sides are styled
///   separately in `theme_tag_things_focus`: `node_defaults` styles the
///   *included* things, and `node_excluded_defaults` styles the *excluded*
///   ones. (Tags currently hold things only; edges follow their endpoint
///   things.)
///
/// * **Themes** -- `theme_default` holds the base look plus reusable
///   `style_aliases`. The type-, dependency-, and tag-focus theme maps layer on
///   top of it. `render_options` and `css` tune rendering and inject raw CSS.
///
/// Most fields are styling-oriented maps keyed by `ThingId`, `TagId`,
/// `EntityTypeId`, or `EdgeGroupId`; see each field for details and examples.
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(bound(deserialize = "\
    ThemeDefault<'id>: Deserialize<'de>, \
    ThemeTypesStyles<'id>: Deserialize<'de>, \
    ThemeThingDependenciesStyles<'id>: Deserialize<'de>, \
    ThemeTagThingsFocus<'id>: Deserialize<'de>\
"))]
pub struct InputDiagram<'id> {
    /// Things in the diagram, as a recursive hierarchy / nesting tree.
    ///
    /// This is the single source of truth for which things exist in the
    /// diagram: a `thing` is rendered as a node when its `ThingId` appears
    /// here. The nesting also affects visual containment in the diagram.
    ///
    /// Display labels are looked up separately in `thing_names`, defaulting
    /// to the `ThingId` when no entry exists.
    #[serde(default, skip_serializing_if = "ThingHierarchy::is_empty")]
    pub things: ThingHierarchy<'id>,

    /// Display labels for things, keyed by `ThingId`.
    ///
    /// Entries are optional: a thing without an entry here uses its `ThingId`
    /// as its display label.
    #[serde(default, skip_serializing_if = "ThingNames::is_empty")]
    pub thing_names: ThingNames<'id>,

    /// Text to copy to clipboard when a thing's copy button is clicked.
    ///
    /// This allows things to have different copy text than their display label.
    #[serde(default, skip_serializing_if = "ThingCopyText::is_empty")]
    pub thing_copy_text: ThingCopyText<'id>,

    /// User-specified flex-direction overrides for container things.
    ///
    /// When a thing has children in `things`, the layout engine
    /// arranges them in alternating row/column directions by default. Entries
    /// here override that default for the specified thing.
    ///
    /// Valid values: `"row"`, `"row_reverse"`, `"column"`, `"column_reverse"`.
    #[serde(default, skip_serializing_if = "ThingLayouts::is_empty")]
    pub thing_layouts: ThingLayouts<'id>,

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

    /// Descriptions to render next to things in the diagram.
    #[serde(default, skip_serializing_if = "ThingDescs::is_empty")]
    pub thing_descs: ThingDescs<'id>,

    /// Processes are groupings of interactions between things sequenced over
    /// time.
    #[serde(default, skip_serializing_if = "Processes::is_empty")]
    pub processes: Processes<'id>,

    /// Tags are labels that can be associated with things, so that the things
    /// can be highlighted when the tag is focused.
    ///
    /// Maps each `TagId` to its display label. The things in each tag are
    /// listed separately in `tag_things`, and the focus styling is defined
    /// in `theme_tag_things_focus`.
    #[serde(default, skip_serializing_if = "TagNames::is_empty")]
    pub tags: TagNames<'id>,

    /// Things associated with each tag, keyed by `TagId`.
    ///
    /// When a tag is focused, the things listed here are highlighted (styled by
    /// `theme_tag_things_focus`'s `node_defaults`) and the remaining things are
    /// dimmed (styled by `node_excluded_defaults`). Edges are highlighted /
    /// dimmed based on whether their endpoint things are in the focused tag.
    ///
    /// Tags currently hold *things* only -- adding edge groups to a tag
    /// directly is not yet supported.
    #[serde(default, skip_serializing_if = "TagThings::is_empty")]
    pub tag_things: TagThings<'id>,

    /// Descriptions to render next to edges and edge groups.
    #[serde(default, skip_serializing_if = "EdgeDescs::is_empty")]
    pub edge_descs: EdgeDescs<'id>,

    /// Text labels for edges at each endpoint.
    ///
    /// Each entry maps an edge instance ID to its `from` and `to` endpoint
    /// labels. Both labels may be set independently, allowing the source and
    /// destination context to be described with different text.
    #[serde(default, skip_serializing_if = "EdgeLabels::is_empty")]
    pub edge_labels: EdgeLabels<'id>,

    /// Tooltips for entities (nodes, edges, and edge groups).
    ///
    /// Contains plain text that provides additional context about entities in
    /// the diagram, such as process steps.
    #[serde(default, skip_serializing_if = "EntityTooltips::is_empty")]
    pub entity_tooltips: EntityTooltips<'id>,

    /// Additional `type`s attached to entities for common styling.
    ///
    /// Keyed by entity ID, so types can be attached to *any* entity -- **both
    /// things and edge groups** (as well as processes and process steps). Each
    /// entity can have multiple types, allowing styles to be stacked, and these
    /// types are appended to the entity's computed default type.
    ///
    /// The look of each type is defined once in `theme_types_styles`, letting a
    /// whole category of nodes and edges be styled together.
    #[serde(default, skip_serializing_if = "EntityTypes::is_empty")]
    pub entity_types: EntityTypes<'id>,

    /// Default theme styles when the diagram has no user interaction.
    #[serde(default, skip_serializing_if = "ThemeDefault::is_empty")]
    pub theme_default: ThemeDefault<'id>,

    /// Styles applied to things / edges of a particular `type`.
    ///
    /// Keyed by `EntityTypeId`. The keys are the same `type_*` ids that are
    /// attached to entities in `entity_types`, plus the built-in default types
    /// (e.g. `type_thing_default`, `type_dependency_edge_sequence_default`).
    /// Because one entity may carry several types, their styles stack, with
    /// later types overriding earlier ones.
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
    /// Keyed by `tag_defaults` (applied to all tags uniformly) or a specific
    /// `TagId` (overrides the defaults for that tag). For each, `node_defaults`
    /// styles the things that *are* in the focused tag (see `tag_things`), and
    /// `node_excluded_defaults` styles the things that are *not* -- e.g.
    /// dimming them via `opacity`.
    #[serde(default, skip_serializing_if = "ThemeTagThingsFocus::is_empty")]
    pub theme_tag_things_focus: ThemeTagThingsFocus<'id>,

    /// Options that control how the diagram is rendered.
    ///
    /// Includes edge curvature and rank direction settings.
    #[serde(default, skip_serializing_if = "RenderOptions::is_default")]
    pub render_options: RenderOptions,

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
            dark_mode_config: DarkModeConfig::default(),
        };

        let theme_types_styles = base_theme_types_styles();
        let theme_tag_things_focus = base_theme_tag_things_focus();

        let css = Css::from_string(String::from(
            "@keyframes stroke-dashoffset-move {\n  \
               0%   { stroke-dasharray: 3; stroke-dashoffset: 30; }\n  \
               100% { stroke-dasharray: 3; stroke-dashoffset: 0; }\n\
             }",
        ));

        Self {
            things: ThingHierarchy::default(),
            thing_names: ThingNames::default(),
            thing_copy_text: ThingCopyText::default(),
            thing_layouts: ThingLayouts::default(),
            thing_dependencies: ThingDependencies::default(),
            thing_interactions: ThingInteractions::default(),
            thing_descs: ThingDescs::default(),
            processes: Processes::default(),
            tags: TagNames::default(),
            tag_things: TagThings::default(),
            edge_descs: EdgeDescs::default(),
            edge_labels: EdgeLabels::default(),
            entity_tooltips: EntityTooltips::default(),
            entity_types: EntityTypes::default(),
            theme_default,
            theme_types_styles,
            theme_thing_dependencies_styles: ThemeThingDependenciesStyles::default(),
            theme_tag_things_focus,
            render_options: RenderOptions::default(),
            css,
        }
    }
}

fn base_style_aliases() -> StyleAliases<'static> {
    [
        // circle_xs
        (
            StyleAlias::CircleXs,
            css_class_partials(vec![], vec![(ThemeAttr::CircleRadius, "4.0")]),
        ),
        // circle_sm
        (
            StyleAlias::CircleSm,
            css_class_partials(vec![], vec![(ThemeAttr::CircleRadius, "8.0")]),
        ),
        // circle_md
        (
            StyleAlias::CircleMd,
            css_class_partials(vec![], vec![(ThemeAttr::CircleRadius, "12.0")]),
        ),
        // circle_lg
        (
            StyleAlias::CircleLg,
            css_class_partials(vec![], vec![(ThemeAttr::CircleRadius, "16.0")]),
        ),
        // circle_xl
        (
            StyleAlias::CircleXl,
            css_class_partials(vec![], vec![(ThemeAttr::CircleRadius, "24.0")]),
        ),
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
                    (ThemeAttr::FillShadeHover, "500"),
                    (ThemeAttr::FillShadeNormal, "600"),
                    (ThemeAttr::FillShadeFocus, "700"),
                    (ThemeAttr::FillShadeActive, "800"),
                    (ThemeAttr::StrokeShadeHover, "600"),
                    (ThemeAttr::StrokeShadeNormal, "700"),
                    (ThemeAttr::StrokeShadeFocus, "800"),
                    (ThemeAttr::StrokeShadeActive, "900"),
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
        // focus_outline
        (
            StyleAlias::FocusOutline,
            css_class_partials(
                vec![],
                vec![
                    (ThemeAttr::OutlineStyle, "dashed"),
                    (ThemeAttr::OutlineStyleNormal, "none"),
                    (ThemeAttr::OutlineWidth, "2"),
                    (ThemeAttr::OutlineColor, "blue"),
                    (ThemeAttr::OutlineShade, "500"),
                ],
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
                vec![
                    StyleAlias::ShadeLight,
                    StyleAlias::PaddingNormal,
                    StyleAlias::FocusOutline,
                ],
                vec![
                    (ThemeAttr::ShapeColor, "slate"),
                    (ThemeAttr::StrokeStyle, "solid"),
                    (ThemeAttr::StrokeWidth, "2"),
                    (ThemeAttr::TextColor, "neutral"),
                    (ThemeAttr::Visibility, "visible"),
                    (ThemeAttr::Gap, "24.0"),
                ],
            ),
        ),
        // edge_defaults
        (
            IdOrDefaults::EdgeDefaults,
            css_class_partials(
                vec![StyleAlias::FocusOutline],
                vec![(ThemeAttr::TextColor, "neutral")],
            ),
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
                    vec![StyleAlias::RoundedSm, StyleAlias::ShadeLight],
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
                    vec![StyleAlias::RoundedSm, StyleAlias::ShadeLight],
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
                    vec![
                        StyleAlias::CircleMd,
                        StyleAlias::RoundedSm,
                        StyleAlias::ShadeMedium,
                    ],
                    vec![
                        (ThemeAttr::StrokeStyle, "solid"),
                        (ThemeAttr::ShapeColor, "sky"),
                        (ThemeAttr::StrokeWidth, "2"),
                    ],
                ),
            )]
            .into_iter()
            .collect(),
        ),
        // type_dependency_edge_default
        (
            EntityType::DependencyEdgeDefault.into_id().into(),
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
        // type_dependency_edge_sequence_default
        (
            EntityType::DependencyEdgeSequenceDefault.into_id().into(),
            [(
                IdOrDefaults::EdgeDefaults,
                css_class_partials(vec![], vec![]),
            )]
            .into_iter()
            .collect(),
        ),
        // type_dependency_edge_cyclic_default
        (
            EntityType::DependencyEdgeCyclicDefault.into_id().into(),
            [(
                IdOrDefaults::EdgeDefaults,
                css_class_partials(vec![], vec![]),
            )]
            .into_iter()
            .collect(),
        ),
        // type_dependency_edge_symmetric_default
        (
            EntityType::DependencyEdgeSymmetricDefault.into_id().into(),
            [(
                IdOrDefaults::EdgeDefaults,
                css_class_partials(vec![], vec![]),
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
                css_class_partials(vec![], vec![]),
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
                css_class_partials(vec![], vec![]),
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
                css_class_partials(vec![], vec![]),
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
                css_class_partials(vec![], vec![]),
            )]
            .into_iter()
            .collect(),
        ),
        // type_interaction_edge_default
        (
            EntityType::InteractionEdgeDefault.into_id().into(),
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
        // type_interaction_edge_sequence_default
        (
            EntityType::InteractionEdgeSequenceDefault.into_id().into(),
            [(
                IdOrDefaults::EdgeDefaults,
                css_class_partials(vec![], vec![]),
            )]
            .into_iter()
            .collect(),
        ),
        // type_interaction_edge_cyclic_default
        (
            EntityType::InteractionEdgeCyclicDefault.into_id().into(),
            [(
                IdOrDefaults::EdgeDefaults,
                css_class_partials(vec![], vec![]),
            )]
            .into_iter()
            .collect(),
        ),
        // type_interaction_edge_symmetric_default
        (
            EntityType::InteractionEdgeSymmetricDefault.into_id().into(),
            [(
                IdOrDefaults::EdgeDefaults,
                css_class_partials(vec![], vec![]),
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
                css_class_partials(vec![], vec![]),
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
                css_class_partials(vec![], vec![]),
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
                css_class_partials(vec![], vec![]),
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
                css_class_partials(vec![], vec![]),
            )]
            .into_iter()
            .collect(),
        ),
        // type_interaction_edge_halo
        (
            EntityType::InteractionEdgeHalo.into_id().into(),
            [(
                IdOrDefaults::EdgeDefaults,
                css_class_partials(
                    vec![],
                    vec![
                        (ThemeAttr::Opacity, "20"),
                        (ThemeAttr::ShapeColor, "slate"),
                        (ThemeAttr::StrokeShade, "800"),
                        (ThemeAttr::StrokeStyle, "solid"),
                        (ThemeAttr::StrokeWidth, "8"),
                    ],
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
