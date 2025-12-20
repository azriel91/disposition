use std::{borrow::Cow, fmt::Write};

use disposition_input_ir_model::IrDiagramAndIssues;
use disposition_input_model::{
    edge::EdgeKind,
    entity::EntityTypes,
    process::{ProcessId, ProcessStepId, Processes},
    tag::{TagNames, TagThings},
    theme::{
        CssClassPartials, IdOrDefaults, StyleAliases, TagIdOrDefaults, ThemeAttr, ThemeDefault,
        ThemeTagThingsFocus, ThemeTypesStyles,
    },
    thing::{
        ThingCopyText, ThingDependencies, ThingHierarchy as InputThingHierarchy, ThingInteractions,
        ThingNames,
    },
    InputDiagram,
};
use disposition_ir_model::{
    edge::{Edge, EdgeGroup, EdgeGroups},
    entity::{EntityTailwindClasses, EntityType, EntityTypeId, EntityTypes as IrEntityTypes},
    layout::{FlexDirection, FlexLayout, NodeLayout, NodeLayouts},
    node::{NodeCopyText, NodeHierarchy, NodeId, NodeNames},
    IrDiagram,
};
use disposition_model_common::{edge::EdgeGroupId, entity::EntityDescs, id, Id, Keys, Map, Set};

/// Maps an input diagram to an intermediate representation diagram.
#[derive(Clone, Copy, Debug)]
pub struct InputToIrDiagramMapper;

