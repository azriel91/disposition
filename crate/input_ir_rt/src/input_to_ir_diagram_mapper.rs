use disposition_input_ir_model::IrDiagramAndIssues;
use disposition_input_model::{
    edge::EdgeKind,
    process::Processes,
    tag::{TagNames, TagThings},
    theme::{
        CssClassPartials, IdOrDefaults, StyleAliases, ThemeAttr, ThemeDefault, ThemeTypesStyles,
    },
    thing::ThingHierarchy as InputThingHierarchy,
    InputDiagram,
};
use disposition_ir_model::{
    edge::{Edge, EdgeGroup, EdgeGroupId, EdgeGroups},
    entity::{EntityTailwindClasses, EntityType, EntityTypes as IrEntityTypes},
    layout::{FlexDirection, FlexLayout, NodeLayout, NodeLayouts},
    node::{NodeCopyText, NodeHierarchy, NodeId, NodeNames},
    IrDiagram,
};
use disposition_model_common::{entity::EntityDescs, id, Id, Map};

/// Maps an input diagram to an intermediate representation diagram.
#[derive(Clone, Copy, Debug)]
pub struct InputToIrDiagramMapper;

impl InputToIrDiagramMapper {
    /// Maps an input diagram to an intermediate representation diagram.
    pub fn map(input_diagram: InputDiagram) -> IrDiagramAndIssues {
        let issues = Vec::new();

        let InputDiagram {
            things,
            thing_copy_text,
            thing_hierarchy,
            thing_dependencies,
            thing_interactions,
            processes,
            tags,
            tag_things,
            entity_descs,
            entity_types,
            theme_default,
            theme_types_styles,
            theme_thing_dependencies_styles: _,
            theme_tag_things_focus: _,
            theme_tag_things_focus_specific: _,
            css,
        } = input_diagram;

        // 1. Build NodeNames from things, tags, processes, and process steps
        let nodes = Self::build_node_names(&things, &tags, &processes);

        // 2. Build NodeCopyText from thing_copy_text
        let node_copy_text = Self::build_node_copy_text(&thing_copy_text);

        // 3. Build NodeHierarchy from tags, processes (with steps), and thing_hierarchy
        let node_hierarchy = Self::build_node_hierarchy(&tags, &processes, &thing_hierarchy);

        // 4. Build EdgeGroups from thing_dependencies and thing_interactions
        let edge_groups = Self::build_edge_groups(&thing_dependencies, &thing_interactions);

        // 5. Build EntityDescs from input entity_descs and process step_descs
        let entity_descs = Self::build_entity_descs(&entity_descs, &processes);

        // 6. Build EntityTypes with defaults for each node type
        let ir_entity_types = Self::build_entity_types(
            &things,
            &tags,
            &processes,
            &entity_types,
            &thing_dependencies,
            &thing_interactions,
        );

        // 7. Build NodeLayouts from node_hierarchy and theme
        let node_layout = Self::build_node_layouts(
            &node_hierarchy,
            &ir_entity_types,
            &theme_default,
            &theme_types_styles,
            &tags,
            &processes,
        );

        // 8. Build TailwindClasses from theme
        let tailwind_classes = Self::build_tailwind_classes(
            &nodes,
            &edge_groups,
            &ir_entity_types,
            &theme_default,
            &theme_types_styles,
            &tags,
            &tag_things,
            &processes,
        );

        let diagram = IrDiagram {
            nodes,
            node_copy_text,
            node_hierarchy,
            edge_groups,
            entity_descs,
            entity_types: ir_entity_types,
            tailwind_classes,
            node_layout,
            css,
        };

        IrDiagramAndIssues { diagram, issues }
    }

    /// Creates an Id from a String.
    fn id_from_string(s: String) -> Id {
        // Use TryFrom<String> to create an Id from an owned String.
        // This will validate the ID format and create a Cow::Owned internally.
        Id::try_from(s).expect("valid ID string")
    }

    /// Build NodeNames from things, tags, processes, and process steps.
    fn build_node_names(
        things: &disposition_input_model::thing::ThingNames,
        tags: &disposition_input_model::tag::TagNames,
        processes: &disposition_input_model::process::Processes,
    ) -> NodeNames {
        let mut nodes = NodeNames::new();

        // Add things
        for (thing_id, name) in things.iter() {
            let node_id: NodeId = thing_id.clone().into_inner().into();
            nodes.insert(node_id, name.clone());
        }

        // Add tags
        for (tag_id, name) in tags.iter() {
            let node_id: NodeId = tag_id.clone().into_inner().into();
            nodes.insert(node_id, name.clone());
        }

        // Add processes and their steps
        for (process_id, process_diagram) in processes.iter() {
            // Add process name
            let process_node_id: NodeId = process_id.clone().into_inner().into();
            let process_name = process_diagram
                .name
                .clone()
                .unwrap_or_else(|| process_id.as_str().to_string());
            nodes.insert(process_node_id, process_name);

            // Add process steps
            for (step_id, step_name) in process_diagram.steps.iter() {
                let step_node_id: NodeId = step_id.clone().into_inner().into();
                nodes.insert(step_node_id, step_name.clone());
            }
        }

        nodes
    }

    /// Build NodeCopyText from thing_copy_text.
    fn build_node_copy_text(
        thing_copy_text: &disposition_input_model::thing::ThingCopyText,
    ) -> NodeCopyText {
        let mut node_copy_text = NodeCopyText::new();

        for (thing_id, text) in thing_copy_text.iter() {
            let node_id: NodeId = thing_id.clone().into_inner().into();
            node_copy_text.insert(node_id, text.clone());
        }

        node_copy_text
    }

    /// Build NodeHierarchy from tags, processes (with steps), and
    /// thing_hierarchy.
    fn build_node_hierarchy(
        tags: &disposition_input_model::tag::TagNames,
        processes: &disposition_input_model::process::Processes,
        thing_hierarchy: &InputThingHierarchy,
    ) -> NodeHierarchy {
        let mut hierarchy = NodeHierarchy::new();

        // Add tags first (for CSS peer selector ordering)
        for (tag_id, _) in tags.iter() {
            let node_id: NodeId = tag_id.clone().into_inner().into();
            hierarchy.insert(node_id, NodeHierarchy::new());
        }

        // Add processes with their steps
        for (process_id, process_diagram) in processes.iter() {
            let process_node_id: NodeId = process_id.clone().into_inner().into();
            let mut process_children = NodeHierarchy::new();

            for (step_id, _) in process_diagram.steps.iter() {
                let step_node_id: NodeId = step_id.clone().into_inner().into();
                process_children.insert(step_node_id, NodeHierarchy::new());
            }

            hierarchy.insert(process_node_id, process_children);
        }

        // Add things hierarchy
        Self::convert_thing_hierarchy_to_node_hierarchy(thing_hierarchy, &mut hierarchy);

        hierarchy
    }

    /// Recursively convert ThingHierarchy to NodeHierarchy.
    fn convert_thing_hierarchy_to_node_hierarchy(
        thing_hierarchy: &InputThingHierarchy,
        node_hierarchy: &mut NodeHierarchy,
    ) {
        for (thing_id, children) in thing_hierarchy.iter() {
            let node_id: NodeId = thing_id.clone().into_inner().into();
            let mut child_hierarchy = NodeHierarchy::new();
            Self::convert_thing_hierarchy_to_node_hierarchy(children, &mut child_hierarchy);
            node_hierarchy.insert(node_id, child_hierarchy);
        }
    }

    /// Build EdgeGroups from thing_dependencies and thing_interactions.
    fn build_edge_groups(
        thing_dependencies: &disposition_input_model::thing::ThingDependencies,
        thing_interactions: &disposition_input_model::thing::ThingInteractions,
    ) -> EdgeGroups {
        let mut edge_groups = EdgeGroups::new();

        // Process thing_dependencies
        for (edge_id, edge_kind) in thing_dependencies.iter() {
            let edge_group_id: EdgeGroupId = edge_id.clone().into_inner().into();
            let edges = Self::edge_kind_to_edges(edge_kind);
            edge_groups.insert(edge_group_id, edges);
        }

        // Process thing_interactions (merge with dependencies if same ID exists)
        for (edge_id, edge_kind) in thing_interactions.iter() {
            let edge_group_id: EdgeGroupId = edge_id.clone().into_inner().into();
            // Only add if not already present from dependencies
            if !edge_groups.contains_key(&edge_group_id) {
                let edges = Self::edge_kind_to_edges(edge_kind);
                edge_groups.insert(edge_group_id, edges);
            }
        }

        edge_groups
    }

