use disposition_input_ir_model::IrDiagramAndIssues;
use disposition_input_model::{
    edge::{EdgeGroup as InputEdgeGroup, EdgeKind},
    entity::EntityTypes,
    process::Processes,
    tag::TagNames,
    theme::{ThemeDefault, ThemeTypesStyles},
    thing::{
        ThingCopyText, ThingDependencies, ThingHierarchy as InputThingHierarchy, ThingId,
        ThingInteractions, ThingNames,
    },
    InputDiagram,
};
use disposition_ir_model::{
    edge::{Edge, EdgeGroup, EdgeGroups},
    entity::EntityType,
    enum_iterator,
    layout::{FlexDirection, FlexLayout, NodeLayout, NodeLayouts},
    node::{NodeCopyText, NodeHierarchy, NodeId, NodeInbuilt, NodeNames, NodeOrdering, NodeShapes},
    process::ProcessStepEntities,
    IrDiagram,
};
use disposition_model_common::{
    edge::EdgeGroupId,
    entity::{EntityDescs, EntityTooltips},
    Id, Map, Set,
};

use self::{
    tailwind_classes_builder::TailwindClassesBuilder, theme_attr_resolver::ThemeAttrResolver,
};

mod tailwind_class_state;
mod tailwind_classes_builder;
mod theme_attr_resolver;

/// Maps an input diagram to an intermediate representation diagram.
#[derive(Clone, Copy, Debug)]
pub struct InputToIrDiagramMapper;

impl InputToIrDiagramMapper {
    /// Maps an input diagram to an intermediate representation diagram.
    pub fn map<'f, 'id>(input_diagram: &'f InputDiagram<'id>) -> IrDiagramAndIssues<'id>
    where
        'id: 'f,
    {
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
            entity_tooltips,
            entity_types,
            theme_default,
            theme_types_styles,
            theme_thing_dependencies_styles: _,
            theme_tag_things_focus,
            css,
        } = input_diagram;

        // 1. Build NodeNames from things, tags, processes, and process steps
        let nodes = Self::build_node_names(things, tags, processes);

        // 2. Build NodeCopyText from thing_copy_text
        let node_copy_text = Self::build_node_copy_text(thing_copy_text);

        // 3. Build NodeHierarchy from tags, processes (with steps), and thing_hierarchy
        let node_hierarchy = Self::build_node_hierarchy(tags, processes, thing_hierarchy);

        // 4. Build NodeOrdering from things, tags, and processes
        let node_ordering = Self::build_node_ordering(things, thing_hierarchy, tags, processes);

        // 5. Build EdgeGroups from thing_dependencies and thing_interactions
        let edge_groups = Self::build_edge_groups(thing_dependencies, thing_interactions);

        // 6. Build EntityDescs from input entity_descs
        let entity_descs = Self::build_entity_descs(entity_descs);

        // 7. Build EntityTooltips from input entity_tooltips
        let entity_tooltips = Self::build_entity_tooltips(entity_tooltips);

        // 8. Build EntityTypes with defaults for each node type
        let ir_entity_types = Self::build_entity_types(
            things,
            tags,
            processes,
            entity_types,
            thing_dependencies,
            thing_interactions,
        );

        // 9. Build NodeLayouts from node_hierarchy and theme
        let node_layouts = Self::build_node_layouts(
            &node_hierarchy,
            &ir_entity_types,
            theme_default,
            theme_types_styles,
            tags,
            processes,
        );

        // 10. Build NodeShapes from theme
        let node_shapes =
            Self::build_node_shapes(&nodes, &ir_entity_types, theme_default, theme_types_styles);

        // 11. Build TailwindClasses from theme
        let tailwind_classes = TailwindClassesBuilder::build(
            &nodes,
            &edge_groups,
            &ir_entity_types,
            theme_default,
            theme_types_styles,
            theme_tag_things_focus,
            tags,
            tag_things,
            processes,
        );

        // 12. Build ProcessStepEntities from step_thing_interactions
        let process_step_entities = Self::build_process_step_entities(processes);

        let diagram = IrDiagram {
            nodes,
            node_copy_text,
            node_hierarchy,
            node_ordering,
            edge_groups,
            entity_descs,
            entity_tooltips,
            entity_types: ir_entity_types,
            tailwind_classes,
            node_layouts,
            node_shapes,
            process_step_entities,
            css: css.clone(),
        };

