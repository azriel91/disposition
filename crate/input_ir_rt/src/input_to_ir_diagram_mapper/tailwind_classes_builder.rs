use std::{borrow::Cow, fmt::Write};

use disposition_input_model::{
    process::{ProcessDiagram, ProcessId, ProcessStepId, Processes},
    tag::{TagNames, TagThings},
    theme::{
        CssClassPartials, IdOrDefaults, StyleAliases, TagIdOrDefaults, ThemeAttr, ThemeDefault,
        ThemeTagThingsFocus, ThemeTypesStyles,
    },
};
use disposition_ir_model::{
    edge::EdgeGroups,
    entity::{EntityTailwindClasses, EntityTypeId},
    node::{NodeId, NodeNames},
};
use disposition_model_common::{edge::EdgeGroupId, entity::EntityTypes, Id, Map, Set};

use super::tailwind_class_state::TailwindClassState;

const CLASSES_BUFFER_WRITE_FAIL: &str = "Failed to write string to buffer";

/// Builds tailwind CSS classes for all entities (nodes, edge groups, edges).
#[derive(Clone, Copy, Debug)]
pub(crate) struct TailwindClassesBuilder;

impl TailwindClassesBuilder {
    /// Build tailwind classes for all entities (nodes, edge groups, edges).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build<'id>(
        nodes: &NodeNames<'id>,
        edge_groups: &EdgeGroups<'id>,
        entity_types: &EntityTypes<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
        theme_tag_things_focus: &ThemeTagThingsFocus<'id>,
        tags: &TagNames<'id>,
        tag_things: &TagThings<'id>,
        processes: &Processes<'id>,
    ) -> EntityTailwindClasses<'id> {
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
                Self::build_process_tailwind_classes(
                    node_id,
                    entity_types,
                    theme_default,
                    theme_types_styles,
                )
            } else if is_process_step {
                // Find the parent process diagram
                let parent_process_id_and_diagram =
                    processes.iter().find_map(|(process_id, process_diagram)| {
                        if process_diagram.steps.contains_key(node_id) {
                            Some((process_id, process_diagram))
                        } else {
                            None
                        }
                    });

                Self::build_process_step_tailwind_classes(
                    node_id,
                    parent_process_id_and_diagram,
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

        // Build classes for edge groups and edges
        let edge_group_and_edge_classes = edge_groups.iter().flat_map(|(edge_group_id, edges)| {
            let edge_group_classes = {
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
            };

            let edge_classes = edges.iter().enumerate().map(move |(index, _edge)| {
                let edge_id_str = format!("{edge_group_id}__{index}");
                let edge_id = Id::try_from(edge_id_str).expect("valid ID string");

                // Check if this edge has a symmetric type (request or response)
                let classes = Self::build_edge_tailwind_classes(
                    &edge_id,
                    entity_types,
                    theme_default,
                    theme_types_styles,
                );
                (edge_id, classes)
            });

            std::iter::once(edge_group_classes).chain(edge_classes)
        });

        node_classes.chain(edge_group_and_edge_classes).collect()
    }

    // === Interaction Maps === //

    /// Build a map of process step ID to (process ID, edge IDs they interact
    /// with).
    fn build_step_interactions_map<'f, 'id>(
        processes: &'f Processes<'id>,
    ) -> Map<&'f ProcessStepId<'id>, (&'f ProcessId<'id>, &'f Vec<EdgeGroupId<'id>>)> {
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
    fn build_edge_group_to_steps_map<'f, 'id>(
        processes: &'f Processes<'id>,
    ) -> Map<&'f EdgeGroupId<'id>, Vec<&'f ProcessStepId<'id>>> {
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
                Map::<&EdgeGroupId<'id>, Vec<&ProcessStepId<'id>>>::new(),
                |mut acc, (edge_group_id, step_id)| {
                    acc.entry(edge_group_id).or_default().push(step_id);
                    acc
                },
            )
    }

    /// Build a map of thing ID to process steps that interact with edges
    /// involving that thing.
    fn build_thing_to_interaction_steps_map<'f, 'id>(
        edge_groups: &'f EdgeGroups<'id>,
        step_interactions: &'f Map<
            &'f ProcessStepId<'id>,
            (&'f ProcessId<'id>, &'f Vec<EdgeGroupId<'id>>),
        >,
    ) -> Map<&'f NodeId<'id>, Set<&'f ProcessStepId<'id>>> {
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
                Map::<&NodeId<'id>, Set<&ProcessStepId<'id>>>::new(),
                |mut acc, (node_id, step_id)| {
                    acc.entry(node_id).or_default().insert(step_id);
                    acc
                },
            )
    }

    // === Per-Entity Tailwind Class Builders === //

    /// Build tailwind classes for a tag node.
    fn build_tag_tailwind_classes<'id>(
        id: &Id<'id>,
        entity_types: &EntityTypes<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
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
        writeln!(&mut classes, "peer/{id}").expect(CLASSES_BUFFER_WRITE_FAIL);

        classes
    }

    /// Build tailwind classes for a process node.
    fn build_process_tailwind_classes<'id>(
        id: &Id<'id>,
        entity_types: &EntityTypes<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
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

        // Processes get `peer/{id}` class
        writeln!(&mut classes, "peer/{id}").expect(CLASSES_BUFFER_WRITE_FAIL);

        classes
    }

    /// Build tailwind classes for a process step node.
    fn build_process_step_tailwind_classes<'id>(
        id: &Id<'id>,
        parent_process_id_and_diagram: Option<(&ProcessId<'id>, &ProcessDiagram<'id>)>,
        entity_types: &EntityTypes<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
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

        // Process steps get:
        //
        // * `group-has-[#{process_id}:focus-within]:visible`
        // * one of `group-has-[#{process_step_id}:focus-within]:visible` for each of
        //   the process steps (including itself).
        //
        // so that when a process or sibling steps are focused, all steps within the
        // process are visible.
        //
        // These are the same for all steps in the process, so technically we could
        // compute it just once.
        //
        // When a process step is selected, `thing`s receive styles
        // `theme_default.process_step_selected_styles` -- see
        // `build_thing_tailwind_classes`
        if let Some((process_id, process_diagram)) = parent_process_id_and_diagram {
            writeln!(
                &mut classes,
                "group-has-[#{process_id}:focus-within]:visible"
            )
            .expect(CLASSES_BUFFER_WRITE_FAIL);

            process_diagram.steps.keys().for_each(|process_step_id| {
                writeln!(
                    &mut classes,
                    "group-has-[#{process_step_id}:focus-within]:visible"
                )
                .expect(CLASSES_BUFFER_WRITE_FAIL);
            });
        }

        writeln!(&mut classes, "peer/{id}").expect(CLASSES_BUFFER_WRITE_FAIL);

        classes
    }

    /// Build tailwind classes for a regular thing node.
    #[allow(clippy::too_many_arguments)]
    fn build_thing_tailwind_classes<'f, 'id>(
        node_id: &NodeId<'id>,
        entity_types: &EntityTypes<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
        theme_tag_things_focus: &ThemeTagThingsFocus<'id>,
        tags: &TagNames<'id>,
        tag_things: &TagThings<'id>,
        thing_to_interaction_steps: &Map<&'f NodeId<'id>, Set<&'f ProcessStepId<'id>>>,
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
        Self::build_thing_tailwind_classes_tags(
            &state,
            &mut classes,
            node_id,
            theme_default,
            theme_tag_things_focus,
            tags,
            tag_things,
        );

        // Add peer classes for process steps that interact with edges involving this
        // thing using styles from `theme_default.process_step_selected_styles`
        Self::build_thing_tailwind_classes_interactions(
            &state,
            &mut classes,
            node_id,
            theme_default,
            thing_to_interaction_steps,
        );

        classes
    }

    /// Write tag-related peer classes for a thing node.
    fn build_thing_tailwind_classes_tags<'id>(
        state: &TailwindClassState<'_>,
        classes: &mut String,
        node_id: &NodeId<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_tag_things_focus: &ThemeTagThingsFocus<'id>,
        tags: &TagNames<'id>,
        tag_things: &TagThings<'id>,
    ) {
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
            tag_focus_state.write_peer_classes(classes, &peer_prefix);
        });
    }

    /// Write interaction-related peer classes for a thing node.
    fn build_thing_tailwind_classes_interactions<'f, 'id>(
        state: &TailwindClassState<'_>,
        classes: &mut String,
        node_id: &NodeId<'id>,
        theme_default: &ThemeDefault<'id>,
        thing_to_interaction_steps: &Map<&'f NodeId<'id>, Set<&'f ProcessStepId<'id>>>,
    ) {
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
                step_selected_state.write_peer_classes(classes, &peer_prefix);
            });
        }
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
    fn build_edge_group_tailwind_classes<'id>(
        edge_group_id: &EdgeGroupId<'id>,
        entity_types: &EntityTypes<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
        interaction_process_step_ids: &[&ProcessStepId<'id>],
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
    fn build_edge_tailwind_classes<'id>(
        edge_id: &Id<'id>,
        entity_types: &EntityTypes<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
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

    // === Tailwind Attribute Resolution === //

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
    fn resolve_tailwind_attrs<'partials, 'tw_state, 'id>(
        entity_id: &Id<'id>,
        entity_types: &'partials EntityTypes<'id>,
        theme_default: &'partials ThemeDefault<'id>,
        theme_types_styles: &'partials ThemeTypesStyles<'id>,
        id_or_defaults_key: IdOrDefaults<'id>,
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
    fn apply_tailwind_from_partials<'partials, 'tw_state, 'id>(
        partials: &'partials CssClassPartials<'id>,
        style_aliases: &'partials StyleAliases<'id>,
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
    fn extract_tailwind_from_map<'partials, 'tw_state, 'id>(
        partials: &'partials CssClassPartials<'id>,
        state: &mut TailwindClassState<'tw_state>,
    ) where
        'partials: 'tw_state,
    {
        partials.iter().for_each(|(theme_attr, value)| {
            state.attrs.insert(*theme_attr, Cow::Borrowed(value));
        });
    }
}