    /// Convert an EdgeKind to a list of Edges.
    fn edge_kind_to_edges(edge_kind: &EdgeKind) -> EdgeGroup {
        let mut edges = Vec::new();

        match edge_kind {
            EdgeKind::Cyclic(things) => {
                if things.is_empty() {
                    return EdgeGroup::from(edges);
                }

                // Create edges from each thing to the next, and from last back to first
                for i in 0..things.len() {
                    let from_id: NodeId = things[i].clone().into_inner().into();
                    let to_idx = (i + 1) % things.len();
                    let to_id: NodeId = things[to_idx].clone().into_inner().into();
                    edges.push(Edge::new(from_id, to_id));
                }
            }
            EdgeKind::Sequence(things) => {
                // Create edges from each thing to the next (no cycle back)
                for i in 0..things.len().saturating_sub(1) {
                    let from_id: NodeId = things[i].clone().into_inner().into();
                    let to_id: NodeId = things[i + 1].clone().into_inner().into();
                    edges.push(Edge::new(from_id, to_id));
                }
            }
        }

        EdgeGroup::from(edges)
    }

    /// Build EntityDescs from input entity_descs and process step_descs.
    fn build_entity_descs(
        input_entity_descs: &EntityDescs,
        processes: &disposition_input_model::process::Processes,
    ) -> EntityDescs {
        let mut entity_descs = EntityDescs::new();

        // Copy existing entity descs
        for (id, desc) in input_entity_descs.iter() {
            entity_descs.insert(id.clone(), desc.clone());
        }

        // Add process step descriptions
        for (_, process_diagram) in processes.iter() {
            for (step_id, desc) in process_diagram.step_descs.iter() {
                let id: Id = step_id.clone().into_inner();
                entity_descs.insert(id, desc.clone());
            }
        }

        entity_descs
    }

    /// Build EntityTypes with defaults for each node type.
    fn build_entity_types(
        things: &disposition_input_model::thing::ThingNames,
        tags: &disposition_input_model::tag::TagNames,
        processes: &disposition_input_model::process::Processes,
        input_entity_types: &disposition_input_model::entity::EntityTypes,
        thing_dependencies: &disposition_input_model::thing::ThingDependencies,
        thing_interactions: &disposition_input_model::thing::ThingInteractions,
    ) -> IrEntityTypes {
        let mut entity_types: Map<Id, Vec<EntityType>> = Map::new();

        // Add things with type_thing_default + any custom type
        for (thing_id, _) in things.iter() {
            let id: Id = thing_id.clone().into_inner();
            let mut types = vec![EntityType::ThingDefault];

            // Check if there's a custom type specified
            if let Some(custom_type) = input_entity_types.get(&id) {
                types.push(EntityType::from(custom_type.clone().into_inner()));
            }

            entity_types.insert(id, types);
        }

        // Add tags with tag_type_default
        for (tag_id, _) in tags.iter() {
            let id: Id = tag_id.clone().into_inner();
            let mut types = vec![EntityType::TagDefault];

            if let Some(custom_type) = input_entity_types.get(&id) {
                types.push(EntityType::from(custom_type.clone().into_inner()));
            }

            entity_types.insert(id, types);
        }

        // Add processes with type_process_default
        for (process_id, process_diagram) in processes.iter() {
            let id: Id = process_id.clone().into_inner();
            let mut types = vec![EntityType::ProcessDefault];

            if let Some(custom_type) = input_entity_types.get(&id) {
                types.push(EntityType::from(custom_type.clone().into_inner()));
            }

            entity_types.insert(id, types);

            // Add process steps with type_process_step_default
            for (step_id, _) in process_diagram.steps.iter() {
                let id: Id = step_id.clone().into_inner();
                let mut types = vec![EntityType::ProcessStepDefault];

                if let Some(custom_type) = input_entity_types.get(&id) {
                    types.push(EntityType::from(custom_type.clone().into_inner()));
                }

                entity_types.insert(id, types);
            }
        }

        // Add edge types from thing_dependencies
        Self::add_edge_types(&mut entity_types, thing_dependencies, input_entity_types);

        // Add edge types from thing_interactions (will merge with existing)
        Self::add_edge_interaction_types(&mut entity_types, thing_interactions, input_entity_types);

        IrEntityTypes::from(entity_types)
    }

    /// Add edge types from dependencies.
    fn add_edge_types(
        entity_types: &mut Map<Id, Vec<EntityType>>,
        thing_deps: &disposition_input_model::thing::ThingDependencies,
        input_entity_types: &disposition_input_model::entity::EntityTypes,
    ) {
        for (edge_group_id, edge_kind) in thing_deps.iter() {
            let edge_count = match edge_kind {
                EdgeKind::Cyclic(things) => things.len(),
                EdgeKind::Sequence(things) => things.len().saturating_sub(1),
            };

            for i in 0..edge_count {
                // Edge ID format: edge_group_id__index
                let edge_id_str = format!("{}__{}", edge_group_id.as_str(), i);
                let edge_id = Self::id_from_string(edge_id_str);

                let default_type = match edge_kind {
                    EdgeKind::Cyclic(_) => EntityType::EdgeDependencyCyclicDefault,
                    EdgeKind::Sequence(_) => EntityType::EdgeDependencySequenceRequestDefault,
                };

                let types = if let Some(custom_type) = input_entity_types.get(&edge_id) {
                    vec![
                        default_type,
                        EntityType::from(custom_type.clone().into_inner()),
                    ]
                } else {
                    vec![default_type]
                };

                entity_types.insert(edge_id, types);
            }
        }
    }

    /// Add interaction types to existing edge types.
    fn add_edge_interaction_types(
        entity_types: &mut Map<Id, Vec<EntityType>>,
        thing_interactions: &disposition_input_model::thing::ThingInteractions,
        _input_entity_types: &disposition_input_model::entity::EntityTypes,
    ) {
        for (edge_group_id, edge_kind) in thing_interactions.iter() {
            let edge_count = match edge_kind {
                EdgeKind::Cyclic(things) => things.len(),
                EdgeKind::Sequence(things) => things.len().saturating_sub(1),
            };

            for i in 0..edge_count {
                let edge_id_str = format!("{}__{}", edge_group_id.as_str(), i);
                let edge_id = Self::id_from_string(edge_id_str);

                let interaction_type = match edge_kind {
                    EdgeKind::Cyclic(_) => EntityType::EdgeInteractionCyclicDefault,
                    EdgeKind::Sequence(_) => EntityType::EdgeInteractionSequenceRequestDefault,
                };

                // Add to existing types or create new entry
                if let Some(types) = entity_types.get_mut(&edge_id) {
                    types.push(interaction_type);
                } else {
                    entity_types.insert(edge_id, vec![interaction_type]);
                }
            }
        }
    }

