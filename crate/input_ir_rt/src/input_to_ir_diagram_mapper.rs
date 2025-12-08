use disposition_input_ir_model::IrDiagramAndIssues;
use disposition_input_model::{
    edge::EdgeKind,
    theme::{
        CssClassPartials, IdOrDefaults, StyleAliases, ThemeAttr, ThemeDefault, ThemeTypesStyles,
    },
    thing::ThingHierarchy as InputThingHierarchy,
    InputDiagram,
};
use disposition_ir_model::{
    edge::{Edge, EdgeGroup, EdgeGroupId, EdgeGroups},
    entity::{EntityType, EntityTypes as IrEntityTypes},
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
            tag_things: _,
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

        let diagram = IrDiagram {
            nodes,
            node_copy_text,
            node_hierarchy,
            edge_groups,
            entity_descs,
            entity_types: ir_entity_types,
            tailwind_classes: Default::default(), // Done later
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

    /// Resolve padding values from theme with priority order:
    /// 1. Node ID itself (highest)
    /// 2. EntityTypes (reverse order)
    /// 3. NodeDefaults (lowest)
    /// 4. Default 0.0f32
    fn resolve_padding(
        node_id: Option<&Id>,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
    ) -> (f32, f32, f32, f32) {
        let mut padding_top: Option<f32> = None;
        let mut padding_right: Option<f32> = None;
        let mut padding_bottom: Option<f32> = None;
        let mut padding_left: Option<f32> = None;

        // 1. Start with NodeDefaults (lowest priority)
        if let Some(node_defaults_partials) =
            theme_default.base_styles.get(&IdOrDefaults::NodeDefaults)
        {
            Self::apply_padding_from_partials(
                node_defaults_partials,
                &theme_default.style_aliases,
                &mut padding_top,
                &mut padding_right,
                &mut padding_bottom,
                &mut padding_left,
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
                    && let Some(type_partials) = type_styles.get(&IdOrDefaults::NodeDefaults)
                {
                    Self::apply_padding_from_partials(
                        type_partials,
                        &theme_default.style_aliases,
                        &mut padding_top,
                        &mut padding_right,
                        &mut padding_bottom,
                        &mut padding_left,
                    );
                }
            }
        }

        // 3. Apply node ID itself (highest priority)
        if let Some(id) = node_id
            && let Some(node_partials) =
                theme_default.base_styles.get(&IdOrDefaults::Id(id.clone()))
        {
            Self::apply_padding_from_partials(
                node_partials,
                &theme_default.style_aliases,
                &mut padding_top,
                &mut padding_right,
                &mut padding_bottom,
                &mut padding_left,
            );
        }

        (
            padding_top.unwrap_or(0.0),
            padding_right.unwrap_or(0.0),
            padding_bottom.unwrap_or(0.0),
            padding_left.unwrap_or(0.0),
        )
    }

    /// Apply padding values from CssClassPartials, checking both direct
    /// attributes and style aliases.
    fn apply_padding_from_partials(
        partials: &CssClassPartials,
        style_aliases: &StyleAliases,
        padding_top: &mut Option<f32>,
        padding_right: &mut Option<f32>,
        padding_bottom: &mut Option<f32>,
        padding_left: &mut Option<f32>,
    ) {
        // First, check style_aliases_applied (lower priority within this partials)
        for alias in partials.style_aliases_applied() {
            if let Some(alias_partials) = style_aliases.get(alias) {
                Self::extract_padding_from_map(
                    alias_partials,
                    padding_top,
                    padding_right,
                    padding_bottom,
                    padding_left,
                );
            }
        }

        // Then, check direct attributes (higher priority within this partials)
        Self::extract_padding_from_map(
            partials,
            padding_top,
            padding_right,
            padding_bottom,
            padding_left,
        );
    }

    /// Extract padding values from a map of ThemeAttr to String.
    fn extract_padding_from_map(
        partials: &CssClassPartials,
        padding_top: &mut Option<f32>,
        padding_right: &mut Option<f32>,
        padding_bottom: &mut Option<f32>,
        padding_left: &mut Option<f32>,
    ) {
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

    /// Resolve margin values from theme with priority order:
    /// 1. Node ID itself (highest)
    /// 2. EntityTypes (reverse order)
    /// 3. NodeDefaults (lowest)
    /// 4. Default 0.0f32
    fn resolve_margin(
        node_id: Option<&Id>,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
    ) -> (f32, f32, f32, f32) {
        let mut margin_top: Option<f32> = None;
        let mut margin_right: Option<f32> = None;
        let mut margin_bottom: Option<f32> = None;
        let mut margin_left: Option<f32> = None;

        // 1. Start with NodeDefaults (lowest priority)
        if let Some(node_defaults_partials) =
            theme_default.base_styles.get(&IdOrDefaults::NodeDefaults)
        {
            Self::apply_margin_from_partials(
                node_defaults_partials,
                &theme_default.style_aliases,
                &mut margin_top,
                &mut margin_right,
                &mut margin_bottom,
                &mut margin_left,
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
                    && let Some(type_partials) = type_styles.get(&IdOrDefaults::NodeDefaults)
                {
                    Self::apply_margin_from_partials(
                        type_partials,
                        &theme_default.style_aliases,
                        &mut margin_top,
                        &mut margin_right,
                        &mut margin_bottom,
                        &mut margin_left,
                    );
                }
            }
        }

        // 3. Apply node ID itself (highest priority)
        if let Some(id) = node_id
            && let Some(node_partials) =
                theme_default.base_styles.get(&IdOrDefaults::Id(id.clone()))
        {
            Self::apply_margin_from_partials(
                node_partials,
                &theme_default.style_aliases,
                &mut margin_top,
                &mut margin_right,
                &mut margin_bottom,
                &mut margin_left,
            );
        }

        (
            margin_top.unwrap_or(0.0),
            margin_right.unwrap_or(0.0),
            margin_bottom.unwrap_or(0.0),
            margin_left.unwrap_or(0.0),
        )
    }

    /// Apply margin values from CssClassPartials, checking both direct
    /// attributes and style aliases.
    fn apply_margin_from_partials(
        partials: &CssClassPartials,
        style_aliases: &StyleAliases,
        margin_top: &mut Option<f32>,
        margin_right: &mut Option<f32>,
        margin_bottom: &mut Option<f32>,
        margin_left: &mut Option<f32>,
    ) {
        // First, check style_aliases_applied (lower priority within this partials)
        for alias in partials.style_aliases_applied() {
            if let Some(alias_partials) = style_aliases.get(alias) {
                Self::extract_margin_from_map(
                    alias_partials,
                    margin_top,
                    margin_right,
                    margin_bottom,
                    margin_left,
                );
            }
        }

        // Then, check direct attributes (higher priority within this partials)
        Self::extract_margin_from_map(
            partials,
            margin_top,
            margin_right,
            margin_bottom,
            margin_left,
        );
    }

    /// Extract margin values from a map of ThemeAttr to String.
    fn extract_margin_from_map(
        partials: &CssClassPartials,
        margin_top: &mut Option<f32>,
        margin_right: &mut Option<f32>,
        margin_bottom: &mut Option<f32>,
        margin_left: &mut Option<f32>,
    ) {
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

    /// Resolve gap value from theme with priority order:
    /// 1. Node ID itself (highest)
    /// 2. EntityTypes (reverse order)
    /// 3. NodeDefaults (lowest)
    /// 4. Default 0.0f32
    fn resolve_gap(
        node_id: Option<&Id>,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
    ) -> f32 {
        let mut gap: Option<f32> = None;

        // 1. Start with NodeDefaults (lowest priority)
        if let Some(node_defaults_partials) =
            theme_default.base_styles.get(&IdOrDefaults::NodeDefaults)
        {
            Self::apply_gap_from_partials(
                node_defaults_partials,
                &theme_default.style_aliases,
                &mut gap,
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
                    && let Some(type_partials) = type_styles.get(&IdOrDefaults::NodeDefaults)
                {
                    Self::apply_gap_from_partials(
                        type_partials,
                        &theme_default.style_aliases,
                        &mut gap,
                    );
                }
            }
        }

        // 3. Apply node ID itself (highest priority)
        if let Some(id) = node_id
            && let Some(node_partials) =
                theme_default.base_styles.get(&IdOrDefaults::Id(id.clone()))
        {
            Self::apply_gap_from_partials(node_partials, &theme_default.style_aliases, &mut gap);
        }

        gap.unwrap_or(0.0)
    }

    /// Apply gap value from CssClassPartials, checking both direct attributes
    /// and style aliases.
    fn apply_gap_from_partials(
        partials: &CssClassPartials,
        style_aliases: &StyleAliases,
        gap: &mut Option<f32>,
    ) {
        // First, check style_aliases_applied (lower priority within this partials)
        for alias in partials.style_aliases_applied() {
            if let Some(alias_partials) = style_aliases.get(alias)
                && let Some(value) = alias_partials.get(&ThemeAttr::Gap)
                && let Ok(v) = value.parse::<f32>()
            {
                *gap = Some(v);
            }
        }

        // Then, check direct attribute (higher priority within this partials)
        if let Some(value) = partials.get(&ThemeAttr::Gap)
            && let Ok(v) = value.parse::<f32>()
        {
            *gap = Some(v);
        }
    }
}