const CLASSES_BUFFER_WRITE_FAIL: &str = "Failed to write string to buffer";

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
            theme_tag_things_focus,
            css,
        } = input_diagram;

        // 1. Build NodeNames from things, tags, processes, and process steps
        let nodes = Self::build_node_names(&things, &tags, &processes);

        // 2. Build NodeCopyText from thing_copy_text
        let node_copy_text = Self::build_node_copy_text(thing_copy_text);

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
            &theme_tag_things_focus,
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
    fn build_node_names(things: &ThingNames, tags: &TagNames, processes: &Processes) -> NodeNames {
        // Add things
        let thing_nodes = things.iter().map(|(thing_id, name)| {
            let node_id = NodeId::from(thing_id.clone().into_inner());
            (node_id, name.clone())
        });

        // Add tags
        let tag_nodes = tags.iter().map(|(tag_id, name)| {
            let node_id = NodeId::from(tag_id.clone().into_inner());
            (node_id, name.clone())
        });

        // Add processes and their steps
        let process_and_step_nodes = processes.iter().flat_map(|(process_id, process_diagram)| {
            // Add process name
            let process_node_id = NodeId::from(process_id.clone().into_inner());
            let process_name = process_diagram
                .name
                .clone()
                .unwrap_or_else(|| process_id.as_str().to_string());

            // Add process steps
            let step_nodes = process_diagram.steps.iter().map(|(step_id, step_name)| {
                let step_node_id = NodeId::from(step_id.clone().into_inner());
                (step_node_id, step_name.clone())
            });

            std::iter::once((process_node_id, process_name)).chain(step_nodes)
        });

        thing_nodes
            .chain(tag_nodes)
            .chain(process_and_step_nodes)
            .collect()
    }

    /// Build NodeCopyText from thing_copy_text.
    fn build_node_copy_text(thing_copy_text: ThingCopyText) -> NodeCopyText {
        thing_copy_text
            .into_inner()
            .into_iter()
            .map(|(thing_id, text)| {
                let node_id = NodeId::from(thing_id.into_inner());
                (node_id, text)
            })
            .collect()
    }

    /// Build NodeHierarchy from tags, processes (with steps), and
    /// thing_hierarchy.
    fn build_node_hierarchy(
        tags: &TagNames,
        processes: &Processes,
        thing_hierarchy: &InputThingHierarchy,
    ) -> NodeHierarchy {
        // Add tags first (for CSS peer selector ordering)
        let tag_entries = tags.keys().map(|tag_id| {
            let node_id = NodeId::from(tag_id.clone().into_inner());
            (node_id, NodeHierarchy::new())
        });

        // Add processes with their steps
        let process_entries = processes.iter().map(|(process_id, process_diagram)| {
            let process_node_id = NodeId::from(process_id.clone().into_inner());
            let process_children: NodeHierarchy = process_diagram
                .steps
                .keys()
                .map(|step_id| {
                    let step_node_id = NodeId::from(step_id.clone().into_inner());
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
    fn convert_thing_hierarchy_to_node_hierarchy(
        thing_hierarchy: &InputThingHierarchy,
    ) -> NodeHierarchy {
        thing_hierarchy
            .iter()
            .map(|(thing_id, children)| {
                let node_id = NodeId::from(thing_id.clone().into_inner());
                let child_hierarchy = Self::convert_thing_hierarchy_to_node_hierarchy(children);
                (node_id, child_hierarchy)
            })
            .collect()
    }

    /// Build EdgeGroups from thing_dependencies and thing_interactions.
    fn build_edge_groups(
        thing_dependencies: &ThingDependencies,
        thing_interactions: &ThingInteractions,
    ) -> EdgeGroups {
        // Process thing_dependencies
        let dependency_entries = thing_dependencies.iter().map(|(edge_group_id, edge_kind)| {
            (edge_group_id.clone(), Self::edge_kind_to_edges(edge_kind))
        });

        // Process thing_interactions (only add if not already present from
        // dependencies)
        let interaction_entries = thing_interactions
            .iter()
            .filter(|(edge_group_id, _)| !thing_dependencies.contains_key(edge_group_id))
            .map(|(edge_group_id, edge_kind)| {
                (edge_group_id.clone(), Self::edge_kind_to_edges(edge_kind))
            });

        dependency_entries.chain(interaction_entries).collect()
    }

    /// Convert an EdgeKind to a list of Edges.
    fn edge_kind_to_edges(edge_kind: &EdgeKind) -> EdgeGroup {
        let edges: Vec<Edge> = match edge_kind {
            EdgeKind::Cyclic(things) => {
                // Create edges from each thing to the next, and from last back to first
                things
                    .iter()
                    .enumerate()
                    .map(|(index, thing)| {
                        let from_id = NodeId::from(thing.clone().into_inner());
                        let to_idx = (index + 1) % things.len();
                        let to_id = NodeId::from(things[to_idx].clone().into_inner());
                        Edge::new(from_id, to_id)
                    })
                    .collect()
            }
            EdgeKind::Sequence(things) => {
                // Create edges from each thing to the next (no cycle back)
                things
                    .windows(2)
                    .map(|pair| {
                        let from_id = NodeId::from(pair[0].clone().into_inner());
                        let to_id = NodeId::from(pair[1].clone().into_inner());
                        Edge::new(from_id, to_id)
                    })
                    .collect()
            }
            EdgeKind::Symmetric(things) => {
                // Create edges from each thing to the next, then back from last to first
                // For [A, B, C]: A -> B -> C -> B -> A
                // For [A] (1 thing): A -> A (request), A -> A (response)
                if things.len() == 1 {
                    // Special case: 1 thing creates 2 self-loop edges (request and response)
                    let node_id = NodeId::from(things[0].clone().into_inner());
                    vec![
                        Edge::new(node_id.clone(), node_id.clone()),
                        Edge::new(node_id.clone(), node_id),
                    ]
                } else {
                    let forward: Vec<Edge> = things
                        .windows(2)
                        .map(|pair| {
                            let from_id = NodeId::from(pair[0].clone().into_inner());
                            let to_id = NodeId::from(pair[1].clone().into_inner());
                            Edge::new(from_id, to_id)
                        })
                        .collect();

                    let reverse: Vec<Edge> = things
                        .windows(2)
                        .rev()
                        .map(|pair| {
                            let from_id = NodeId::from(pair[1].clone().into_inner());
                            let to_id = NodeId::from(pair[0].clone().into_inner());
                            Edge::new(from_id, to_id)
                        })
                        .collect();

                    forward.into_iter().chain(reverse).collect()
                }
            }
        };

        EdgeGroup::from(edges)
    }

    /// Build EntityDescs from input entity_descs and process step_descs.
    fn build_entity_descs(input_entity_descs: &EntityDescs, processes: &Processes) -> EntityDescs {
        // Copy existing entity descs
        let existing_entries = input_entity_descs
            .iter()
            .map(|(id, desc)| (id.clone(), desc.clone()));

        // Add process step descriptions
        let step_entries = processes.values().flat_map(|process_diagram| {
            process_diagram.step_descs.iter().map(|(step_id, desc)| {
                let id: Id = step_id.clone().into_inner();
                (id, desc.clone())
            })
        });

        existing_entries.chain(step_entries).collect()
    }

    /// Build EntityTypes with defaults for each node type.
    fn build_entity_types(
        things: &ThingNames,
        tags: &TagNames,
        processes: &Processes,
        input_entity_types: &EntityTypes,
        thing_dependencies: &ThingDependencies,
        thing_interactions: &ThingInteractions,
    ) -> IrEntityTypes {
        // Helper to build types vector with default and optional custom type
        let build_types = |id: &Id, default_type: EntityType| {
            let mut types = Set::new();
            types.insert(default_type);
            if let Some(custom_type) = input_entity_types.get(id) {
                types.insert(EntityType::from(custom_type.clone().into_inner()));
            }
            types
        };

        // Add things with type_thing_default + any custom type
        let thing_entries = things.keys().map(|thing_id| {
            let id: Id = thing_id.clone().into_inner();
            let types = build_types(&id, EntityType::ThingDefault);
            (id, types)
        });

        // Add tags with tag_type_default
        let tag_entries = tags.keys().map(|tag_id| {
            let id: Id = tag_id.clone().into_inner();
            let types = build_types(&id, EntityType::TagDefault);
            (id, types)
        });

        // Add processes with type_process_default and their steps
        let process_entries = processes.iter().flat_map(|(process_id, process_diagram)| {
            let process_id_inner: Id = process_id.clone().into_inner();
            let process_types = build_types(&process_id_inner, EntityType::ProcessDefault);

            // Add process steps with type_process_step_default
            let step_entries = process_diagram.steps.keys().map(|step_id| {
                let id: Id = step_id.clone().into_inner();
                let types = build_types(&id, EntityType::ProcessStepDefault);
                (id, types)
            });

            std::iter::once((process_id_inner, process_types)).chain(step_entries)
        });

        let mut entity_types: Map<Id, Set<EntityType>> = thing_entries
            .chain(tag_entries)
            .chain(process_entries)
            .collect();

        // Add edge types from thing_dependencies
        Self::add_edge_types(&mut entity_types, thing_dependencies, input_entity_types);

        // Add edge types from thing_interactions (will merge with existing)
        Self::add_edge_interaction_types(&mut entity_types, thing_interactions, input_entity_types);

        IrEntityTypes::from(entity_types)
    }

    /// Add edge types from dependencies.
    fn add_edge_types(
        entity_types: &mut Map<Id, Set<EntityType>>,
        thing_deps: &ThingDependencies,
        input_entity_types: &EntityTypes,
    ) {
        let edge_entries = thing_deps.iter().flat_map(|(edge_group_id, edge_kind)| {
            let (edge_count, forward_count) = match edge_kind {
                EdgeKind::Cyclic(things) => (things.len(), things.len()),
                EdgeKind::Sequence(things) => {
                    let count = things.len().saturating_sub(1);
                    (count, count)
                }
                EdgeKind::Symmetric(things) => {
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

                let default_type = match edge_kind {
                    EdgeKind::Cyclic(_) => EntityType::EdgeDependencyCyclicDefault,
                    EdgeKind::Sequence(_) => EntityType::EdgeDependencySequenceDefault,
                    EdgeKind::Symmetric(_) => {
                        // First half are forward (request), second half are reverse (response)
                        if i < forward_count {
                            EntityType::EdgeDependencySymmetricRequestDefault
                        } else {
                            EntityType::EdgeDependencySymmetricResponseDefault
                        }
                    }
                };

                let mut types = Set::new();
                types.insert(default_type);

                if let Some(custom_type) = input_entity_types.get(&edge_id) {
                    types.insert(EntityType::from(custom_type.clone().into_inner()));
                }

                (edge_id, types)
            })
        });

        entity_types.extend(edge_entries);
    }

    /// Add interaction types to existing edge types.
    fn add_edge_interaction_types(
        entity_types: &mut Map<Id, Set<EntityType>>,
        thing_interactions: &ThingInteractions,
        _input_entity_types: &EntityTypes,
    ) {
        thing_interactions
            .iter()
            .flat_map(|(edge_group_id, edge_kind)| {
                let (edge_count, forward_count) = match edge_kind {
                    EdgeKind::Cyclic(things) => (things.len(), things.len()),
                    EdgeKind::Sequence(things) => {
                        let count = things.len().saturating_sub(1);
                        (count, count)
                    }
                    EdgeKind::Symmetric(things) => {
                        // Forward edges + reverse edges
                        // For 1 thing: 2 edges (1 request, 1 response)
                        // For n things: (n-1) forward + (n-1) reverse
                        let forward = things.len().max(1).saturating_sub(1).max(1);
                        let total = if things.len() <= 1 { 2 } else { forward * 2 };
                        (total, forward)
                    }
                };

                (0..edge_count).map(move |i| {
                    let edge_id_str = format!("{edge_group_id}__{i}");
                    let edge_id = Self::id_from_string(edge_id_str);

                    let interaction_type = match edge_kind {
                        EdgeKind::Cyclic(_) => EntityType::EdgeInteractionCyclicDefault,
                        EdgeKind::Sequence(_) => EntityType::EdgeInteractionSequenceDefault,
                        EdgeKind::Symmetric(_) => {
                            // First half are forward (request), second half are reverse (response)
                            if i < forward_count {
                                EntityType::EdgeInteractionSymmetricRequestDefault
                            } else {
                                EntityType::EdgeInteractionSymmetricResponseDefault
                            }
                        }
                    };

                    (edge_id, interaction_type)
                })
            })
            .for_each(|(edge_id, interaction_type)| {
                // Add to existing types or create new entry
                entity_types
                    .entry(edge_id)
                    .or_default()
                    .insert(interaction_type);
            });
    }

    /// Build NodeLayouts from node_hierarchy and theme data.
    fn build_node_layouts(
        node_hierarchy: &NodeHierarchy,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
        tags: &TagNames,
        processes: &Processes,
    ) -> NodeLayouts {
        let mut node_layouts = NodeLayouts::new();

        // Helper to determine if a node is a tag
        let is_tag = |node_id: &NodeId| tags.contains_key(node_id);

        // Helper to determine if a node is a process
        let is_process = |node_id: &NodeId| processes.contains_key(node_id);

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
        let process_layouts = processes.iter().flat_map(|(process_id, process_diagram)| {
            let process_node_id = NodeId::from(process_id.clone().into_inner());

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
                let step_node_id = NodeId::from(step_id.clone().into_inner());
                (step_node_id, NodeLayout::None)
            });

            std::iter::once((process_node_id, process_layout)).chain(step_layouts)
        });

        node_layouts.extend(process_layouts);

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
        let tag_layouts = tags.keys().map(|tag_id| {
            let tag_node_id = NodeId::from(tag_id.clone().into_inner());
            (tag_node_id, NodeLayout::None)
        });

        node_layouts.extend(tag_layouts);

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
        let thing_layouts: Vec<_> = hierarchy
            .iter()
            // Skip tags and processes (already handled)
            .filter(|(node_id, _)| !is_tag(node_id) && !is_process(node_id))
            .flat_map(|(node_id, children)| {
                let layout = if children.is_empty() {
                    // Leaf node - no layout needed
                    NodeLayout::None
                } else {
                    // Container node - use flex layout
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
            types
                .iter()
                .filter_map(|entity_type| {
                    let type_id = EntityTypeId::from(entity_type.clone().into_id());
                    theme_types_styles
                        .get(&type_id)
                        .and_then(|type_styles| type_styles.get(&IdOrDefaults::NodeDefaults))
                })
                .for_each(|type_partials| {
                    apply_from_partials(type_partials, &theme_default.style_aliases, state);
                });
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
        partials
            .style_aliases_applied()
            .iter()
            .filter_map(|alias| style_aliases.get(alias))
            .for_each(|alias_partials| Self::extract_padding_from_map(alias_partials, state));

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
        partials
            .style_aliases_applied()
            .iter()
            .filter_map(|alias| style_aliases.get(alias))
            .for_each(|alias_partials| Self::extract_margin_from_map(alias_partials, state));

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
        partials
            .style_aliases_applied()
            .iter()
            .filter_map(|alias| style_aliases.get(alias))
            .filter_map(|alias_partials| alias_partials.get(&ThemeAttr::Gap))
            .filter_map(|value| value.parse::<f32>().ok())
            .for_each(|v| *state = Some(v));

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
        theme_tag_things_focus: &ThemeTagThingsFocus,
        tags: &TagNames,
        tag_things: &TagThings,
        processes: &Processes,
    ) -> EntityTailwindClasses {
        // Build a map of process step ID to (process ID, edge IDs they interact with)
        let step_interactions = Self::build_step_interactions_map(processes);

        // Build a map of edge group ID to process steps that interact with it
        let edge_group_to_steps = Self::build_edge_group_to_steps_map(processes);

        // Build a map of thing ID to process steps that interact with edges involving
        // that thing
        let thing_to_interaction_steps =
            Self::build_thing_to_interaction_steps_map(edge_groups, &step_interactions);

        // Build classes for each node
        let node_classes = nodes.keys().map(|node_id| {
            // Determine node kind
            let is_tag = tags.contains_key(node_id);
            let is_process = processes.contains_key(node_id);
            let is_process_step = processes
                .values()
                .any(|process_diagram| process_diagram.steps.contains_key(node_id));

            let classes = if is_tag {
                Self::build_tag_tailwind_classes(
                    node_id,
                    entity_types,
                    theme_default,
                    theme_types_styles,
                )
            } else if is_process {
                // Find the child process step IDs
                let child_step_ids = processes
                    .iter()
                    .find_map(|(process_id, process_diagram)| {
                        if process_id.as_ref() == node_id.as_ref() {
                            Some(process_diagram.steps.keys())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();

                Self::build_process_tailwind_classes(
                    node_id,
                    child_step_ids,
                    entity_types,
                    theme_default,
                    theme_types_styles,
                )
            } else if is_process_step {
                // Find the parent process ID
                let parent_process_id =
                    processes.iter().find_map(|(process_id, process_diagram)| {
                        if process_diagram.steps.contains_key(node_id) {
                            Some(process_id)
                        } else {
                            None
                        }
                    });

                Self::build_process_step_tailwind_classes(
                    node_id,
                    parent_process_id,
                    entity_types,
                    theme_default,
                    theme_types_styles,
                )
            } else {
                // Regular thing node
                Self::build_thing_tailwind_classes(
                    node_id,
                    entity_types,
                    theme_default,
                    theme_types_styles,
                    theme_tag_things_focus,
                    tags,
                    tag_things,
                    &thing_to_interaction_steps,
                )
            };

            (node_id.clone().into_inner(), classes)
        });

        // Build classes for edge groups
        let edge_group_classes = edge_groups.keys().map(|edge_group_id| {
            // Get the process steps that interact with this edge group
            let interaction_steps = edge_group_to_steps
                .get(edge_group_id)
                .cloned()
                .unwrap_or_default();

            let classes = Self::build_edge_group_tailwind_classes(
                edge_group_id,
                entity_types,
                theme_default,
                theme_types_styles,
                &interaction_steps,
            );

            (edge_group_id.clone().into_inner(), classes)
        });

        // Build classes for individual edges within edge groups (symmetric edges)
        let edge_classes = edge_groups.iter().flat_map(|(edge_group_id, edges)| {
            edges.iter().enumerate().filter_map(move |(index, _edge)| {
                let edge_id_str = format!("{edge_group_id}__{index}");
                let edge_id = Self::id_from_string(edge_id_str);

                // Check if this edge has a symmetric type (request or response)
                let edge_types = entity_types.get(&edge_id)?;
                let is_symmetric = edge_types.iter().any(|entity_type| {
                    matches!(
                        entity_type,
                        EntityType::EdgeDependencySymmetricRequestDefault
                            | EntityType::EdgeDependencySymmetricResponseDefault
                            | EntityType::EdgeInteractionSymmetricRequestDefault
                            | EntityType::EdgeInteractionSymmetricResponseDefault
                    )
                });

                if is_symmetric {
                    let classes = Self::build_symmetric_edge_tailwind_classes(
                        &edge_id,
                        entity_types,
                        theme_default,
                        theme_types_styles,
                    );
                    Some((edge_id, classes))
                } else {
                    None
                }
            })
        });

        node_classes
            .chain(edge_group_classes)
            .chain(edge_classes)
            .collect()
    }

    /// Build a map of process step ID to (process ID, edge IDs they interact
    /// with).
    fn build_step_interactions_map(
        processes: &Processes,
    ) -> Map<&ProcessStepId, (&ProcessId, &Vec<EdgeGroupId>)> {
        processes
            .iter()
            .flat_map(|(process_id, process_diagram)| {
                process_diagram.step_thing_interactions.iter().map(
                    move |(process_step_id, edge_group_ids)| {
                        (process_step_id, (process_id, edge_group_ids))
                    },
                )
            })
            .collect()
    }

    /// Build a map of edge group ID to process steps that interact with it.
    fn build_edge_group_to_steps_map(
        processes: &Processes,
    ) -> Map<&EdgeGroupId, Vec<&ProcessStepId>> {
        processes
            .values()
            .flat_map(|process_diagram| {
                process_diagram.step_thing_interactions.iter().flat_map(
                    |(step_id, edge_group_ids)| {
                        edge_group_ids
                            .iter()
                            .map(move |edge_group_id| (edge_group_id, step_id))
                    },
                )
            })
            .fold(
                Map::<&EdgeGroupId, Vec<&ProcessStepId>>::new(),
                |mut acc, (edge_group_id, step_id)| {
                    acc.entry(edge_group_id).or_default().push(step_id);
                    acc
                },
            )
    }

    /// Build a map of thing ID to process steps that interact with edges
    /// involving that thing.
    fn build_thing_to_interaction_steps_map<'f>(
        edge_groups: &'f EdgeGroups,
        step_interactions: &'f Map<&'f ProcessStepId, (&'f ProcessId, &'f Vec<EdgeGroupId>)>,
    ) -> Map<&'f NodeId, Set<&'f ProcessStepId>> {
        // For each process step and its edge interactions
        step_interactions
            .iter()
            .flat_map(|(process_step_id, (_process_id, edge_group_ids))| {
                // For each edge group the step interacts with
                edge_group_ids.iter().flat_map(move |edge_group_id| {
                    edge_groups
                        .get(edge_group_id)
                        .into_iter()
                        .flat_map(move |edges| {
                            edges.iter().flat_map(move |edge| {
                                // Add this step to both the from and to things
                                [&edge.from, &edge.to]
                                    .into_iter()
                                    .map(move |node_id| (node_id, *process_step_id))
                            })
                        })
                })
            })
            .fold(
                Map::<&NodeId, Set<&ProcessStepId>>::new(),
                |mut acc, (node_id, step_id)| {
                    acc.entry(node_id).or_default().insert(step_id);
                    acc
                },
            )
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
            id,
            entity_types,
            theme_default,
            theme_types_styles,
            IdOrDefaults::NodeDefaults,
            &mut state,
        );

        let mut classes = String::new();
        state.write_classes(&mut classes);

        // Tags get peer/{id} class
        writeln!(&mut classes, "peer/{}", id.as_str()).expect(CLASSES_BUFFER_WRITE_FAIL);

        classes
    }

    /// Build tailwind classes for a process node.
    fn build_process_tailwind_classes(
        id: &Id,
        child_step_ids: Keys<'_, ProcessStepId, String>,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
    ) -> String {
        let mut state = TailwindClassState::default();

        Self::resolve_tailwind_attrs(
            id,
            entity_types,
            theme_default,
            theme_types_styles,
            IdOrDefaults::NodeDefaults,
            &mut state,
        );

        let mut classes = String::new();
        state.write_classes(&mut classes);

        // Processes get group/{id} class
        writeln!(&mut classes, "group/{}", id.as_str()).expect(CLASSES_BUFFER_WRITE_FAIL);

        // Processes get peer/{step_id} classes for each child process step
        // This is because process nodes are sibling elements to thing/edge_group
        // elements, whereas process step nodes are not siblings, so things and
        // edge_groups can only react to the process nodes' state for the
        // sibling selector to work.
        child_step_ids.for_each(|step_id| {
            writeln!(&mut classes, "peer/{}", step_id.as_str()).expect(CLASSES_BUFFER_WRITE_FAIL);
        });

        classes
    }

    /// Build tailwind classes for a process step node.
    fn build_process_step_tailwind_classes(
        id: &Id,
        parent_process_id: Option<&ProcessId>,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
    ) -> String {
        let mut state = TailwindClassState::default();

        Self::resolve_tailwind_attrs(
            id,
            entity_types,
            theme_default,
            theme_types_styles,
            IdOrDefaults::NodeDefaults,
            &mut state,
        );

        let mut classes = String::new();
        state.write_classes(&mut classes);

        // Process steps get group-focus-within/{process_id}:visible class
        // Note: peer/{step_id} classes are placed on the parent process node instead,
        // because process nodes are sibling elements to thing/edge_group elements,
        // whereas process step nodes are not siblings.
        if let Some(process_id) = parent_process_id {
            writeln!(&mut classes, "group-focus-within/{process_id}:visible")
                .expect(CLASSES_BUFFER_WRITE_FAIL);
        }

        classes
    }

    /// Build tailwind classes for a regular thing node.
    #[allow(clippy::too_many_arguments)]
    fn build_thing_tailwind_classes(
        node_id: &NodeId,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
        theme_tag_things_focus: &ThemeTagThingsFocus,
        tags: &TagNames,
        tag_things: &TagThings,
        thing_to_interaction_steps: &Map<&NodeId, Set<&ProcessStepId>>,
    ) -> String {
        let mut state = TailwindClassState::default();

        Self::resolve_tailwind_attrs(
            node_id.as_ref(),
            entity_types,
            theme_default,
            theme_types_styles,
            IdOrDefaults::NodeDefaults,
            &mut state,
        );

        let mut classes = String::new();
        state.write_classes(&mut classes);

        // Add peer classes for each tag
        tags.keys().for_each(|tag_id| {
            let is_thing_in_tag = tag_things
                .get(tag_id)
                .is_some_and(|thing_ids| thing_ids.contains(node_id.as_ref()));

            // Determine which IdOrDefaults key to use for styling
            let style_key = if is_thing_in_tag {
                IdOrDefaults::NodeDefaults
            } else {
                IdOrDefaults::NodeExcludedDefaults
            };

            // Build the tag focus state by:
            // 1. Starting with the thing's colors
            // 2. Applying TagDefaults styles
            // 3. Applying tag-specific styles (overrides)
            let mut tag_focus_state = TailwindClassState::default();
            if let Some(shape_color) = state.attrs.get(&ThemeAttr::ShapeColor) {
                tag_focus_state
                    .attrs
                    .insert(ThemeAttr::ShapeColor, shape_color.clone());
            };
            if let Some(fill_color) = state.attrs.get(&ThemeAttr::FillColor) {
                tag_focus_state
                    .attrs
                    .insert(ThemeAttr::FillColor, fill_color.clone());
            };
            if let Some(stroke_color) = state.attrs.get(&ThemeAttr::StrokeColor) {
                tag_focus_state
                    .attrs
                    .insert(ThemeAttr::StrokeColor, stroke_color.clone());
            };

            // Apply TagDefaults styles
            if let Some(tag_defaults_styles) =
                theme_tag_things_focus.get(&TagIdOrDefaults::TagDefaults)
                && let Some(partials) = tag_defaults_styles.get(&style_key)
            {
                Self::apply_tailwind_from_partials(
                    partials,
                    &theme_default.style_aliases,
                    &mut tag_focus_state,
                );
            }

            // Apply tag-specific styles (override TagDefaults)
            let tag_id_key = TagIdOrDefaults::Custom(tag_id.clone());
            if let Some(tag_specific_styles) = theme_tag_things_focus.get(&tag_id_key)
                && let Some(partials) = tag_specific_styles.get(&style_key)
            {
                Self::apply_tailwind_from_partials(
                    partials,
                    &theme_default.style_aliases,
                    &mut tag_focus_state,
                );
            }

            let peer_prefix = format!("peer-[:focus-within]/{tag_id}:");
            tag_focus_state.write_peer_classes(&mut classes, &peer_prefix);
        });

        // Add peer classes for process steps that interact with edges involving this
        // thing using styles from `theme_default.process_step_selected_styles`
        if let Some(interaction_steps) = thing_to_interaction_steps.get(node_id) {
            interaction_steps.iter().for_each(|step_id| {
                // Build a state from the thing's current colors + process_step_selected_styles
                let mut step_selected_state = TailwindClassState::default();

                // Copy the thing's colors
                if let Some(shape_color) = state.attrs.get(&ThemeAttr::ShapeColor) {
                    step_selected_state
                        .attrs
                        .insert(ThemeAttr::ShapeColor, shape_color.clone());
                };
                if let Some(fill_color) = state.attrs.get(&ThemeAttr::FillColor) {
                    step_selected_state
                        .attrs
                        .insert(ThemeAttr::FillColor, fill_color.clone());
                };
                if let Some(stroke_color) = state.attrs.get(&ThemeAttr::StrokeColor) {
                    step_selected_state
                        .attrs
                        .insert(ThemeAttr::StrokeColor, stroke_color.clone());
                };

                [
                    // lowest priority
                    IdOrDefaults::NodeDefaults,
                    IdOrDefaults::Id(node_id.clone().into_inner()),
                    // highest priority
                ]
                .iter()
                .filter_map(|id_or_defaults| {
                    theme_default
                        .process_step_selected_styles
                        .get(id_or_defaults)
                })
                .for_each(|css_class_partials| {
                    Self::apply_tailwind_from_partials(
                        css_class_partials,
                        &theme_default.style_aliases,
                        &mut step_selected_state,
                    );
                });

                let peer_prefix = format!("peer-[:focus-within]/{step_id}:");
                step_selected_state.write_peer_classes(&mut classes, &peer_prefix);
            });
        }

        classes
    }

    /// Build tailwind classes for an edge group.
    ///
    /// # Parameters
    ///
    /// * `edge_group_id`: The ID of the edge group.
    /// * `entity_types`: The entity types of the edge group.
    /// * `theme_default`: The theme with styling information.
    /// * `theme_types_styles`: Styles for each entity type.
    /// * `interaction_process_step_ids`: The process step IDs that interact
    ///   with this edge.
    fn build_edge_group_tailwind_classes(
        edge_group_id: &EdgeGroupId,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
        interaction_process_step_ids: &[&ProcessStepId],
    ) -> String {
        let mut state = TailwindClassState::default();

        Self::resolve_tailwind_attrs(
            edge_group_id,
            entity_types,
            theme_default,
            theme_types_styles,
            IdOrDefaults::EdgeDefaults,
            &mut state,
        );

        let mut classes = String::new();
        state.write_classes(&mut classes);

        // Add peer classes for each process step that interacts with this edge
        // using styles from `theme_default.process_step_selected_styles.edge_defaults`
        interaction_process_step_ids.iter().for_each(|step_id| {
            // Build a state from the thing's current colors + process_step_selected_styles
            let mut step_selected_state = TailwindClassState::default();

            [
                // lowest priority
                IdOrDefaults::EdgeDefaults,
                IdOrDefaults::Id(edge_group_id.clone().into_inner()),
                // highest priority
            ]
            .iter()
            .filter_map(|id_or_defaults| {
                theme_default
                    .process_step_selected_styles
                    .get(id_or_defaults)
            })
            .for_each(|css_class_partials| {
                Self::apply_tailwind_from_partials(
                    css_class_partials,
                    &theme_default.style_aliases,
                    &mut step_selected_state,
                );
            });

            let peer_prefix = format!("peer-[:focus-within]/{step_id}:");
            step_selected_state.write_peer_classes(&mut classes, &peer_prefix);
        });

        classes
    }

    /// Build tailwind classes for individual symmetric edges within an edge
    /// group.
    fn build_symmetric_edge_tailwind_classes(
        edge_id: &Id,
        entity_types: &IrEntityTypes,
        theme_default: &ThemeDefault,
        theme_types_styles: &ThemeTypesStyles,
    ) -> String {
        let mut state = TailwindClassState::default();

        Self::resolve_tailwind_attrs(
            edge_id,
            entity_types,
            theme_default,
            theme_types_styles,
            IdOrDefaults::EdgeDefaults,
            &mut state,
        );

        let mut classes = String::new();
        state.write_classes(&mut classes);
        classes
    }

    /// Resolve tailwind attributes for a node.
    ///
    /// # Parameters
    ///
    /// * `entity_id`: Thing, process, process step, tag, or edge ID.
    /// * `entity_types`: The entity types of the entity.
    /// * `theme_default`: The theme defined for the diagram.
    /// * `theme_types_styles`: The styles defined for entity types.
    /// * `id_or_defaults_key`: `IdOrDefaults::NodeDefaults` or
    ///   `IdOrDefaults::EdgeDefaults`.
    /// * `state`: Tailwind class state to write the resolved classes to.
    fn resolve_tailwind_attrs<'partials, 'tw_state>(
        entity_id: &Id,
        entity_types: &'partials IrEntityTypes,
        theme_default: &'partials ThemeDefault,
        theme_types_styles: &'partials ThemeTypesStyles,
        id_or_defaults_key: IdOrDefaults,
        state: &mut TailwindClassState<'tw_state>,
    ) where
        'partials: 'tw_state,
    {
        // 1. Start with NodeDefaults/EdgeDefaults (lowest priority)
        if let Some(defaults_partials) = theme_default.base_styles.get(&id_or_defaults_key) {
            Self::apply_tailwind_from_partials(
                defaults_partials,
                &theme_default.style_aliases,
                state,
            );
        }

        // 2. Apply EntityTypes in order (later types override earlier ones)
        if let Some(types) = entity_types.get(entity_id) {
            types
                .iter()
                .filter_map(|entity_type| {
                    let type_id = EntityTypeId::from(entity_type.clone().into_id());
                    theme_types_styles
                        .get(&type_id)
                        .and_then(|type_styles| type_styles.get(&id_or_defaults_key))
                })
                .for_each(|type_partials| {
                    Self::apply_tailwind_from_partials(
                        type_partials,
                        &theme_default.style_aliases,
                        state,
                    );
                });
        }

        // 3. Apply node ID itself (highest priority)
        if let Some(node_partials) = theme_default
            .base_styles
            .get(&IdOrDefaults::Id(entity_id.clone()))
        {
            Self::apply_tailwind_from_partials(node_partials, &theme_default.style_aliases, state);
        }
    }

    /// Apply tailwind attribute values from CssClassPartials.
    fn apply_tailwind_from_partials<'partials, 'tw_state>(
        partials: &'partials CssClassPartials,
        style_aliases: &'partials StyleAliases,
        state: &mut TailwindClassState<'tw_state>,
    ) where
        'partials: 'tw_state,
    {
        // First, check style_aliases_applied (lower priority within this partials)
        partials
            .style_aliases_applied()
            .iter()
            .filter_map(|alias| style_aliases.get(alias))
            .for_each(|alias_partials| Self::extract_tailwind_from_map(alias_partials, state));

        // Then, check direct attributes (higher priority within this partials)
        Self::extract_tailwind_from_map(partials, state);
    }

    /// Extract tailwind attribute values from a CssClassPartials map.
    fn extract_tailwind_from_map<'partials, 'tw_state>(
        partials: &'partials CssClassPartials,
        state: &mut TailwindClassState<'tw_state>,
    ) where
        'partials: 'tw_state,
    {
        partials.iter().for_each(|(theme_attr, value)| {
            state.attrs.insert(*theme_attr, Cow::Borrowed(value));
        });
    }
}

/// State for accumulating resolved tailwind class attributes.
///
/// This struct holds a map of [`ThemeAttr`] to their resolved string values,
/// which are then used to generate the appropriate tailwind CSS classes.
#[derive(Default)]
struct TailwindClassState<'tw_state> {
    /// Map of theme attributes to their resolved values.
    attrs: Map<ThemeAttr, Cow<'tw_state, str>>,
}

impl<'tw_state> TailwindClassState<'tw_state> {
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
    fn get_fill_color(&self, state: HighlightState) -> Option<&str> {
        let (state_specific, base, shape) = match state {
            HighlightState::Normal => (
                ThemeAttr::FillColorNormal,
                ThemeAttr::FillColor,
                ThemeAttr::ShapeColor,
            ),
            HighlightState::Focus => (
                ThemeAttr::FillColorFocus,
                ThemeAttr::FillColor,
                ThemeAttr::ShapeColor,
            ),
            HighlightState::Hover => (
                ThemeAttr::FillColorHover,
                ThemeAttr::FillColor,
                ThemeAttr::ShapeColor,
            ),
            HighlightState::Active => (
                ThemeAttr::FillColorActive,
                ThemeAttr::FillColor,
                ThemeAttr::ShapeColor,
            ),
        };

        self.attrs
            .get(&state_specific)
            .or_else(|| self.attrs.get(&base))
            .or_else(|| self.attrs.get(&shape))
            .map(|c| c.as_ref())
    }

    /// Get the resolved fill shade for a state.
    fn get_fill_shade(&self, state: HighlightState) -> Option<&str> {
        let (state_specific, base) = match state {
            HighlightState::Normal => (ThemeAttr::FillShadeNormal, ThemeAttr::FillShade),
            HighlightState::Focus => (ThemeAttr::FillShadeFocus, ThemeAttr::FillShade),
            HighlightState::Hover => (ThemeAttr::FillShadeHover, ThemeAttr::FillShade),
            HighlightState::Active => (ThemeAttr::FillShadeActive, ThemeAttr::FillShade),
        };

        self.attrs
            .get(&state_specific)
            .or_else(|| self.attrs.get(&base))
            .map(|c| c.as_ref())
    }

    /// Get the resolved stroke color for a state.
    fn get_stroke_color(&self, state: HighlightState) -> Option<&str> {
        let (state_specific, base, shape) = match state {
            HighlightState::Normal => (
                ThemeAttr::StrokeColorNormal,
                ThemeAttr::StrokeColor,
                ThemeAttr::ShapeColor,
            ),
            HighlightState::Focus => (
                ThemeAttr::StrokeColorFocus,
                ThemeAttr::StrokeColor,
                ThemeAttr::ShapeColor,
            ),
            HighlightState::Hover => (
                ThemeAttr::StrokeColorHover,
                ThemeAttr::StrokeColor,
                ThemeAttr::ShapeColor,
            ),
            HighlightState::Active => (
                ThemeAttr::StrokeColorActive,
                ThemeAttr::StrokeColor,
                ThemeAttr::ShapeColor,
            ),
        };

        self.attrs
            .get(&state_specific)
            .or_else(|| self.attrs.get(&base))
            .or_else(|| self.attrs.get(&shape))
            .map(|c| c.as_ref())
    }

    /// Get the resolved stroke shade for a state.
    fn get_stroke_shade(&self, state: HighlightState) -> Option<&str> {
        let (state_specific, base) = match state {
            HighlightState::Normal => (ThemeAttr::StrokeShadeNormal, ThemeAttr::StrokeShade),
            HighlightState::Focus => (ThemeAttr::StrokeShadeFocus, ThemeAttr::StrokeShade),
            HighlightState::Hover => (ThemeAttr::StrokeShadeHover, ThemeAttr::StrokeShade),
            HighlightState::Active => (ThemeAttr::StrokeShadeActive, ThemeAttr::StrokeShade),
        };

        self.attrs
            .get(&state_specific)
            .or_else(|| self.attrs.get(&base))
            .map(|c| c.as_ref())
    }

    /// Write tailwind classes to the given string.
    fn write_classes(&self, classes: &mut String) {
        self.write_peer_classes(classes, "");
    }

    /// Write peer-prefixed classes to the given string for tag/step
    /// highlighting.
    ///
    /// This method determines what classes to write based on the attributes
    /// present in the state:
    ///
    /// - If only [`ThemeAttr::Opacity`] is set (no fill/stroke shade normals or
    ///   animation), writes only the opacity class.
    /// - If [`ThemeAttr::Animate`] or fill/stroke shade normals are set, writes
    ///   the animation class (if present) followed by full fill/stroke peer
    ///   classes.
    fn write_peer_classes(&self, classes: &mut String, prefix: &str) {
        // Stroke dasharray from stroke_style
        if let Some(style) = self.attrs.get(&ThemeAttr::StrokeStyle)
            && let Some(dasharray) = Self::stroke_style_to_dasharray(style)
        {
            writeln!(classes, "{prefix}[stroke-dasharray:{dasharray}]")
                .expect(CLASSES_BUFFER_WRITE_FAIL);
        }

        // Stroke width
        if let Some(width) = self.attrs.get(&ThemeAttr::StrokeWidth) {
            writeln!(classes, "{prefix}stroke-{width}").expect(CLASSES_BUFFER_WRITE_FAIL);
        }

        // Visibility
        if let Some(visibility) = self.attrs.get(&ThemeAttr::Visibility) {
            writeln!(classes, "{prefix}{visibility}").expect(CLASSES_BUFFER_WRITE_FAIL);
        }

        if let Some(opacity) = self.attrs.get(&ThemeAttr::Opacity) {
            writeln!(classes, "{prefix}opacity-{opacity}").expect(CLASSES_BUFFER_WRITE_FAIL);
        }
        if let Some(animate) = self.attrs.get(&ThemeAttr::Animate) {
            writeln!(classes, "{prefix}animate-{animate}").expect(CLASSES_BUFFER_WRITE_FAIL);
        }

        let fill_color_hover = self.get_fill_color(HighlightState::Hover);
        let fill_shade_hover = self.get_fill_shade(HighlightState::Hover);
        let fill_color_normal = self.get_fill_color(HighlightState::Normal);
        let fill_shade_normal = self.get_fill_shade(HighlightState::Normal);
        let fill_color_focus = self.get_fill_color(HighlightState::Focus);
        let fill_shade_focus = self.get_fill_shade(HighlightState::Focus);
        let fill_color_active = self.get_fill_color(HighlightState::Active);
        let fill_shade_active = self.get_fill_shade(HighlightState::Active);

        let stroke_color_hover = self.get_stroke_color(HighlightState::Hover);
        let stroke_shade_hover = self.get_stroke_shade(HighlightState::Hover);
        let stroke_color_normal = self.get_stroke_color(HighlightState::Normal);
        let stroke_shade_normal = self.get_stroke_shade(HighlightState::Normal);
        let stroke_color_focus = self.get_stroke_color(HighlightState::Focus);
        let stroke_shade_focus = self.get_stroke_shade(HighlightState::Focus);
        let stroke_color_active = self.get_stroke_color(HighlightState::Active);
        let stroke_shade_active = self.get_stroke_shade(HighlightState::Active);

        // Fill classes with peer prefix
        if let Some((fill_color_hover, fill_shade_hover)) = fill_color_hover.zip(fill_shade_hover) {
            writeln!(
                classes,
                "{prefix}hover:fill-{fill_color_hover}-{fill_shade_hover}"
            )
            .expect(CLASSES_BUFFER_WRITE_FAIL);
        }
        if let Some((fill_color_normal, fill_shade_normal)) =
            fill_color_normal.zip(fill_shade_normal)
        {
            writeln!(
                classes,
                "{prefix}fill-{fill_color_normal}-{fill_shade_normal}"
            )
            .expect(CLASSES_BUFFER_WRITE_FAIL);
        }
        if let Some((fill_color_focus, fill_shade_focus)) = fill_color_focus.zip(fill_shade_focus) {
            writeln!(
                classes,
                "{prefix}focus:fill-{fill_color_focus}-{fill_shade_focus}"
            )
            .expect(CLASSES_BUFFER_WRITE_FAIL);
        }
        if let Some((fill_color_active, fill_shade_active)) =
            fill_color_active.zip(fill_shade_active)
        {
            writeln!(
                classes,
                "{prefix}active:fill-{fill_color_active}-{fill_shade_active}"
            )
            .expect(CLASSES_BUFFER_WRITE_FAIL);
        }

        // Stroke classes with peer prefix
        if let Some((stroke_color_hover, stroke_shade_hover)) =
            stroke_color_hover.zip(stroke_shade_hover)
        {
            writeln!(
                classes,
                "{prefix}hover:stroke-{stroke_color_hover}-{stroke_shade_hover}"
            )
            .expect(CLASSES_BUFFER_WRITE_FAIL);
        }
        if let Some((stroke_color_normal, stroke_shade_normal)) =
            stroke_color_normal.zip(stroke_shade_normal)
        {
            writeln!(
                classes,
                "{prefix}stroke-{stroke_color_normal}-{stroke_shade_normal}"
            )
            .expect(CLASSES_BUFFER_WRITE_FAIL);
        }
        if let Some((stroke_color_focus, stroke_shade_focus)) =
            stroke_color_focus.zip(stroke_shade_focus)
        {
            writeln!(
                classes,
                "{prefix}focus:stroke-{stroke_color_focus}-{stroke_shade_focus}"
            )
            .expect(CLASSES_BUFFER_WRITE_FAIL);
        }
        if let Some((stroke_color_active, stroke_shade_active)) =
            stroke_color_active.zip(stroke_shade_active)
        {
            writeln!(
                classes,
                "{prefix}active:stroke-{stroke_color_active}-{stroke_shade_active}"
            )
            .expect(CLASSES_BUFFER_WRITE_FAIL);
        }

        // Text classes
        let text_color = self.attrs.get(&ThemeAttr::TextColor).map(|c| c.as_ref());
        let text_shade = self.attrs.get(&ThemeAttr::TextShade).map(|c| c.as_ref());
        if let Some((text_color, text_shade)) = text_color.zip(text_shade) {
            writeln!(classes, "[&>text]:fill-{text_color}-{text_shade}")
                .expect(CLASSES_BUFFER_WRITE_FAIL);
        }
    }
}

/// States for fill and stroke colors.
#[derive(Clone, Copy)]
enum HighlightState {
    Normal,
    Focus,
    Hover,
    Active,
}