    /// Build NodeLayouts from node_hierarchy and theme data.
    fn build_node_layouts(
        node_hierarchy: &NodeHierarchy,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
        tags: &disposition_input_model::tag::TagNames,
        processes: &disposition_input_model::process::Processes,
    ) -> NodeLayouts {
        let mut node_layouts = NodeLayouts::new();

        // Collect tag and process node IDs for special handling
        let tag_ids: Vec<NodeId> = tags
            .iter()
            .map(|(tag_id, _)| NodeId::from(tag_id.clone().into_inner()))
            .collect();
        let process_ids: Vec<NodeId> = processes
            .iter()
            .map(|(proc_id, _)| NodeId::from(proc_id.clone().into_inner()))
            .collect();

        // Helper to determine if a node is a tag
        let is_tag = |node_id: &NodeId| tag_ids.iter().any(|id| id == node_id);

        // Helper to determine if a node is a process
        let is_process = |node_id: &NodeId| process_ids.iter().any(|id| id == node_id);

        // 1. Add _root container layout
        let root_id = id!("_root");
        let root_layout = Self::build_container_layout(
            &root_id,
            FlexDirection::ColumnReverse,
            true,
            entity_types,
            theme_default,
            theme_types_styles,
        );
        node_layouts.insert(NodeId::from(root_id), root_layout);

        // 2. Add _things_and_processes_container layout
        let things_and_processes_id = id!("_things_and_processes_container");
        let things_and_processes_layout = Self::build_container_layout(
            &things_and_processes_id,
            FlexDirection::RowReverse,
            true,
            entity_types,
            theme_default,
            theme_types_styles,
        );
        node_layouts.insert(
            NodeId::from(things_and_processes_id),
            things_and_processes_layout,
        );

        // 3. Add _processes_container layout
        let processes_container_id = id!("_processes_container");
        let processes_container_layout = Self::build_container_layout(
            &processes_container_id,
            FlexDirection::Row,
            true,
            entity_types,
            theme_default,
            theme_types_styles,
        );
        node_layouts.insert(
            NodeId::from(processes_container_id),
            processes_container_layout,
        );

        // 4. Build layouts for all processes
        for (process_id, process_diagram) in processes.iter() {
            let process_node_id = NodeId::from(process_id.clone().into_inner());

            // Processes with steps get flex layout (column direction)
            if !process_diagram.steps.is_empty() {
                let layout = Self::build_node_flex_layout(
                    &process_node_id,
                    FlexDirection::Column,
                    false,
                    entity_types,
                    theme_default,
                    theme_types_styles,
                );
                node_layouts.insert(process_node_id.clone(), layout);
            } else {
                node_layouts.insert(process_node_id.clone(), NodeLayout::None);
            }

            // Process steps are always leaves (no children)
            for (step_id, _) in process_diagram.steps.iter() {
                let step_node_id = NodeId::from(step_id.clone().into_inner());
                node_layouts.insert(step_node_id, NodeLayout::None);
            }
        }

        // 5. Add _tags_container layout
        let tags_container_id = id!("_tags_container");
        let tags_container_layout = Self::build_container_layout(
            &tags_container_id,
            FlexDirection::Row,
            true,
            entity_types,
            theme_default,
            theme_types_styles,
        );
        node_layouts.insert(NodeId::from(tags_container_id), tags_container_layout);

        // 6. Tags are always leaves
        for (tag_id, _) in tags.iter() {
            let tag_node_id = NodeId::from(tag_id.clone().into_inner());
            node_layouts.insert(tag_node_id, NodeLayout::None);
        }

        // 7. Add _things_container layout
        let things_container_id = id!("_things_container");
        let things_container_layout = Self::build_container_layout(
            &things_container_id,
            FlexDirection::Row,
            true,
            entity_types,
            theme_default,
            theme_types_styles,
        );
        node_layouts.insert(NodeId::from(things_container_id), things_container_layout);

        // 8. Build layouts for all things in hierarchy
        Self::build_thing_layouts(
            node_hierarchy,
            0,
            entity_types,
            theme_default,
            theme_types_styles,
            &mut node_layouts,
            &is_tag,
            &is_process,
        );

        node_layouts
    }

    /// Build a container layout with specified direction.
    fn build_container_layout(
        container_id: &Id,
        direction: FlexDirection,
        wrap: bool,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
    ) -> NodeLayout {
        // Containers don't have entity types, so we only resolve from NodeDefaults
        let (padding_top, padding_right, padding_bottom, padding_left) = Self::resolve_padding(
            Some(container_id),
            entity_types,
            theme_default,
            theme_types_styles,
        );
        let (margin_top, margin_right, margin_bottom, margin_left) = Self::resolve_margin(
            Some(container_id),
            entity_types,
            theme_default,
            theme_types_styles,
        );
        let gap = Self::resolve_gap(
            Some(container_id),
            entity_types,
            theme_default,
            theme_types_styles,
        );

        NodeLayout::Flex(FlexLayout {
            direction,
            wrap,
            padding_top,
            padding_right,
            padding_bottom,
            padding_left,
            margin_top,
            margin_right,
            margin_bottom,
            margin_left,
            gap,
        })
    }

    /// Build a flex layout for a specific node.
    fn build_node_flex_layout(
        node_id: &NodeId,
        direction: FlexDirection,
        wrap: bool,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
    ) -> NodeLayout {
        let id: Id = node_id.clone().into_inner();
        let (padding_top, padding_right, padding_bottom, padding_left) =
            Self::resolve_padding(Some(&id), entity_types, theme_default, theme_types_styles);
        let (margin_top, margin_right, margin_bottom, margin_left) =
            Self::resolve_margin(Some(&id), entity_types, theme_default, theme_types_styles);
        let gap = Self::resolve_gap(Some(&id), entity_types, theme_default, theme_types_styles);

        NodeLayout::Flex(FlexLayout {
            direction,
            wrap,
            padding_top,
            padding_right,
            padding_bottom,
            padding_left,
            margin_top,
            margin_right,
            margin_bottom,
            margin_left,
            gap,
        })
    }

    /// Recursively build layouts for things in the hierarchy.
    #[allow(clippy::too_many_arguments)] // we may reduce this during refactoring
    fn build_thing_layouts<F, G>(
        hierarchy: &NodeHierarchy,
        depth: usize,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
        node_layouts: &mut NodeLayouts,
        is_tag: &F,
        is_process: &G,
    ) where
        F: Fn(&NodeId) -> bool,
        G: Fn(&NodeId) -> bool,
    {
        for (node_id, children) in hierarchy.iter() {
            // Skip tags and processes (already handled)
            if is_tag(node_id) || is_process(node_id) {
                continue;
            }

            if children.is_empty() {
                // Leaf node - no layout needed
                node_layouts.insert(node_id.clone(), NodeLayout::None);
            } else {
                // Container node - use flex layout
                // Direction alternates based on depth: column at even depths, row at odd depths
                let direction = if depth.is_multiple_of(2) {
                    FlexDirection::Column
                } else {
                    FlexDirection::Row
                };

                let layout = Self::build_node_flex_layout(
                    node_id,
                    direction,
                    false,
                    entity_types,
                    theme_default,
                    theme_types_styles,
                );
                node_layouts.insert(node_id.clone(), layout);

                // Recursively process children
                Self::build_thing_layouts(
                    children,
                    depth + 1,
                    entity_types,
                    theme_default,
                    theme_types_styles,
                    node_layouts,
                    is_tag,
                    is_process,
                );
            }
        }
    }

    /// Resolves a theme attribute value by traversing theme sources in priority
    /// order:
    ///
    /// 1. `NodeDefaults` from `theme_default` (lowest priority)
    /// 2. `EntityType`s applied to the node (in order, later overrides earlier)
    /// 3. The `NodeId` itself from `theme_default` (highest priority)
    ///
    /// Within each level, `StyleAlias`es are applied first, then direct
    /// attributes.
    ///
    /// # Parameters
    /// - `state`: Mutable state that accumulates resolved values
    /// - `apply_from_partials`: Closure that extracts values from
    ///   `CssClassPartials` and applies them to state, considering style
    ///   aliases
    /// - `finalize`: Closure that converts the accumulated state into the final
    ///   result with defaults
    fn resolve_theme_attr<State, Result>(
        node_id: Option<&Id>,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
        state: &mut State,
        apply_from_partials: impl Fn(&CssClassPartials, &StyleAliases, &mut State),
        finalize: impl FnOnce(&State) -> Result,
    ) -> Result {
        // 1. Start with NodeDefaults (lowest priority)
        if let Some(node_defaults_partials) =
            theme_default.base_styles.get(&IdOrDefaults::NodeDefaults)
        {
            apply_from_partials(node_defaults_partials, &theme_default.style_aliases, state);
        }

        // 2. Apply EntityTypes in order (later types override earlier ones)
        if let Some(id) = node_id
            && let Some(types) = entity_types.get(id)
        {
            for entity_type in types.iter() {
                let type_id = disposition_model_common::entity::EntityTypeId::from(
                    Self::id_from_string(entity_type.as_str().to_string()),
                );
                if let Some(type_styles) = theme_types_styles.get(&type_id)
                    && let Some(type_partials) = type_styles.get(&IdOrDefaults::NodeDefaults)
                {
                    apply_from_partials(type_partials, &theme_default.style_aliases, state);
                }
            }
        }

        // 3. Apply node ID itself (highest priority)
        if let Some(id) = node_id
            && let Some(node_partials) =
                theme_default.base_styles.get(&IdOrDefaults::Id(id.clone()))
        {
            apply_from_partials(node_partials, &theme_default.style_aliases, state);
        }

        finalize(state)
    }

