use disposition_input_ir_model::IrDiagramAndIssues;
use disposition_input_model::{
    edge::EdgeKind, thing::ThingHierarchy as InputThingHierarchy, InputDiagram,
};
use disposition_ir_model::{
    edge::{Edge, EdgeGroup, EdgeGroupId, EdgeGroups},
    entity::{EntityType, EntityTypes as IrEntityTypes},
    node::{NodeCopyText, NodeHierarchy, NodeId, NodeNames},
    IrDiagram,
};
use disposition_model_common::{entity::EntityDescs, Id, Map};

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
            theme_default: _,
            theme_types_styles: _,
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

        let diagram = IrDiagram {
            nodes,
            node_copy_text,
            node_hierarchy,
            edge_groups,
            entity_descs,
            entity_types: ir_entity_types,
            tailwind_classes: Default::default(), // Done later
            node_layout: Default::default(),      // Done later
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
}
