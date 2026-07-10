use disposition_model_common::{entity::EntityTooltips, theme::Css, RenderOptions};
use serde::{Deserialize, Serialize};

use crate::{
    edge::{EdgeDescs, EdgeFaceAssignments, EdgeGroups, EdgeLabels},
    entity::{EntityTailwindClasses, EntityTypes},
    layout::NodeLayouts,
    node::{
        NodeCopyText, NodeFaceEdges, NodeHierarchy, NodeNames, NodeNestingInfos, NodeOrdering,
        NodeRanksNested, NodeShapes,
    },
    process::{ProcessStepEdges, ProcessStepEntities, ProcessStepGraphs, ProcessStepRanks},
    thing::{ThingDescs, ThingLayoutEdges},
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
/// thing_descs:
///   t_localhost: "User's computer"
///
/// edge_descs:
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
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
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

    /// Descriptions to render next to things in the diagram.
    #[serde(default, skip_serializing_if = "ThingDescs::is_empty")]
    pub thing_descs: ThingDescs<'id>,

    /// Invisible edges between things that affect rank/layout without ever
    /// being rendered as a path.
    ///
    /// Carried through from the input diagram for debugging/inspection (e.g.
    /// `--data ir-diagram`) -- these edges never enter `edge_groups`, so they
    /// have no effect on rendering; they were already folded into
    /// `node_ranks_nested` during mapping.
    #[serde(default, skip_serializing_if = "ThingLayoutEdges::is_empty")]
    pub thing_layout_edges: ThingLayoutEdges<'id>,

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

    /// Hierarchy-aware computed ranks for nodes based on dependency edges.
    ///
    /// Holds a rank map for the root level and for each container node.
    /// Within each level, nodes with higher ranks are positioned further along
    /// the flex direction axis. Dependency edges that cross container
    /// boundaries are attributed to the lowest common ancestor (LCA) level.
    #[serde(default, skip_serializing_if = "NodeRanksNested::is_empty")]
    pub node_ranks_nested: NodeRanksNested<'id>,

    /// Nesting information for each node in the hierarchy.
    ///
    /// Contains each node's ancestor chain and sibling index path from the
    /// root. Used to compute edge spacer positions for cross-rank edges.
    #[serde(default, skip_serializing_if = "NodeNestingInfos::is_empty")]
    pub node_nesting_infos: NodeNestingInfos<'id>,

    /// Pre-layout face assignment for every edge.
    ///
    /// Maps each edge ID to the faces of its `from` and `to` nodes that the
    /// edge exits or enters. Computed before taffy layout using rank and
    /// sibling data. Used to build envelope nodes with the right number of
    /// edge label slots per face.
    #[serde(default, skip_serializing_if = "EdgeFaceAssignments::is_empty")]
    pub edge_face_assignments: EdgeFaceAssignments<'id>,

    /// Map from node ID and face to the edge IDs on that face.
    ///
    /// Derived from `edge_face_assignments` and `edge_groups`. Used by
    /// `IrToTaffyBuilder` to build the right number of edge label leaf
    /// nodes on each face of each envelope node.
    #[serde(default, skip_serializing_if = "NodeFaceEdges::is_empty")]
    pub node_face_edges: NodeFaceEdges<'id>,

    /// Shape configuration for each node.
    ///
    /// Defines the shape and corner radii for each node in the diagram.
    #[serde(default, skip_serializing_if = "NodeShapes::is_empty")]
    pub node_shapes: NodeShapes<'id>,

    /// Map from process step node IDs to the entity IDs they interact with.
    ///
    /// Each process step can reference one or more entities (typically edge
    /// group IDs from `thing_interactions`) that are activated when the step
    /// is focused. This is used to conditionally attach CSS animations to
    /// edges based on which process step currently has focus.
    #[serde(default, skip_serializing_if = "ProcessStepEntities::is_empty")]
    pub process_step_entities: ProcessStepEntities<'id>,

    /// Directed edges between process steps, derived from process step
    /// dependencies.
    ///
    /// Each edge points from a prerequisite step to a step that depends on it.
    #[serde(default, skip_serializing_if = "ProcessStepEdges::is_empty")]
    pub process_step_edges: ProcessStepEdges<'id>,

    /// Computed ranks for process steps based on process step dependencies.
    ///
    /// Steps that depend on other steps have higher ranks, positioning them
    /// further along the flex direction axis. Steps without any dependencies
    /// default to rank `0`.
    #[serde(default, skip_serializing_if = "ProcessStepRanks::is_empty")]
    pub process_step_ranks: ProcessStepRanks<'id>,

    /// Git-graph layout (lane placement and connectors) for each process's
    /// steps.
    ///
    /// Drives the lane-based positioning of process step circles and the
    /// connector paths drawn between them.
    #[serde(default, skip_serializing_if = "ProcessStepGraphs::is_empty")]
    pub process_step_graphs: ProcessStepGraphs<'id>,

    /// Options that control how the diagram is rendered.
    ///
    /// Includes edge curvature and rank direction settings.
    #[serde(default, skip_serializing_if = "RenderOptions::is_default")]
    pub render_options: RenderOptions,

    /// Additional CSS to place in the SVG's inline `<styles>` section.
    ///
    /// Allows for custom CSS rules such as keyframe animations that
    /// cannot be expressed through Tailwind classes alone.
    #[serde(default, skip_serializing_if = "Css::is_empty")]
    pub css: Css,

    /// Resolved stroke width (pixels) of the interaction edge halo, from
    /// `ThemeAttr::StrokeWidth` on `type_interaction_edge_halo`.
    ///
    /// Used to size the halo's outline rails proportionally to the halo's
    /// own width, rather than a value hardcoded independently of the theme.
    /// Defaults to `0.0` when not resolved (e.g. an `IrDiagram` built
    /// directly rather than through `InputToIrDiagramMapper`).
    #[serde(default)]
    pub interaction_edge_halo_stroke_width: f32,
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
            thing_descs: self.thing_descs.into_static(),
            thing_layout_edges: self.thing_layout_edges.into_static(),
            edge_descs: self.edge_descs.into_static(),
            edge_labels: self.edge_labels.into_static(),
            entity_tooltips: self.entity_tooltips.into_static(),
            entity_types: self.entity_types.into_static(),
            tailwind_classes: self.tailwind_classes.into_static(),
            node_layouts: self.node_layouts.into_static(),
            node_ranks_nested: self.node_ranks_nested.into_static(),
            node_nesting_infos: self.node_nesting_infos.into_static(),
            edge_face_assignments: self.edge_face_assignments.into_static(),
            node_face_edges: self.node_face_edges.into_static(),
            node_shapes: self.node_shapes.into_static(),
            process_step_entities: self.process_step_entities.into_static(),
            process_step_edges: self.process_step_edges.into_static(),
            process_step_ranks: self.process_step_ranks.into_static(),
            process_step_graphs: self.process_step_graphs.into_static(),
            render_options: self.render_options,
            css: self.css,
            interaction_edge_halo_stroke_width: self.interaction_edge_halo_stroke_width,
        }
    }
}