    fn resolve_padding(
        node_id: Option<&Id>,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
    ) -> (f32, f32, f32, f32) {
        let mut state = (None, None, None, None);

        Self::resolve_theme_attr(
            node_id,
            entity_types,
            theme_default,
            theme_types_styles,
            &mut state,
            Self::apply_padding_from_partials,
            |state| {
                (
                    state.0.unwrap_or(0.0),
                    state.1.unwrap_or(0.0),
                    state.2.unwrap_or(0.0),
                    state.3.unwrap_or(0.0),
                )
            },
        )
    }

    /// Apply padding values from CssClassPartials, checking both direct
    /// attributes and style aliases.
    fn apply_padding_from_partials(
        partials: &CssClassPartials,
        style_aliases: &StyleAliases,
        state: &mut (Option<f32>, Option<f32>, Option<f32>, Option<f32>),
    ) {
        // First, check style_aliases_applied (lower priority within this partials)
        for alias in partials.style_aliases_applied() {
            if let Some(alias_partials) = style_aliases.get(alias) {
                Self::extract_padding_from_map(alias_partials, state);
            }
        }

        // Then, check direct attributes (higher priority within this partials)
        Self::extract_padding_from_map(partials, state);
    }

    /// Extract padding values from a map of ThemeAttr to String.
    fn extract_padding_from_map(
        partials: &CssClassPartials,
        state: &mut (Option<f32>, Option<f32>, Option<f32>, Option<f32>),
    ) {
        let (padding_top, padding_right, padding_bottom, padding_left) = state;

        // Check compound Padding first (applies to all sides)
        if let Some(value) = partials.get(&ThemeAttr::Padding)
            && let Ok(v) = value.parse::<f32>()
        {
            *padding_top = Some(v);
            *padding_right = Some(v);
            *padding_bottom = Some(v);
            *padding_left = Some(v);
        }

        // Check PaddingX (horizontal) - overrides Padding for left/right
        if let Some(value) = partials.get(&ThemeAttr::PaddingX)
            && let Ok(v) = value.parse::<f32>()
        {
            *padding_left = Some(v);
            *padding_right = Some(v);
        }

        // Check PaddingY (vertical) - overrides Padding for top/bottom
        if let Some(value) = partials.get(&ThemeAttr::PaddingY)
            && let Ok(v) = value.parse::<f32>()
        {
            *padding_top = Some(v);
            *padding_bottom = Some(v);
        }

        // Check specific padding attributes (highest specificity)
        if let Some(value) = partials.get(&ThemeAttr::PaddingTop)
            && let Ok(v) = value.parse::<f32>()
        {
            *padding_top = Some(v);
        }
        if let Some(value) = partials.get(&ThemeAttr::PaddingRight)
            && let Ok(v) = value.parse::<f32>()
        {
            *padding_right = Some(v);
        }
        if let Some(value) = partials.get(&ThemeAttr::PaddingBottom)
            && let Ok(v) = value.parse::<f32>()
        {
            *padding_bottom = Some(v);
        }
        if let Some(value) = partials.get(&ThemeAttr::PaddingLeft)
            && let Ok(v) = value.parse::<f32>()
        {
            *padding_left = Some(v);
        }
    }

    fn resolve_margin(
        node_id: Option<&Id>,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
    ) -> (f32, f32, f32, f32) {
        let mut state = (None, None, None, None);

        Self::resolve_theme_attr(
            node_id,
            entity_types,
            theme_default,
            theme_types_styles,
            &mut state,
            Self::apply_margin_from_partials,
            |state| {
                (
                    state.0.unwrap_or(0.0),
                    state.1.unwrap_or(0.0),
                    state.2.unwrap_or(0.0),
                    state.3.unwrap_or(0.0),
                )
            },
        )
    }

    /// Apply margin values from CssClassPartials, checking both direct
    /// attributes and style aliases.
    fn apply_margin_from_partials(
        partials: &CssClassPartials,
        style_aliases: &StyleAliases,
        state: &mut (Option<f32>, Option<f32>, Option<f32>, Option<f32>),
    ) {
        // First, check style_aliases_applied (lower priority within this partials)
        for alias in partials.style_aliases_applied() {
            if let Some(alias_partials) = style_aliases.get(alias) {
                Self::extract_margin_from_map(alias_partials, state);
            }
        }

        // Then, check direct attributes (higher priority within this partials)
        Self::extract_margin_from_map(partials, state);
    }

    /// Extract margin values from a map of ThemeAttr to String.
    fn extract_margin_from_map(
        partials: &CssClassPartials,
        state: &mut (Option<f32>, Option<f32>, Option<f32>, Option<f32>),
    ) {
        let (margin_top, margin_right, margin_bottom, margin_left) = state;

        // Check compound Margin first (applies to all sides)
        if let Some(value) = partials.get(&ThemeAttr::Margin)
            && let Ok(v) = value.parse::<f32>()
        {
            *margin_top = Some(v);
            *margin_right = Some(v);
            *margin_bottom = Some(v);
            *margin_left = Some(v);
        }

        // Check MarginX (horizontal) - overrides Margin for left/right
        if let Some(value) = partials.get(&ThemeAttr::MarginX)
            && let Ok(v) = value.parse::<f32>()
        {
            *margin_left = Some(v);
            *margin_right = Some(v);
        }

        // Check MarginY (vertical) - overrides Margin for top/bottom
        if let Some(value) = partials.get(&ThemeAttr::MarginY)
            && let Ok(v) = value.parse::<f32>()
        {
            *margin_top = Some(v);
            *margin_bottom = Some(v);
        }

        // Check specific margin attributes (highest specificity)
        if let Some(value) = partials.get(&ThemeAttr::MarginTop)
            && let Ok(v) = value.parse::<f32>()
        {
            *margin_top = Some(v);
        }
        if let Some(value) = partials.get(&ThemeAttr::MarginRight)
            && let Ok(v) = value.parse::<f32>()
        {
            *margin_right = Some(v);
        }
        if let Some(value) = partials.get(&ThemeAttr::MarginBottom)
            && let Ok(v) = value.parse::<f32>()
        {
            *margin_bottom = Some(v);
        }
        if let Some(value) = partials.get(&ThemeAttr::MarginLeft)
            && let Ok(v) = value.parse::<f32>()
        {
            *margin_left = Some(v);
        }
    }

    fn resolve_gap(
        node_id: Option<&Id>,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
    ) -> f32 {
        let mut state = None;

        Self::resolve_theme_attr(
            node_id,
            entity_types,
            theme_default,
            theme_types_styles,
            &mut state,
            Self::apply_gap_from_partials,
            |state| state.unwrap_or(0.0),
        )
    }

    /// Apply gap value from CssClassPartials, checking both direct attributes
    /// and style aliases.
    fn apply_gap_from_partials(
        partials: &CssClassPartials,
        style_aliases: &StyleAliases,
        state: &mut Option<f32>,
    ) {
        // First, check style_aliases_applied (lower priority within this partials)
        for alias in partials.style_aliases_applied() {
            if let Some(alias_partials) = style_aliases.get(alias)
                && let Some(value) = alias_partials.get(&ThemeAttr::Gap)
                && let Ok(v) = value.parse::<f32>()
            {
                *state = Some(v);
            }
        }

        // Then, check direct attribute (higher priority within this partials)
        if let Some(value) = partials.get(&ThemeAttr::Gap)
            && let Ok(v) = value.parse::<f32>()
        {
            *state = Some(v);
        }
    }

