use disposition_model_common::{entity::EntityTooltips, theme::Css};
use serde::{Deserialize, Serialize};

use crate::{
    edge::EdgeGroups,
    entity::{EntityDescs, EntityTailwindClasses, EntityTypes},
    layout::NodeLayouts,
    node::{NodeCopyText, NodeHierarchy, NodeNames, NodeOrdering},
    shape::NodeShapes,
};

/// The intermediate representation of a diagram.
///
/// This is the computed data structure from combining the layered values from
/// the input data. It contains all the information needed to generate the
/// final SVG output.
///
/// Key differences from the input model:
///
/// * Uses a unified `NodeId` for all things, tags, processes, and steps
/// * `node_hierarchy` includes the flow-layout hierarchy for all node types,
///   not just things.
/// * `edge_groups` contains explicit `from`/`to` edges instead of `EdgeKind`
/// * `tailwind_classes` contains computed CSS classes instead of theme configs
/// * `node_layout` defines the flex layout values for each node.
/// * `node_tooltip` defines the tooltip content for process steps.
///
/// # Example
///
/// ```yaml
/// nodes:
///   t_aws: "☁️ Amazon Web Services"
///   tag_app_development: "Application Development"
///   proc_app_dev: "App Development"
///   proc_app_dev_step_repository_clone: "Clone repository"
///
/// node_hierarchy:
///   tag_app_development: {}
///   proc_app_dev:
///     proc_app_dev_step_repository_clone: {}
///   t_aws:
///     t_aws_iam: {}
///
/// node_ordering:
///   tag_app_development: 10
///   proc_app_dev_step_repository_clone: 3
///   proc_app_dev: 2
///   t_aws: 1
///
/// edge_groups:
///   edge_t_localhost__t_github_user_repo:
///     - from: t_localhost
///       to: t_github_user_repo
///
/// entity_descs:
///   t_localhost: "User's computer"
///   edge_t_localhost__t_github_user_repo__pull: "Fetch from GitHub"
///
/// entity_tooltips:
///   proc_app_release_step_tag_and_push: |-
///     When the PR is merged, tag the commit and push the tag to GitHub.
///
/// tailwind_classes:
///   t_aws: "stroke-1 visible hover:fill-yellow-50 fill-yellow-100"
///
/// node_layout:
///   _root:
///     flex:
///       direction: "column_reverse"
///       wrap: true
///       padding_top: 4.0
///       padding_right: 4.0
///       padding_bottom: 4.0
///       padding_left: 4.0
///       margin_top: 0.0
///       margin_right: 0.0
///       margin_bottom: 0.0
///       margin_left: 0.0
///       gap: 4.0
///
/// css: >-
///   @keyframes stroke-dashoffset-move { ... }
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct IrDiagram<'id> {
    /// All nodes in the diagram and their display labels.
    ///
    /// This includes things, tags, processes, and process steps.
    #[serde(default, skip_serializing_if = "NodeNames::is_empty")]
    pub nodes: NodeNames<'id>,

    /// Text to copy to clipboard when a node's copy button is clicked.
    ///
    /// This allows nodes to have different copy text than their display label.
    /// Typically only includes `thing` nodes.
    #[serde(default, skip_serializing_if = "NodeCopyText::is_empty")]
    pub node_copy_text: NodeCopyText<'id>,

    /// Hierarchy of all nodes as a recursive tree structure.
    ///
    /// This includes tags, processes (with their steps), and things.
    /// The order of declaration is important for CSS peer selector ordering.
    #[serde(default, skip_serializing_if = "NodeHierarchy::is_empty")]
    pub node_hierarchy: NodeHierarchy<'id>,

    /// Order that nodes should appear in the final SVG, and their tab indices.
    ///
    /// The map order defines rendering order (tags, then process steps, then
    /// processes, then things), while the values define the tab indices for
    /// keyboard navigation.
    #[serde(default, skip_serializing_if = "NodeOrdering::is_empty")]
    pub node_ordering: NodeOrdering<'id>,

    /// Edge groups derived from `thing_dependencies` and `thing_interactions`.
    ///
    /// Each edge group contains explicit `from`/`to` edges.
    #[serde(default, skip_serializing_if = "EdgeGroups::is_empty")]
    pub edge_groups: EdgeGroups<'id>,

    /// Descriptions for entities (nodes, edges, and edge groups).
    ///
    /// Contains text (typically markdown) that provides additional context
    /// about entities in the diagram, such as things or edge labels.
    #[serde(default, skip_serializing_if = "EntityDescs::is_empty")]
    pub entity_descs: EntityDescs<'id>,

    /// Descriptions for entities (nodes, edges, and edge groups).
    ///
    /// Contains text (typically markdown) that provides additional context
    /// about entities in the diagram, such as process steps.
    #[serde(default, skip_serializing_if = "EntityTooltips::is_empty")]
    pub entity_tooltips: EntityTooltips<'id>,

    /// Entity types attached to nodes and edges for common styling.
    ///
    /// Each node/edge can have multiple types, allowing styles to be stacked.
    #[serde(default, skip_serializing_if = "EntityTypes::is_empty")]
    pub entity_types: EntityTypes<'id>,

    /// Computed Tailwind CSS classes for interactive visibility behaviour.
    ///
    /// These classes control visibility, colors, animations, and interactions
    /// based on the diagram's state.
    #[serde(default, skip_serializing_if = "EntityTailwindClasses::is_empty")]
    pub tailwind_classes: EntityTailwindClasses<'id>,

    /// Layout configuration for each node.
    ///
    /// Defines how children of each container node should be arranged.
    #[serde(default, skip_serializing_if = "NodeLayouts::is_empty")]
    pub node_layouts: NodeLayouts<'id>,

    /// Shape configuration for each node.
    ///
    /// Defines the shape and corner radii for each node in the diagram.
    #[serde(default, skip_serializing_if = "NodeShapes::is_empty")]
    pub node_shapes: NodeShapes<'id>,

    /// Additional CSS to place in the SVG's inline `<styles>` section.
    ///
    /// Allows for custom CSS rules such as keyframe animations that
    /// cannot be expressed through Tailwind classes alone.
    #[serde(default, skip_serializing_if = "Css::is_empty")]
    pub css: Css,
}

impl<'id> IrDiagram<'id> {
    /// Returns a new `IrDiagram` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Converts this `IrDiagram` into one with a `'static` lifetime.
    ///
    /// If any inner `Cow` is borrowed, this will clone the string to create
    /// an owned version.
    pub fn into_static(self) -> IrDiagram<'static> {
        IrDiagram {
            nodes: self.nodes.into_static(),
            node_copy_text: self.node_copy_text.into_static(),
            node_hierarchy: self.node_hierarchy.into_static(),
            node_ordering: self.node_ordering.into_static(),
            edge_groups: self.edge_groups.into_static(),
            entity_descs: self.entity_descs.into_static(),
            entity_tooltips: self.entity_tooltips.into_static(),
            entity_types: self.entity_types.into_static(),
            tailwind_classes: self.tailwind_classes.into_static(),
            node_layouts: self.node_layouts.into_static(),
            node_shapes: self.node_shapes.into_static(),
            css: self.css,
        }
    }
}