        IrDiagramAndIssues { diagram, issues }
    }

    /// Creates an Id from a String.
    fn id_from_string(s: String) -> Id<'static> {
        // Use TryFrom<String> to create an Id from an owned String.
        // This will validate the ID format and create a Cow::Owned internally.
        Id::try_from(s).expect("valid ID string")
    }

    // === Node Names === //

    /// Build NodeNames from things, tags, processes, and process steps.
    fn build_node_names<'id>(
        things: &ThingNames<'id>,
        tags: &TagNames<'id>,
        processes: &Processes<'id>,
    ) -> NodeNames<'id> {
        // Add things
        let thing_nodes = things.iter().map(|(thing_id, name)| {
            let node_id = NodeId::from(thing_id.as_ref().clone());
            (node_id, name.clone())
        });

        // Add tags
        let tag_nodes = tags.iter().map(|(tag_id, name)| {
            let node_id = NodeId::from(tag_id.as_ref().clone());
            (node_id, name.clone())
        });

        // Add processes and their steps
        let process_and_step_nodes = processes.iter().flat_map(|(process_id, process_diagram)| {
            // Add process name
            let process_node_id = NodeId::from(process_id.as_ref().clone());
            let process_name = process_diagram
                .name
                .clone()
                .unwrap_or_else(|| process_id.as_str().to_string());

            // Add process steps
            let step_nodes = process_diagram.steps.iter().map(|(step_id, step_name)| {
                let step_node_id = NodeId::from(step_id.as_ref().clone());
                (step_node_id, step_name.clone())
            });

            std::iter::once((process_node_id, process_name)).chain(step_nodes)
        });

        thing_nodes
            .chain(tag_nodes)
            .chain(process_and_step_nodes)
            .collect()
    }

    // === Node Copy Text === //

    /// Build NodeCopyText from thing_copy_text.
    fn build_node_copy_text<'id>(thing_copy_text: &ThingCopyText<'id>) -> NodeCopyText<'id> {
        thing_copy_text
            .iter()
            .map(|(thing_id, text)| {
                let node_id = NodeId::from(thing_id.as_ref().clone());
                (node_id, text.clone())
            })
            .collect()
    }

    // === Node Hierarchy === //

    /// Build NodeHierarchy from tags, processes (with steps), and
    /// thing_hierarchy.
    fn build_node_hierarchy<'id>(
        tags: &TagNames<'id>,
        processes: &Processes<'id>,
        thing_hierarchy: &InputThingHierarchy<'id>,
    ) -> NodeHierarchy<'id> {
        // Add tags first (for CSS peer selector ordering)
        let tag_entries = tags.keys().map(|tag_id| {
            let node_id = NodeId::from(tag_id.as_ref().clone());
            (node_id, NodeHierarchy::new())
        });

        // Add processes with their steps
        let process_entries = processes.iter().map(|(process_id, process_diagram)| {
            let process_node_id = NodeId::from(process_id.as_ref().clone());
            let process_children: NodeHierarchy = process_diagram
                .steps
                .keys()
                .map(|step_id| {
                    let step_node_id = NodeId::from(step_id.as_ref().clone());
                    (step_node_id, NodeHierarchy::new())
                })
                .collect();

            (process_node_id, process_children)
        });

        // Add things hierarchy
        let thing_hierarchy = Self::convert_thing_hierarchy_to_node_hierarchy(thing_hierarchy);

        tag_entries
            .chain(process_entries)
            .chain(thing_hierarchy)
            .collect()
    }

    /// Recursively convert ThingHierarchy to NodeHierarchy.
    fn convert_thing_hierarchy_to_node_hierarchy<'id>(
        thing_hierarchy: &InputThingHierarchy<'id>,
    ) -> NodeHierarchy<'id> {
        thing_hierarchy
            .iter()
            .map(|(thing_id, children)| {
                let node_id = NodeId::from(thing_id.as_ref().clone());
                let child_hierarchy = Self::convert_thing_hierarchy_to_node_hierarchy(children);
                (node_id, child_hierarchy)
            })
            .collect()
    }

    // === Node Ordering === //

    /// Build NodeOrdering from things, tags, and processes.
    ///
    /// The map order defines the rendering order in the SVG:
    /// 1. Tags (for CSS peer selector ordering)
    /// 2. Processes (must come before process steps for peer styling)
    /// 3. Process steps
    /// 4. Things (in hierarchy order)
    ///
    /// The tab indices are calculated for keyboard navigation:
    /// 1. Things (starting from 1, in declaration order)
    /// 2. Processes and their steps (process first, then its steps)
    /// 3. Tags (at the end)
    fn build_node_ordering<'id>(
        things: &ThingNames<'id>,
        thing_hierarchy: &InputThingHierarchy<'id>,
        tags: &TagNames<'id>,
        processes: &Processes<'id>,
    ) -> NodeOrdering<'id> {
        // First, calculate tab indices in the user-expected order:
        // things, then processes with their steps, then tags
        let mut tab_index: u32 = 1;

        // Collect things tab indices in hierarchy order (depth-first)
        let mut tab_indices = Map::<&Id<'id>, u32>::new();
        Self::collect_thing_tab_indices_recursive(
            thing_hierarchy,
            &mut tab_index,
            &mut tab_indices,
        );

        // Collect process and step tab indices
        let mut process_step_count = 0;
        processes.iter().for_each(|(process_id, process_diagram)| {
            tab_indices.insert(process_id.as_ref(), tab_index);
            tab_index += 1;

            process_diagram.steps.keys().for_each(|step_id| {
                tab_indices.insert(step_id.as_ref(), tab_index);
                tab_index += 1;
            });

            process_step_count += process_diagram.steps.len();
        });

        // Collect tag tab indices
        tags.keys().for_each(|tag_id| {
            tab_indices.insert(tag_id.as_ref(), tab_index);
            tab_index += 1;
        });

        // Now build the NodeOrdering map in rendering order:
        // tags, then process steps, then processes, then things
        let mut node_ordering = NodeOrdering::with_capacity(
            tags.len() + processes.len() + process_step_count + things.len(),
        );

        // 1. Tags first (for CSS peer selector ordering)
        tags.keys().for_each(|tag_id| {
            let tab_idx = tab_indices.get(tag_id.as_ref()).copied().unwrap_or(0);
            let tag_node_id = NodeId::from(tag_id.as_ref().clone());
            node_ordering.insert(tag_node_id, tab_idx);
        });

        // 2. Processes (must come before process steps for peer styling)
        processes.keys().for_each(|process_id| {
            let process_node_id = NodeId::from(process_id.as_ref().clone());
            let tab_idx = tab_indices
                .get(process_node_id.as_ref())
                .copied()
                .unwrap_or(0);
            node_ordering.insert(process_node_id, tab_idx);
        });

        // 3. Process steps
        processes.values().for_each(|process_diagram| {
            process_diagram.steps.keys().for_each(|step_id| {
                let process_step_node_id = NodeId::from(step_id.as_ref().clone());
                let tab_idx = tab_indices
                    .get(process_step_node_id.as_ref())
                    .copied()
                    .unwrap_or(0);
                node_ordering.insert(process_step_node_id, tab_idx);
            });
        });

        // 4. Things (in hierarchy order)
        Self::add_things_to_ordering_recursive(thing_hierarchy, &tab_indices, &mut node_ordering);

        node_ordering
    }

    /// Recursively collect tab indices for things in hierarchy order.
    fn collect_thing_tab_indices_recursive<'f, 'id>(
        thing_hierarchy: &'f InputThingHierarchy<'id>,
        tab_index: &mut u32,
        tab_indices: &mut Map<&'f Id<'id>, u32>,
    ) {
        thing_hierarchy.iter().for_each(|(thing_id, children)| {
            tab_indices.insert(thing_id.as_ref(), *tab_index);
            *tab_index += 1;

            // Recurse into children
            Self::collect_thing_tab_indices_recursive(children, tab_index, tab_indices);
        });
    }

    /// Recursively add things to ordering in hierarchy order.
    fn add_things_to_ordering_recursive<'f, 'id>(
        thing_hierarchy: &'f InputThingHierarchy<'id>,
        thing_tab_indices: &Map<&'f Id<'id>, u32>,
        node_ordering: &mut NodeOrdering<'id>,
    ) {
        thing_hierarchy.iter().for_each(|(thing_id, children)| {
            let thing_node_id = NodeId::from(thing_id.as_ref().clone());
            let tab_idx = thing_tab_indices
                .get(thing_node_id.as_ref())
                .copied()
                .unwrap_or(0);
            node_ordering.insert(thing_node_id, tab_idx);

            // Recurse into children
            Self::add_things_to_ordering_recursive(children, thing_tab_indices, node_ordering);
        });
    }

    // === Edge Groups === //

    /// Build EdgeGroups from thing_dependencies and thing_interactions.
    fn build_edge_groups<'id>(
        thing_dependencies: &ThingDependencies<'id>,
        thing_interactions: &ThingInteractions<'id>,
    ) -> EdgeGroups<'id> {
        // Process thing_dependencies
        let dependency_entries =
            thing_dependencies
                .iter()
                .map(|(edge_group_id, input_edge_group)| {
                    (
                        edge_group_id.clone(),
                        Self::input_edge_group_to_edges(input_edge_group),
                    )
                });

        // Process thing_interactions (only add if not already present from
        // dependencies)
        let interaction_entries = thing_interactions
            .iter()
            .filter(|(edge_group_id, _)| !thing_dependencies.contains_key(edge_group_id))
            .map(|(edge_group_id, input_edge_group)| {
                (
                    edge_group_id.clone(),
                    Self::input_edge_group_to_edges(input_edge_group),
                )
            });

        dependency_entries.chain(interaction_entries).collect()
    }

    /// Convert an [`InputEdgeGroup`] to a list of [`Edge`]s.
    fn input_edge_group_to_edges<'id>(input_edge_group: &InputEdgeGroup<'id>) -> EdgeGroup<'id> {
        let things = &input_edge_group.things;
        let edges: Vec<Edge> = Self::edge_kind_to_edges(input_edge_group.kind, things);

        EdgeGroup::from(edges)
    }

    /// Convert an [`EdgeKind`] and a list of things to a list of [`Edge`]s.
    fn edge_kind_to_edges<'id>(edge_kind: EdgeKind, things: &[ThingId<'id>]) -> Vec<Edge<'id>> {
        match edge_kind {
            EdgeKind::Cyclic => {
                // Create edges from each thing to the next, and from last back to first
                things
                    .iter()
                    .enumerate()
                    .map(|(index, thing)| {
                        let from_id = NodeId::from(thing.as_ref().clone());
                        let to_idx = (index + 1) % things.len();
                        let to_id = NodeId::from(things[to_idx].as_ref().clone());
                        Edge::new(from_id, to_id)
                    })
                    .collect()
            }
            EdgeKind::Sequence => {
                // Create edges from each thing to the next (no cycle back)
                things
                    .windows(2)
                    .map(|pair| {
                        let from_id = NodeId::from(pair[0].as_ref().clone());
                        let to_id = NodeId::from(pair[1].as_ref().clone());
                        Edge::new(from_id, to_id)
                    })
                    .collect()
            }
            EdgeKind::Symmetric => {
                // Create edges from each thing to the next, then back from last to first
                // For [A, B, C]: A -> B -> C -> B -> A
                // For [A] (1 thing): A -> A (request), A -> A (response)
                if things.len() == 1 {
                    // Special case: 1 thing creates 2 self-loop edges (request and response)
                    let node_id = NodeId::from(things[0].as_ref().clone());
                    vec![
                        Edge::new(node_id.clone(), node_id.clone()),
                        Edge::new(node_id.clone(), node_id),
                    ]
                } else {
                    let forward: Vec<Edge> = things
                        .windows(2)
                        .map(|pair| {
                            let from_id = NodeId::from(pair[0].as_ref().clone());
                            let to_id = NodeId::from(pair[1].as_ref().clone());
                            Edge::new(from_id, to_id)
                        })
                        .collect();

                    let reverse: Vec<Edge> = things
                        .windows(2)
                        .rev()
                        .map(|pair| {
                            let from_id = NodeId::from(pair[1].as_ref().clone());
                            let to_id = NodeId::from(pair[0].as_ref().clone());
                            Edge::new(from_id, to_id)
                        })
                        .collect();

                    forward.into_iter().chain(reverse).collect()
                }
            }
        }
    }

    // === Entity Descs / Tooltips === //

    /// Build EntityDescs from input entity_descs.
    fn build_entity_descs<'id>(input_entity_descs: &EntityDescs<'id>) -> EntityDescs<'id> {
        // Copy existing entity descs
        input_entity_descs
            .iter()
            .map(|(id, desc)| (id.clone(), desc.clone()))
            .collect()
    }

    /// Build EntityTooltips from input entity_tooltips.
    fn build_entity_tooltips<'id>(
        input_entity_tooltips: &EntityTooltips<'id>,
    ) -> EntityTooltips<'id> {
        // Copy existing entity tooltips
        input_entity_tooltips
            .iter()
            .map(|(id, tooltip)| (id.clone(), tooltip.clone()))
            .collect()
    }

    // === Entity Types === //

    /// Build EntityTypes with defaults for each node type.
    fn build_entity_types<'id>(
        things: &ThingNames<'id>,
        tags: &TagNames<'id>,
        processes: &Processes<'id>,
        input_entity_types: &EntityTypes<'id>,
        thing_dependencies: &ThingDependencies<'id>,
        thing_interactions: &ThingInteractions<'id>,
    ) -> EntityTypes<'id> {
        // Helper to build types vector with default and optional custom types
        let build_types = |id: &Id<'id>, default_type: EntityType| {
            let mut types = Set::new();
            types.insert(default_type);
            if let Some(custom_types) = input_entity_types.get(id) {
                types.extend(custom_types.iter().cloned());
            }
            types
        };

        // Add things with type_thing_default + any custom type
        let thing_entries = things.keys().map(|thing_id| {
            let id: Id = thing_id.as_ref().clone();
            let types = build_types(&id, EntityType::ThingDefault);
            (id, types)
        });

        // Add tags with tag_type_default
        let tag_entries = tags.keys().map(|tag_id| {
            let id: Id = tag_id.as_ref().clone();
            let types = build_types(&id, EntityType::TagDefault);
            (id, types)
        });

        // Add processes with type_process_default and their steps
        let process_entries = processes.iter().flat_map(|(process_id, process_diagram)| {
            let process_id_inner: Id = process_id.as_ref().clone();
            let process_types = build_types(&process_id_inner, EntityType::ProcessDefault);

            // Add process steps with type_process_step_default
            let step_entries = process_diagram.steps.keys().map(|step_id| {
                let id: Id = step_id.as_ref().clone();
                let types = build_types(&id, EntityType::ProcessStepDefault);
                (id, types)
            });

            std::iter::once((process_id_inner, process_types)).chain(step_entries)
        });

        // node inbuilt types
        let node_inbuilt_types = enum_iterator::all::<NodeInbuilt>().map(|node_inbuilt| {
            let mut entity_types = Set::with_capacity(1);
            entity_types.insert(node_inbuilt.entity_type());

            (node_inbuilt.id(), entity_types)
        });

        let mut entity_types: Map<Id<'id>, Set<EntityType>> = node_inbuilt_types
            .chain(thing_entries)
            .chain(tag_entries)
            .chain(process_entries)
            .collect();

        // Add edge types from thing_dependencies
        Self::build_entity_types_dependencies(
            &mut entity_types,
            thing_dependencies,
            input_entity_types,
        );

        // Add edge types from thing_interactions (will merge with existing)
        Self::build_entity_types_interactions(
            &mut entity_types,
            thing_interactions,
            input_entity_types,
        );

        EntityTypes::from(entity_types)
    }

    /// Add edge types from dependencies.
    fn build_entity_types_dependencies<'id>(
        entity_types: &mut Map<Id<'id>, Set<EntityType>>,
        thing_deps: &ThingDependencies<'id>,
        input_entity_types: &EntityTypes<'id>,
    ) {
        let edge_group_entries = thing_deps
            .iter()
            .flat_map(|(edge_group_id, input_edge_group)| {
                let edge_kind = input_edge_group.kind;
                let things = &input_edge_group.things;

                // edge group entity types
                let edge_group_entity_types = Self::build_entity_types_for_edge_groups(
                    input_entity_types,
                    edge_group_id,
                    edge_kind,
                    Self::edge_group_default_type_dependency,
                );

                // edge entity types
                let edge_entity_types = Self::build_entity_types_for_edges(
                    input_entity_types,
                    edge_group_id,
                    edge_kind,
                    things,
                    Self::edge_default_type_dependency,
                );
                std::iter::once(edge_group_entity_types).chain(edge_entity_types)
            });

        entity_types.extend(edge_group_entries);
    }

    /// Add interaction types to existing edge types.
    fn build_entity_types_interactions<'id>(
        entity_types: &mut Map<Id<'id>, Set<EntityType>>,
        thing_interactions: &ThingInteractions<'id>,
        input_entity_types: &EntityTypes<'id>,
    ) {
        let edge_group_entries =
            thing_interactions
                .iter()
                .flat_map(|(edge_group_id, input_edge_group)| {
                    let edge_kind = input_edge_group.kind;
                    let things = &input_edge_group.things;

                    // edge group entity types
                    let edge_group_entity_types = Self::build_entity_types_for_edge_groups(
                        input_entity_types,
                        edge_group_id,
                        edge_kind,
                        Self::edge_group_default_type_interaction,
                    );

                    // edge entity types
                    let edge_entity_types = Self::build_entity_types_for_edges(
                        input_entity_types,
                        edge_group_id,
                        edge_kind,
                        things,
                        Self::edge_default_type_interaction,
                    );
                    std::iter::once(edge_group_entity_types).chain(edge_entity_types)
                });

        entity_types.extend(edge_group_entries);
    }

    fn build_entity_types_for_edge_groups<'id>(
        input_entity_types: &EntityTypes<'id>,
        edge_group_id: &EdgeGroupId<'id>,
        edge_kind: EdgeKind,
        edge_group_default_type_fn: fn(EdgeKind) -> EntityType,
    ) -> (Id<'id>, Set<EntityType>) {
        let edge_group_id: Id<'id> = edge_group_id.as_ref().clone();

        let edge_group_default_type = edge_group_default_type_fn(edge_kind);

        let mut types = Set::new();
        types.insert(edge_group_default_type);

        if let Some(custom_types) = input_entity_types.get(&edge_group_id) {
            types.extend(custom_types.iter().cloned());
        }

        (edge_group_id, types)
    }

    fn edge_group_default_type_dependency(edge_kind: EdgeKind) -> EntityType {
        match edge_kind {
            EdgeKind::Cyclic => EntityType::DependencyEdgeCyclicDefault,
            EdgeKind::Sequence => EntityType::DependencyEdgeSequenceDefault,
            EdgeKind::Symmetric => EntityType::DependencyEdgeSymmetricDefault,
        }
    }

    fn edge_default_type_dependency(
        edge_kind: EdgeKind,
        forward_count: usize,
        i: usize,
    ) -> EntityType {
        match edge_kind {
            EdgeKind::Cyclic => EntityType::DependencyEdgeCyclicForwardDefault,
            EdgeKind::Sequence => EntityType::DependencyEdgeSequenceForwardDefault,
            EdgeKind::Symmetric => {
                // First half are forward, second half are reverse
                if i < forward_count {
                    EntityType::DependencyEdgeSymmetricForwardDefault
                } else {
                    EntityType::DependencyEdgeSymmetricReverseDefault
                }
            }
        }
    }

    fn edge_group_default_type_interaction(edge_kind: EdgeKind) -> EntityType {
        match edge_kind {
            EdgeKind::Cyclic => EntityType::InteractionEdgeCyclicDefault,
            EdgeKind::Sequence => EntityType::InteractionEdgeSequenceDefault,
            EdgeKind::Symmetric => EntityType::InteractionEdgeSymmetricDefault,
        }
    }

    fn edge_default_type_interaction(
        edge_kind: EdgeKind,
        forward_count: usize,
        i: usize,
    ) -> EntityType {
        match edge_kind {
            EdgeKind::Cyclic => EntityType::InteractionEdgeCyclicForwardDefault,
            EdgeKind::Sequence => EntityType::InteractionEdgeSequenceForwardDefault,
            EdgeKind::Symmetric => {
                // First half are forward, second half are reverse
                if i < forward_count {
                    EntityType::InteractionEdgeSymmetricForwardDefault
                } else {
                    EntityType::InteractionEdgeSymmetricReverseDefault
                }
            }
        }
    }

    fn build_entity_types_for_edges<'id>(
        input_entity_types: &EntityTypes<'id>,
        edge_group_id: &EdgeGroupId<'id>,
        edge_kind: EdgeKind,
        things: &[ThingId<'id>],
        edge_default_type_fn: fn(EdgeKind, usize, usize) -> EntityType,
    ) -> impl Iterator<Item = (Id<'id>, Set<EntityType>)> {
        let (edge_count, forward_count) = match edge_kind {
            EdgeKind::Cyclic => (things.len(), things.len()),
            EdgeKind::Sequence => {
                let count = things.len().saturating_sub(1);
                (count, count)
            }
            EdgeKind::Symmetric => {
                // Forward edges + reverse edges
                // For 1 thing: 2 edges (1 request, 1 response)
                // For n things: (n-1) forward + (n-1) reverse
                let forward = things.len().max(1).saturating_sub(1).max(1);
                let total = if things.len() <= 1 { 2 } else { forward * 2 };
                (total, forward)
            }
        };

        (0..edge_count).map(move |i| {
            // Edge ID format: edge_group_id__index
            let edge_id_str = format!("{edge_group_id}__{i}");
            let edge_id = Self::id_from_string(edge_id_str);

            let edge_default_type = edge_default_type_fn(edge_kind, forward_count, i);

            let mut types = Set::new();
            types.insert(edge_default_type);

            if let Some(custom_types) = input_entity_types.get(&edge_id) {
                types.extend(custom_types.iter().cloned());
            }

            (edge_id, types)
        })
    }

    // === Node Layouts === //

    /// Build NodeLayouts from node_hierarchy and theme data.
    fn build_node_layouts<'id>(
        node_hierarchy: &NodeHierarchy<'id>,
        entity_types: &EntityTypes<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
        tags: &TagNames<'id>,
        processes: &Processes<'id>,
    ) -> NodeLayouts<'id> {
        let mut node_layouts = NodeLayouts::new();

        // Helper to determine if a node is a tag
        let is_tag = |node_id: &NodeId<'id>| tags.contains_key(node_id);

        // Helper to determine if a node is a process
        let is_process = |node_id: &NodeId<'id>| processes.contains_key(node_id);

        // 1. Add _root container layout
        let root_id = NodeInbuilt::Root.id();
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
        let things_and_processes_id = NodeInbuilt::ThingsAndProcessesContainer.id();
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
        let processes_container_id = NodeInbuilt::ProcessesContainer.id();
        let processes_container_layout = Self::build_container_layout(
            &processes_container_id,
            FlexDirection::Column,
            false,
            entity_types,
            theme_default,
            theme_types_styles,
        );
        node_layouts.insert(
            NodeId::from(processes_container_id),
            processes_container_layout,
        );

        // 4. Build layouts for all processes
        let process_layouts = processes.iter().flat_map(|(process_id, process_diagram)| {
            let process_node_id = NodeId::from(process_id.as_ref().clone());

            // Processes with steps get flex layout (column direction)
            let process_layout = if !process_diagram.steps.is_empty() {
                Self::build_node_flex_layout(
                    &process_node_id,
                    FlexDirection::Column,
                    false,
                    entity_types,
                    theme_default,
                    theme_types_styles,
                )
            } else {
                NodeLayout::None
            };

            // Process steps are always leaves (no children)
            let step_layouts = process_diagram.steps.keys().map(|step_id| {
                let step_node_id = NodeId::from(step_id.as_ref().clone());
                (step_node_id, NodeLayout::None)
            });

            std::iter::once((process_node_id, process_layout)).chain(step_layouts)
        });

        node_layouts.extend(process_layouts);

        // 5. Add _tags_container layout
        let tags_container_id = NodeInbuilt::TagsContainer.id();
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
        let tag_layouts = tags.keys().map(|tag_id| {
            let tag_node_id = NodeId::from(tag_id.as_ref().clone());
            (tag_node_id, NodeLayout::None)
        });

        node_layouts.extend(tag_layouts);

        // 7. Add _things_container layout
        let things_container_id = NodeInbuilt::ThingsContainer.id();
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
    fn build_container_layout<'id>(
        container_id: &Id<'id>,
        direction: FlexDirection,
        wrap: bool,
        entity_types: &EntityTypes<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
    ) -> NodeLayout {
        let (padding_top, padding_right, padding_bottom, padding_left) =
            ThemeAttrResolver::resolve_padding(
                Some(container_id),
                entity_types,
                theme_default,
                theme_types_styles,
            );
        let (margin_top, margin_right, margin_bottom, margin_left) =
            ThemeAttrResolver::resolve_margin(
                Some(container_id),
                entity_types,
                theme_default,
                theme_types_styles,
            );
        let gap = ThemeAttrResolver::resolve_gap(
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
    fn build_node_flex_layout<'id>(
        node_id: &NodeId<'id>,
        direction: FlexDirection,
        wrap: bool,
        entity_types: &EntityTypes<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
    ) -> NodeLayout {
        let id: Id<'id> = node_id.as_ref().clone();
        let (padding_top, padding_right, padding_bottom, padding_left) =
            ThemeAttrResolver::resolve_padding(
                Some(&id),
                entity_types,
                theme_default,
                theme_types_styles,
            );
        let (margin_top, margin_right, margin_bottom, margin_left) =
            ThemeAttrResolver::resolve_margin(
                Some(&id),
                entity_types,
                theme_default,
                theme_types_styles,
            );
        let gap = ThemeAttrResolver::resolve_gap(
            Some(&id),
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

    /// Recursively build layouts for things in the hierarchy.
    #[allow(clippy::too_many_arguments)] // we may reduce this during refactoring
    fn build_thing_layouts<'id, F, G>(
        hierarchy: &NodeHierarchy<'id>,
        depth: usize,
        entity_types: &EntityTypes<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
        node_layouts: &mut NodeLayouts<'id>,
        is_tag: &F,
        is_process: &G,
    ) where
        F: Fn(&NodeId<'id>) -> bool,
        G: Fn(&NodeId<'id>) -> bool,
    {
        let thing_layouts: Vec<_> = hierarchy
            .iter()
            // Skip tags and processes (already handled)
            .filter(|(node_id, _)| !is_tag(node_id) && !is_process(node_id))
            .flat_map(|(node_id, children)| {
                let layout = if children.is_empty() {
                    // Leaf node -- no layout needed
                    NodeLayout::None
                } else {
                    // Container node -- use flex layout
                    // Direction alternates based on depth: column at even depths, row at odd
                    // depths
                    let direction = if depth.is_multiple_of(2) {
                        FlexDirection::Column
                    } else {
                        FlexDirection::Row
                    };

                    Self::build_node_flex_layout(
                        node_id,
                        direction,
                        false,
                        entity_types,
                        theme_default,
                        theme_types_styles,
                    )
                };

                // Collect children info for recursive processing
                let children_to_process = if !children.is_empty() {
                    Some(children.clone())
                } else {
                    None
                };

                std::iter::once((node_id.clone(), layout, children_to_process))
            })
            .collect();

        // Insert layouts and recursively process children
        thing_layouts
            .into_iter()
            .for_each(|(node_id, layout, children_opt)| {
                node_layouts.insert(node_id, layout);

                if let Some(children) = children_opt {
                    Self::build_thing_layouts(
                        &children,
                        depth + 1,
                        entity_types,
                        theme_default,
                        theme_types_styles,
                        node_layouts,
                        is_tag,
                        is_process,
                    );
                }
            });
    }

    // === Node Shapes === //

    /// Build NodeShapes for all nodes from theme data.
    ///
    /// This extracts the corner radius values from the theme configuration
    /// for each node and creates a `NodeShape` (currently `Rect` with corner
    /// radii).
    fn build_node_shapes<'id>(
        nodes: &NodeNames<'id>,
        entity_types: &EntityTypes<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
    ) -> NodeShapes<'id> {
        nodes
            .iter()
            .map(|(node_id, _name)| {
                let id: Id<'id> = node_id.as_ref().clone();
                let shape = ThemeAttrResolver::resolve_node_shape(
                    &id,
                    entity_types,
                    theme_default,
                    theme_types_styles,
                );
                (node_id.clone(), shape)
            })
            .collect()
    }

    // === Process Step Entities === //

    /// Build [`ProcessStepEntities`] from the process step thing interactions.
    ///
    /// For each process step, collects the edge group IDs it interacts with
    /// and stores them as `Id`s keyed by the process step's `NodeId`.
    fn build_process_step_entities<'id>(processes: &Processes<'id>) -> ProcessStepEntities<'id> {
        processes
            .iter()
            .flat_map(|(_process_id, process_diagram)| {
                process_diagram
                    .step_thing_interactions
                    .iter()
                    .map(|(step_id, edge_group_ids)| {
                        let node_id = NodeId::from(step_id.as_ref().clone());
                        let entity_ids: Vec<Id<'id>> = edge_group_ids
                            .iter()
                            .map(|edge_group_id| edge_group_id.as_ref().clone())
                            .collect();
                        (node_id, entity_ids)
                    })
            })
            .collect()
    }
}