    // =========================================================================
    // Tailwind Classes Building
    // =========================================================================

    /// Build tailwind classes for all entities (nodes, edge groups, edges).
    #[allow(clippy::too_many_arguments)]
    fn build_tailwind_classes(
        nodes: &NodeNames,
        edge_groups: &EdgeGroups,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
        tags: &TagNames,
        tag_things: &TagThings,
        processes: &Processes,
    ) -> EntityTailwindClasses {
        let mut tailwind_classes = EntityTailwindClasses::new();

        // Build a map of process step ID to (process ID, edge IDs they interact with)
        let step_interactions = Self::build_step_interactions_map(processes);

        // Build a map of tag ID to thing IDs
        let tag_thing_ids: Map<Id, Vec<Id>> = tag_things
            .iter()
            .map(|(tag_id, thing_ids)| {
                let tag_id: Id = tag_id.clone().into_inner();
                let thing_ids: Vec<Id> = thing_ids
                    .iter()
                    .map(|thing_id| thing_id.clone().into_inner())
                    .collect();
                (tag_id, thing_ids)
            })
            .collect();

        // Build a map of edge group ID to process steps that interact with it
        let edge_group_to_steps = Self::build_edge_group_to_steps_map(processes);

        // Build a map of thing ID to process steps that interact with edges involving
        // that thing
        let thing_to_interacting_steps =
            Self::build_thing_to_interacting_steps_map(edge_groups, &step_interactions);

        // Build classes for each node
        for (node_id, _name) in nodes.iter() {
            let id: Id = node_id.clone().into_inner();

            // Determine node kind
            let is_tag = tags.iter().any(|(t, _)| t.as_str() == id.as_str());
            let is_process = processes.iter().any(|(p, _)| p.as_str() == id.as_str());
            let is_process_step = processes
                .iter()
                .any(|(_, pd)| pd.steps.iter().any(|(s, _)| s.as_str() == id.as_str()));

            let classes = if is_tag {
                Self::build_tag_tailwind_classes(
                    &id,
                    entity_types,
                    theme_default,
                    theme_types_styles,
                )
            } else if is_process {
                // Find the child process step IDs
                let child_step_ids: Vec<Id> = processes
                    .iter()
                    .find(|(p, _)| p.as_str() == id.as_str())
                    .map(|(_, pd)| {
                        pd.steps
                            .iter()
                            .map(|(s, _)| s.clone().into_inner())
                            .collect()
                    })
                    .unwrap_or_default();

                Self::build_process_tailwind_classes(
                    &id,
                    &child_step_ids,
                    entity_types,
                    theme_default,
                    theme_types_styles,
                )
            } else if is_process_step {
                // Find the parent process ID
                let parent_process_id = processes
                    .iter()
                    .find(|(_, pd)| pd.steps.iter().any(|(s, _)| s.as_str() == id.as_str()))
                    .map(|(p, _)| p.as_str().to_string());

                Self::build_process_step_tailwind_classes(
                    &id,
                    parent_process_id.as_deref(),
                    entity_types,
                    theme_default,
                    theme_types_styles,
                )
            } else {
                // Regular thing node
                Self::build_thing_tailwind_classes(
                    &id,
                    entity_types,
                    theme_default,
                    theme_types_styles,
                    &tag_thing_ids,
                    &thing_to_interacting_steps,
                )
            };

            tailwind_classes.insert(id, classes);
        }

        // Build classes for edge groups
        for (edge_group_id, _edges) in edge_groups.iter() {
            let id: Id = edge_group_id.clone().into_inner();

            // Get the process steps that interact with this edge group
            let interacting_steps = edge_group_to_steps.get(&id).cloned().unwrap_or_default();

            let classes = Self::build_edge_group_tailwind_classes(
                &id,
                entity_types,
                theme_default,
                theme_types_styles,
                &interacting_steps,
            );

            tailwind_classes.insert(id, classes);
        }

        tailwind_classes
    }

    /// Build a map of process step ID to (process ID, edge IDs they interact
    /// with).
    fn build_step_interactions_map(processes: &Processes) -> Map<Id, (Id, Vec<Id>)> {
        let mut step_interactions: Map<Id, (Id, Vec<Id>)> = Map::new();

        for (process_id, process_diagram) in processes.iter() {
            let process_id: Id = process_id.clone().into_inner();

            for (step_id, edge_ids) in process_diagram.step_thing_interactions.iter() {
                let step_id: Id = step_id.clone().into_inner();
                let edge_ids: Vec<Id> = edge_ids.iter().map(|e| e.clone().into_inner()).collect();
                step_interactions.insert(step_id, (process_id.clone(), edge_ids));
            }
        }

        step_interactions
    }

    /// Build a map of edge group ID to process steps that interact with it.
    fn build_edge_group_to_steps_map(processes: &Processes) -> Map<Id, Vec<Id>> {
        let mut edge_group_to_steps: Map<Id, Vec<Id>> = Map::new();

        for (_process_id, process_diagram) in processes.iter() {
            for (step_id, edge_ids) in process_diagram.step_thing_interactions.iter() {
                let step_id: Id = step_id.clone().into_inner();

                for edge_id in edge_ids.iter() {
                    let edge_id: Id = edge_id.clone().into_inner();
                    edge_group_to_steps
                        .entry(edge_id)
                        .or_default()
                        .push(step_id.clone());
                }
            }
        }

        edge_group_to_steps
    }

    /// Build a map of thing ID to process steps that interact with edges
    /// involving that thing.
    fn build_thing_to_interacting_steps_map(
        edge_groups: &EdgeGroups,
        step_interactions: &Map<Id, (Id, Vec<Id>)>,
    ) -> Map<Id, Vec<Id>> {
        let mut thing_to_steps: Map<Id, Vec<Id>> = Map::new();

        // For each process step and its edge interactions
        for (step_id, (_process_id, edge_group_ids)) in step_interactions.iter() {
            // For each edge group the step interacts with
            for edge_group_id in edge_group_ids.iter() {
                // Find the edge group and get its endpoints
                let edge_group_id_typed =
                    EdgeGroupId::from(Self::id_from_string(edge_group_id.as_str().to_string()));
                if let Some(edges) = edge_groups.get(&edge_group_id_typed) {
                    for edge in edges.iter() {
                        // Add this step to both the from and to things
                        let from_id: Id = edge.from.clone().into_inner();
                        let to_id: Id = edge.to.clone().into_inner();

                        thing_to_steps
                            .entry(from_id)
                            .or_default()
                            .push(step_id.clone());
                        thing_to_steps
                            .entry(to_id)
                            .or_default()
                            .push(step_id.clone());
                    }
                }
            }
        }

        // Deduplicate step IDs for each thing
        for steps in thing_to_steps.values_mut() {
            let mut seen = std::collections::HashSet::new();
            steps.retain(|id| seen.insert(id.as_str().to_string()));
        }

        thing_to_steps
    }

    /// Build tailwind classes for a tag node.
    fn build_tag_tailwind_classes(
        id: &Id,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
    ) -> String {
        let mut state = TailwindClassState::default();

        Self::resolve_tailwind_attrs(
            Some(id),
            entity_types,
            theme_default,
            theme_types_styles,
            true, // is_node
            &mut state,
        );

        let mut classes = state.to_classes_string(true);

        // Tags get peer/{id} class
        classes.push_str(&format!("\n\npeer/{}", id.as_str()));

        classes
    }

