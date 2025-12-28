use disposition_model_common::theme::Css;
use serde::{Deserialize, Serialize};

use crate::{
    edge::EdgeGroups,
    entity::{EntityDescs, EntityTailwindClasses, EntityTypes},
    layout::NodeLayouts,
    node::{NodeCopyText, NodeHierarchy, NodeNames},
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
/// * `node_hierarchy` includes all node types, not just things
/// * `edge_groups` contains explicit `from`/`to` edges instead of `EdgeKind`
/// * `tailwind_classes` contains computed CSS classes instead of theme configs
/// * `node_layout` defines positioning for all nodes
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
/// edge_groups:
///   edge_t_localhost__t_github_user_repo:
///     - from: t_localhost
///       to: t_github_user_repo
///
/// entity_descs:
///   proc_app_release_step_tag_and_push: |-
///     When the PR is merged, tag the commit and push the tag to GitHub.
///   edge_t_localhost__t_github_user_repo__pull: "Fetch from GitHub"
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
pub struct IrDiagram {
    /// All nodes in the diagram and their display labels.
    ///
    /// This includes things, tags, processes, and process steps.
    #[serde(default, skip_serializing_if = "NodeNames::is_empty")]
    pub nodes: NodeNames,

    /// Text to copy to clipboard when a node's copy button is clicked.
    ///
    /// This allows nodes to have different copy text than their display label.
    /// Typically only includes `thing` nodes.
    #[serde(default, skip_serializing_if = "NodeCopyText::is_empty")]
    pub node_copy_text: NodeCopyText,

    /// Hierarchy of all nodes as a recursive tree structure.
    ///
    /// This includes tags, processes (with their steps), and things.
    /// The order of declaration is important for CSS peer selector ordering.
    #[serde(default, skip_serializing_if = "NodeHierarchy::is_empty")]
    pub node_hierarchy: NodeHierarchy,

    /// Edge groups derived from `thing_dependencies` and `thing_interactions`.
    ///
    /// Each edge group contains explicit `from`/`to` edges.
    #[serde(default, skip_serializing_if = "EdgeGroups::is_empty")]
    pub edge_groups: EdgeGroups,

    /// Descriptions for entities (nodes, edges, and edge groups).
    ///
    /// Contains text (typically markdown) that provides additional context
    /// about entities in the diagram, such as process steps or edge labels.
    #[serde(default, skip_serializing_if = "EntityDescs::is_empty")]
    pub entity_descs: EntityDescs,

    /// Entity types attached to nodes and edges for common styling.
    ///
    /// Each node/edge can have multiple types, allowing styles to be stacked.
    #[serde(default, skip_serializing_if = "EntityTypes::is_empty")]
    pub entity_types: EntityTypes,

    /// Computed Tailwind CSS classes for interactive visibility behaviour.
    ///
    /// These classes control visibility, colors, animations, and interactions
    /// based on the diagram's state.
    #[serde(default, skip_serializing_if = "EntityTailwindClasses::is_empty")]
    pub tailwind_classes: EntityTailwindClasses,

    /// Layout configuration for each node.
    ///
    /// Defines how children of each container node should be arranged.
    #[serde(default, skip_serializing_if = "NodeLayouts::is_empty")]
    pub node_layouts: NodeLayouts,

    /// Additional CSS to place in the SVG's inline `<styles>` section.
    ///
    /// Allows for custom CSS rules such as keyframe animations that
    /// cannot be expressed through Tailwind classes alone.
    #[serde(default, skip_serializing_if = "Css::is_empty")]
    pub css: Css,
}

impl IrDiagram {
    /// Returns a new `IrDiagram` with default values.
    pub fn new() -> Self {
        Self::default()
    }
}