    /// Build tailwind classes for a process node.
    fn build_process_tailwind_classes(
        id: &Id,
        child_step_ids: &[Id],
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
    ) -> String {
        let mut state = TailwindClassState::default();

        Self::resolve_tailwind_attrs(
            Some(id),
            entity_types,
            theme_default,
            theme_types_styles,
            true, // is_node
            &mut state,
        );

        let mut classes = state.to_classes_string(true);

        // Processes get group/{id} class
        classes.push_str(&format!("\n\ngroup/{}", id.as_str()));

        // Processes get peer/{step_id} classes for each child process step
        // This is because process nodes are sibling elements to thing/edge_group
        // elements, whereas process step nodes are not siblings, so things and
        // edge_groups can only react to the process nodes' state for the
        // sibling selector to work.
        for step_id in child_step_ids {
            classes.push_str(&format!("\npeer/{}", step_id.as_str()));
        }

        classes
    }

    /// Build tailwind classes for a process step node.
    fn build_process_step_tailwind_classes(
        id: &Id,
        parent_process_id: Option<&str>,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
    ) -> String {
        let mut state = TailwindClassState::default();

        Self::resolve_tailwind_attrs(
            Some(id),
            entity_types,
            theme_default,
            theme_types_styles,
            true, // is_node
            &mut state,
        );

        let mut classes = state.to_classes_string(true);

        // Process steps get group-focus-within/{process_id}:visible class
        // Note: peer/{step_id} classes are placed on the parent process node instead,
        // because process nodes are sibling elements to thing/edge_group elements,
        // whereas process step nodes are not siblings.
        if let Some(process_id) = parent_process_id {
            classes.push_str(&format!("\n\ngroup-focus-within/{}:visible", process_id));
        }

        classes
    }

    /// Build tailwind classes for a regular thing node.
    fn build_thing_tailwind_classes(
        id: &Id,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
        tag_thing_ids: &Map<Id, Vec<Id>>,
        thing_to_interacting_steps: &Map<Id, Vec<Id>>,
    ) -> String {
        let mut state = TailwindClassState::default();

        Self::resolve_tailwind_attrs(
            Some(id),
            entity_types,
            theme_default,
            theme_types_styles,
            true, // is_node
            &mut state,
        );

        let mut classes = state.to_classes_string(true);

        // Add peer classes for tags that include this thing
        for (tag_id, thing_ids) in tag_thing_ids.iter() {
            if thing_ids.iter().any(|t| t.as_str() == id.as_str()) {
                // When a tag is focused, things within it get highlighted with shade_pale
                // Start with current state's colors but use shade_pale
                let tag_focus_state = TailwindClassState {
                    shape_color: state.shape_color.clone(),
                    fill_color: state.fill_color.clone(),
                    stroke_color: state.stroke_color.clone(),
                    // Apply shade_pale
                    fill_shade_hover: Some("50".to_string()),
                    fill_shade_normal: Some("100".to_string()),
                    fill_shade_focus: Some("200".to_string()),
                    fill_shade_active: Some("300".to_string()),
                    stroke_shade_hover: Some("100".to_string()),
                    stroke_shade_normal: Some("200".to_string()),
                    stroke_shade_focus: Some("300".to_string()),
                    stroke_shade_active: Some("400".to_string()),
                    ..Default::default()
                };

                let peer_prefix = format!("peer-[:focus-within]/{}:", tag_id.as_str());
                classes.push_str(&format!(
                    "\n\n{}animate-[stroke-dashoffset-move_2s_linear_infinite]",
                    peer_prefix
                ));
                classes.push_str(&tag_focus_state.to_peer_classes_string(&peer_prefix));
            }
        }

        // Add peer classes for process steps that interact with edges involving this
        // thing
        if let Some(interacting_steps) = thing_to_interacting_steps.get(id) {
            for step_id in interacting_steps.iter() {
                let peer_prefix = format!("peer-[:focus-within]/{}:", step_id.as_str());

                // Get the thing's color for interaction highlighting
                let color = state
                    .shape_color
                    .as_deref()
                    .or(state.fill_color.as_deref())
                    .unwrap_or("slate");

                classes.push_str(&format!(
                    "\n\n{}animate-[stroke-dashoffset-move_2s_linear_infinite]",
                    peer_prefix
                ));
                classes.push_str(&format!("\n{}stroke-{}-500", peer_prefix, color));
                classes.push_str(&format!("\n{}fill-{}-100", peer_prefix, color));
            }
        }

        classes
    }

    /// Build tailwind classes for an edge group.
    fn build_edge_group_tailwind_classes(
        id: &Id,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
        interacting_steps: &[Id],
    ) -> String {
        let mut state = TailwindClassState::default();

        Self::resolve_tailwind_attrs_for_edge(
            Some(id),
            entity_types,
            theme_default,
            theme_types_styles,
            &mut state,
        );

        let mut classes = state.to_classes_string(false);

        // Add peer classes for each process step that interacts with this edge
        for step_id in interacting_steps {
            let peer_prefix = format!("peer-[:focus-within]/{}:", step_id.as_str());

            // Interaction styling for edges
            classes.push_str(&format!(
                "\n\n{}animate-[stroke-dashoffset-move-request_2s_linear_infinite]",
                peer_prefix
            ));
            classes.push_str(&format!(
                "\n{}stroke-[dasharray:0,80,12,2,4,2,2,2,1,2,1,120]",
                peer_prefix
            ));
            classes.push_str(&format!("\n{}stroke-[2px]", peer_prefix));
            classes.push_str(&format!("\n{}visible", peer_prefix));

            // Use violet for interaction colors (as shown in example)
            classes.push_str(&format!("\n{}hover:fill-violet-600", peer_prefix));
            classes.push_str(&format!("\n{}fill-violet-700", peer_prefix));
            classes.push_str(&format!("\n{}focus:fill-violet-800", peer_prefix));
            classes.push_str(&format!("\n{}active:fill-violet-900", peer_prefix));
            classes.push_str(&format!("\n{}hover:stroke-violet-700", peer_prefix));
            classes.push_str(&format!("\n{}stroke-violet-800", peer_prefix));
            classes.push_str(&format!("\n{}focus:stroke-violet-900", peer_prefix));
            classes.push_str(&format!("\n{}active:stroke-violet-950", peer_prefix));
        }

        classes
    }

    /// Resolve tailwind attributes for a node.
    fn resolve_tailwind_attrs(
        node_id: Option<&Id>,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
        is_node: bool,
        state: &mut TailwindClassState,
    ) {
        let defaults_key = if is_node {
            IdOrDefaults::NodeDefaults
        } else {
            IdOrDefaults::EdgeDefaults
        };

        // 1. Start with NodeDefaults/EdgeDefaults (lowest priority)
        if let Some(defaults_partials) = theme_default.base_styles.get(&defaults_key) {
            Self::apply_tailwind_from_partials(
                defaults_partials,
                &theme_default.style_aliases,
                state,
            );
        }

        // 2. Apply EntityTypes in order (later types override earlier ones)
        if let Some(id) = node_id
            && let Some(types) = entity_types.get(id)
        {
            for entity_type in types.iter() {
                let type_id = disposition_model_common::entity::EntityTypeId::from(
                    Self::id_from_string(entity_type.as_str().to_string()),
                );
                if let Some(type_styles) = theme_types_styles.get(&type_id)
                    && let Some(type_partials) = type_styles.get(&defaults_key)
                {
                    Self::apply_tailwind_from_partials(
                        type_partials,
                        &theme_default.style_aliases,
                        state,
                    );
                }
            }
        }

        // 3. Apply node ID itself (highest priority)
        if let Some(id) = node_id
            && let Some(node_partials) =
                theme_default.base_styles.get(&IdOrDefaults::Id(id.clone()))
        {
            Self::apply_tailwind_from_partials(node_partials, &theme_default.style_aliases, state);
        }
    }

    /// Resolve tailwind attributes for an edge.
    fn resolve_tailwind_attrs_for_edge(
        edge_id: Option<&Id>,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
        state: &mut TailwindClassState,
    ) {
        // 1. Start with EdgeDefaults (lowest priority)
        if let Some(defaults_partials) = theme_default.base_styles.get(&IdOrDefaults::EdgeDefaults)
        {
            Self::apply_tailwind_from_partials(
                defaults_partials,
                &theme_default.style_aliases,
                state,
            );
        }

        // 2. Apply EntityTypes in order (later types override earlier ones)
        if let Some(id) = edge_id
            && let Some(types) = entity_types.get(id)
        {
            for entity_type in types.iter() {
                let type_id = disposition_model_common::entity::EntityTypeId::from(
                    Self::id_from_string(entity_type.as_str().to_string()),
                );
                if let Some(type_styles) = theme_types_styles.get(&type_id)
                    && let Some(type_partials) = type_styles.get(&IdOrDefaults::EdgeDefaults)
                {
                    Self::apply_tailwind_from_partials(
                        type_partials,
                        &theme_default.style_aliases,
                        state,
                    );
                }
            }
        }

        // 3. Apply edge ID itself (highest priority)
        if let Some(id) = edge_id
            && let Some(edge_partials) =
                theme_default.base_styles.get(&IdOrDefaults::Id(id.clone()))
        {
            Self::apply_tailwind_from_partials(edge_partials, &theme_default.style_aliases, state);
        }
    }

    /// Apply tailwind attribute values from CssClassPartials.
    fn apply_tailwind_from_partials(
        partials: &CssClassPartials,
        style_aliases: &StyleAliases,
        state: &mut TailwindClassState,
    ) {
        // First, check style_aliases_applied (lower priority within this partials)
        for alias in partials.style_aliases_applied() {
            if let Some(alias_partials) = style_aliases.get(alias) {
                Self::extract_tailwind_from_map(alias_partials, state);
            }
        }

        // Then, check direct attributes (higher priority within this partials)
        Self::extract_tailwind_from_map(partials, state);
    }

    /// Extract tailwind attribute values from a CssClassPartials map.
    fn extract_tailwind_from_map(partials: &CssClassPartials, state: &mut TailwindClassState) {
        // Visibility
        if let Some(value) = partials.get(&ThemeAttr::Visibility) {
            state.visibility = Some(value.clone());
        }

        // Stroke width
        if let Some(value) = partials.get(&ThemeAttr::StrokeWidth) {
            state.stroke_width = Some(value.clone());
        }

        // Stroke style - converts to stroke-dasharray
        if let Some(value) = partials.get(&ThemeAttr::StrokeStyle) {
            state.stroke_style = Some(value.clone());
        }
        if let Some(value) = partials.get(&ThemeAttr::StrokeStyleNormal) {
            state.stroke_style_normal = Some(value.clone());
        }
        if let Some(value) = partials.get(&ThemeAttr::StrokeStyleFocus) {
            state.stroke_style_focus = Some(value.clone());
        }
        if let Some(value) = partials.get(&ThemeAttr::StrokeStyleHover) {
            state.stroke_style_hover = Some(value.clone());
        }
        if let Some(value) = partials.get(&ThemeAttr::StrokeStyleActive) {
            state.stroke_style_active = Some(value.clone());
        }

        // Shape color (base for both fill and stroke)
        if let Some(value) = partials.get(&ThemeAttr::ShapeColor) {
            state.shape_color = Some(value.clone());
        }

        // Fill colors
        if let Some(value) = partials.get(&ThemeAttr::FillColor) {
            state.fill_color = Some(value.clone());
        }
        if let Some(value) = partials.get(&ThemeAttr::FillColorNormal) {
            state.fill_color_normal = Some(value.clone());
        }
        if let Some(value) = partials.get(&ThemeAttr::FillColorFocus) {
            state.fill_color_focus = Some(value.clone());
        }
        if let Some(value) = partials.get(&ThemeAttr::FillColorHover) {
            state.fill_color_hover = Some(value.clone());
        }
        if let Some(value) = partials.get(&ThemeAttr::FillColorActive) {
            state.fill_color_active = Some(value.clone());
        }

        // Fill shades
        if let Some(value) = partials.get(&ThemeAttr::FillShade) {
            state.fill_shade = Some(value.clone());
        }
        if let Some(value) = partials.get(&ThemeAttr::FillShadeNormal) {
            state.fill_shade_normal = Some(value.clone());
        }
        if let Some(value) = partials.get(&ThemeAttr::FillShadeFocus) {
            state.fill_shade_focus = Some(value.clone());
        }
        if let Some(value) = partials.get(&ThemeAttr::FillShadeHover) {
            state.fill_shade_hover = Some(value.clone());
        }
        if let Some(value) = partials.get(&ThemeAttr::FillShadeActive) {
            state.fill_shade_active = Some(value.clone());
        }

        // Stroke colors
        if let Some(value) = partials.get(&ThemeAttr::StrokeColor) {
            state.stroke_color = Some(value.clone());
        }
        if let Some(value) = partials.get(&ThemeAttr::StrokeColorNormal) {
            state.stroke_color_normal = Some(value.clone());
        }
        if let Some(value) = partials.get(&ThemeAttr::StrokeColorFocus) {
            state.stroke_color_focus = Some(value.clone());
        }
        if let Some(value) = partials.get(&ThemeAttr::StrokeColorHover) {
            state.stroke_color_hover = Some(value.clone());
        }
        if let Some(value) = partials.get(&ThemeAttr::StrokeColorActive) {
            state.stroke_color_active = Some(value.clone());
        }

        // Stroke shades
        if let Some(value) = partials.get(&ThemeAttr::StrokeShade) {
            state.stroke_shade = Some(value.clone());
        }
        if let Some(value) = partials.get(&ThemeAttr::StrokeShadeNormal) {
            state.stroke_shade_normal = Some(value.clone());
        }
        if let Some(value) = partials.get(&ThemeAttr::StrokeShadeFocus) {
            state.stroke_shade_focus = Some(value.clone());
        }
        if let Some(value) = partials.get(&ThemeAttr::StrokeShadeHover) {
            state.stroke_shade_hover = Some(value.clone());
        }
        if let Some(value) = partials.get(&ThemeAttr::StrokeShadeActive) {
            state.stroke_shade_active = Some(value.clone());
        }

        // Text
        if let Some(value) = partials.get(&ThemeAttr::TextColor) {
            state.text_color = Some(value.clone());
        }
        if let Some(value) = partials.get(&ThemeAttr::TextShade) {
            state.text_shade = Some(value.clone());
        }

        // Animation
        if let Some(value) = partials.get(&ThemeAttr::Animate) {
            state.animate = Some(value.clone());
        }
    }
}

/// State for accumulating resolved tailwind class attributes.
#[derive(Default)]
struct TailwindClassState {
    // Visibility
    visibility: Option<String>,
    // Stroke
    stroke_width: Option<String>,
    stroke_style: Option<String>,
    stroke_style_normal: Option<String>,
    stroke_style_focus: Option<String>,
    stroke_style_hover: Option<String>,
    stroke_style_active: Option<String>,
    // Colors - base
    shape_color: Option<String>,
    // Fill colors
    fill_color: Option<String>,
    fill_color_normal: Option<String>,
    fill_color_focus: Option<String>,
    fill_color_hover: Option<String>,
    fill_color_active: Option<String>,
    // Fill shades
    fill_shade: Option<String>,
    fill_shade_normal: Option<String>,
    fill_shade_focus: Option<String>,
    fill_shade_hover: Option<String>,
    fill_shade_active: Option<String>,
    // Stroke colors
    stroke_color: Option<String>,
    stroke_color_normal: Option<String>,
    stroke_color_focus: Option<String>,
    stroke_color_hover: Option<String>,
    stroke_color_active: Option<String>,
    // Stroke shades
    stroke_shade: Option<String>,
    stroke_shade_normal: Option<String>,
    stroke_shade_focus: Option<String>,
    stroke_shade_hover: Option<String>,
    stroke_shade_active: Option<String>,
    // Text
    text_color: Option<String>,
    text_shade: Option<String>,
    // Animation
    animate: Option<String>,
}

impl TailwindClassState {
    /// Convert stroke style to stroke-dasharray value.
    fn stroke_style_to_dasharray(style: &str) -> Option<&'static str> {
        match style {
            "solid" => Some("none"),
            "dashed" => Some("3"),
            "dotted" => Some("2"),
            _ => None,
        }
    }

    /// Get the resolved fill color for a state.
    fn get_fill_color(&self, state: FillStrokeState) -> &str {
        match state {
            FillStrokeState::Normal => self
                .fill_color_normal
                .as_deref()
                .or(self.fill_color.as_deref())
                .or(self.shape_color.as_deref())
                .unwrap_or("slate"),
            FillStrokeState::Focus => self
                .fill_color_focus
                .as_deref()
                .or(self.fill_color.as_deref())
                .or(self.shape_color.as_deref())
                .unwrap_or("slate"),
            FillStrokeState::Hover => self
                .fill_color_hover
                .as_deref()
                .or(self.fill_color.as_deref())
                .or(self.shape_color.as_deref())
                .unwrap_or("slate"),
            FillStrokeState::Active => self
                .fill_color_active
                .as_deref()
                .or(self.fill_color.as_deref())
                .or(self.shape_color.as_deref())
                .unwrap_or("slate"),
        }
    }

    /// Get the resolved fill shade for a state.
    fn get_fill_shade(&self, state: FillStrokeState) -> &str {
        match state {
            FillStrokeState::Normal => self
                .fill_shade_normal
                .as_deref()
                .or(self.fill_shade.as_deref())
                .unwrap_or("300"),
            FillStrokeState::Focus => self
                .fill_shade_focus
                .as_deref()
                .or(self.fill_shade.as_deref())
                .unwrap_or("400"),
            FillStrokeState::Hover => self
                .fill_shade_hover
                .as_deref()
                .or(self.fill_shade.as_deref())
                .unwrap_or("200"),
            FillStrokeState::Active => self
                .fill_shade_active
                .as_deref()
                .or(self.fill_shade.as_deref())
                .unwrap_or("500"),
        }
    }

    /// Get the resolved stroke color for a state.
    fn get_stroke_color(&self, state: FillStrokeState) -> &str {
        match state {
            FillStrokeState::Normal => self
                .stroke_color_normal
                .as_deref()
                .or(self.stroke_color.as_deref())
                .or(self.shape_color.as_deref())
                .unwrap_or("slate"),
            FillStrokeState::Focus => self
                .stroke_color_focus
                .as_deref()
                .or(self.stroke_color.as_deref())
                .or(self.shape_color.as_deref())
                .unwrap_or("slate"),
            FillStrokeState::Hover => self
                .stroke_color_hover
                .as_deref()
                .or(self.stroke_color.as_deref())
                .or(self.shape_color.as_deref())
                .unwrap_or("slate"),
            FillStrokeState::Active => self
                .stroke_color_active
                .as_deref()
                .or(self.stroke_color.as_deref())
                .or(self.shape_color.as_deref())
                .unwrap_or("slate"),
        }
    }

    /// Get the resolved stroke shade for a state.
    fn get_stroke_shade(&self, state: FillStrokeState) -> &str {
        match state {
            FillStrokeState::Normal => self
                .stroke_shade_normal
                .as_deref()
                .or(self.stroke_shade.as_deref())
                .unwrap_or("400"),
            FillStrokeState::Focus => self
                .stroke_shade_focus
                .as_deref()
                .or(self.stroke_shade.as_deref())
                .unwrap_or("500"),
            FillStrokeState::Hover => self
                .stroke_shade_hover
                .as_deref()
                .or(self.stroke_shade.as_deref())
                .unwrap_or("300"),
            FillStrokeState::Active => self
                .stroke_shade_active
                .as_deref()
                .or(self.stroke_shade.as_deref())
                .unwrap_or("600"),
        }
    }

    /// Convert state to tailwind classes string.
    fn to_classes_string(&self, is_node: bool) -> String {
        let mut classes = Vec::new();

        // Stroke dasharray from stroke_style
        if let Some(style) = &self.stroke_style
            && let Some(dasharray) = Self::stroke_style_to_dasharray(style)
        {
            classes.push(format!("[stroke-dasharray:{}]", dasharray));
        }

        // Stroke width
        if let Some(width) = &self.stroke_width {
            classes.push(format!("stroke-{}", width));
        }

        // Visibility
        if let Some(visibility) = &self.visibility {
            classes.push(visibility.clone());
        }

        // Fill classes (hover, normal, focus, active)
        classes.push(format!(
            "hover:fill-{}-{}",
            self.get_fill_color(FillStrokeState::Hover),
            self.get_fill_shade(FillStrokeState::Hover)
        ));
        classes.push(format!(
            "fill-{}-{}",
            self.get_fill_color(FillStrokeState::Normal),
            self.get_fill_shade(FillStrokeState::Normal)
        ));
        classes.push(format!(
            "focus:fill-{}-{}",
            self.get_fill_color(FillStrokeState::Focus),
            self.get_fill_shade(FillStrokeState::Focus)
        ));
        classes.push(format!(
            "active:fill-{}-{}",
            self.get_fill_color(FillStrokeState::Active),
            self.get_fill_shade(FillStrokeState::Active)
        ));

        // Stroke classes (hover, normal, focus, active)
        classes.push(format!(
            "hover:stroke-{}-{}",
            self.get_stroke_color(FillStrokeState::Hover),
            self.get_stroke_shade(FillStrokeState::Hover)
        ));
        classes.push(format!(
            "stroke-{}-{}",
            self.get_stroke_color(FillStrokeState::Normal),
            self.get_stroke_shade(FillStrokeState::Normal)
        ));
        classes.push(format!(
            "focus:stroke-{}-{}",
            self.get_stroke_color(FillStrokeState::Focus),
            self.get_stroke_shade(FillStrokeState::Focus)
        ));
        classes.push(format!(
            "active:stroke-{}-{}",
            self.get_stroke_color(FillStrokeState::Active),
            self.get_stroke_shade(FillStrokeState::Active)
        ));

        // Text classes (only for nodes)
        if is_node {
            let text_color = self.text_color.as_deref().unwrap_or("neutral");
            let text_shade = self.text_shade.as_deref().unwrap_or("900");
            classes.push(format!("[&>text]:fill-{}-{}", text_color, text_shade));
        }

        classes.join("\n")
    }

    /// Convert state to peer-prefixed classes string for tag/step highlighting.
    fn to_peer_classes_string(&self, prefix: &str) -> String {
        let mut classes = Vec::new();

        // Fill classes with peer prefix
        classes.push(format!(
            "\n{}hover:fill-{}-{}",
            prefix,
            self.get_fill_color(FillStrokeState::Hover),
            self.get_fill_shade(FillStrokeState::Hover)
        ));
        classes.push(format!(
            "\n{}fill-{}-{}",
            prefix,
            self.get_fill_color(FillStrokeState::Normal),
            self.get_fill_shade(FillStrokeState::Normal)
        ));
        classes.push(format!(
            "\n{}focus:fill-{}-{}",
            prefix,
            self.get_fill_color(FillStrokeState::Focus),
            self.get_fill_shade(FillStrokeState::Focus)
        ));
        classes.push(format!(
            "\n{}active:fill-{}-{}",
            prefix,
            self.get_fill_color(FillStrokeState::Active),
            self.get_fill_shade(FillStrokeState::Active)
        ));

        // Stroke classes with peer prefix
        classes.push(format!(
            "\n{}hover:stroke-{}-{}",
            prefix,
            self.get_stroke_color(FillStrokeState::Hover),
            self.get_stroke_shade(FillStrokeState::Hover)
        ));
        classes.push(format!(
            "\n{}stroke-{}-{}",
            prefix,
            self.get_stroke_color(FillStrokeState::Normal),
            self.get_stroke_shade(FillStrokeState::Normal)
        ));
        classes.push(format!(
            "\n{}focus:stroke-{}-{}",
            prefix,
            self.get_stroke_color(FillStrokeState::Focus),
            self.get_stroke_shade(FillStrokeState::Focus)
        ));
        classes.push(format!(
            "\n{}active:stroke-{}-{}",
            prefix,
            self.get_stroke_color(FillStrokeState::Active),
            self.get_stroke_shade(FillStrokeState::Active)
        ));

        classes.join("")
    }
}

/// States for fill and stroke colors.
#[derive(Clone, Copy)]
enum FillStrokeState {
    Normal,
    Focus,
    Hover,
    Active,
}
