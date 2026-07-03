use std::{borrow::Cow, fmt::Write};

use disposition_input_model::{
    process::{ProcessDiagram, ProcessId, ProcessStepId, Processes},
    tag::{TagNames, TagThings},
    theme::{
        CssClassPartials, DarkModeShadeConfig, IdOrDefaults, StyleAliases, TagIdOrDefaults,
        ThemeAttr, ThemeDefault, ThemeTagThingsFocus, ThemeTypesStyles,
    },
    DiagramFocus,
};
use disposition_ir_model::{
    edge::{EdgeGroups, EdgeId},
    entity::{EntityTailwindClasses, EntityTypeId},
    node::{NodeId, NodeNames},
};
use disposition_model_common::{
    edge::EdgeGroupId,
    entity::{EntityType, EntityTypes},
    Id, Map, RenderOptions, Set,
};

use crate::{EdgeHaloIdGenerator, EdgeHaloOutlineIdGenerator};

use super::{
    css_theme_vars::CssThemeVars, tailwind_class_state::TailwindClassState,
    tailwind_focus_mode::TailwindFocusMode, ThemeResolveCtx,
};

const CLASSES_BUFFER_WRITE_FAIL: &str = "Failed to write string to buffer";

/// Result of building tailwind classes, containing both the per-entity
/// classes and the collected CSS theme variable definitions.
pub(crate) struct TailwindClassesBuildResult<'id> {
    /// Per-entity tailwind CSS classes.
    pub(crate) tailwind_classes: EntityTailwindClasses<'id>,
    /// CSS variable definitions for light/dark mode colour pairs.
    pub(crate) css_theme_vars: CssThemeVars,
}

/// Resolved, edge-independent tailwind classes for the interaction edge
/// halo and its outline rails, split by direction.
///
/// `None` fields mean `RenderOptions::interaction_edge_halo` is disabled, so
/// no halo (or outline) entry should be emitted for any edge.
#[derive(Clone, Debug, Default)]
struct InteractionEdgeHaloClasses {
    /// Halo classes for forward (request) interaction edges.
    halo_forward: Option<String>,
    /// Halo classes for reverse (response) interaction edges.
    halo_reverse: Option<String>,
    /// Halo outline classes for forward (request) interaction edges.
    outline_forward: Option<String>,
    /// Halo outline classes for reverse (response) interaction edges.
    outline_reverse: Option<String>,
}

/// Builds tailwind CSS classes for all entities (nodes, edge groups, edges).
#[derive(Clone, Copy, Debug)]
pub(crate) struct TailwindClassesBuilder;

impl TailwindClassesBuilder {
    /// Build tailwind classes for all entities (nodes, edge groups, edges).
    ///
    /// Returns both the per-entity tailwind classes and the collected CSS
    /// theme variable definitions that should be prepended to the diagram's
    /// CSS.
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
        process_render_expanded: bool,
        focus_mode: TailwindFocusMode<'_, 'id>,
        render_options: &RenderOptions,
    ) -> TailwindClassesBuildResult<'id> {
        let mut css_theme_vars = CssThemeVars::new(theme_default.dark_mode_config.selector);

        // Build a map of process step ID to (process ID, edge IDs they interact with)
        let step_interactions = Self::build_step_interactions_map(processes);

        // Build a map of edge group ID to process steps that interact with it
        let edge_group_to_steps = Self::build_edge_group_to_steps_map(processes);

        // Build a map of thing ID to process steps that interact with edges involving
        // that thing
        let thing_to_interaction_steps =
            Self::build_thing_to_interaction_steps_map(edge_groups, &step_interactions);

        // Build classes for each node
        let node_classes: Vec<_> = nodes
            .keys()
            .map(|node_id| {
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
                        &mut css_theme_vars,
                        focus_mode,
                    )
                } else if is_process {
                    Self::build_process_tailwind_classes(
                        node_id,
                        entity_types,
                        theme_default,
                        theme_types_styles,
                        &mut css_theme_vars,
                        focus_mode,
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
                        &mut css_theme_vars,
                        process_render_expanded,
                        focus_mode,
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
                        &mut css_theme_vars,
                        focus_mode,
                    )
                };

                (node_id.clone().into_inner(), classes)
            })
            .collect();

        // Build classes for edge groups and individual edges.
        let theme_resolve_ctx = ThemeResolveCtx {
            entity_types,
            theme_default,
            theme_types_styles,
        };
        let edge_classes = Self::edge_tailwind_classes_build(
            edge_groups,
            &edge_group_to_steps,
            theme_resolve_ctx,
            &mut css_theme_vars,
            focus_mode,
            processes.is_empty(),
            render_options,
        );

        let tailwind_classes = node_classes.into_iter().chain(edge_classes).collect();

        TailwindClassesBuildResult {
            tailwind_classes,
            css_theme_vars,
        }
    }

    /// Builds the tailwind classes for every edge group and individual edge.
    ///
    /// Edge group classes are resolved first into a `TailwindClassState` (not
    /// yet stringified). Each individual edge then clones the group state,
    /// applies any edge-specific overrides, and writes the combined classes to
    /// a single string. This prevents conflicting tailwind classes that would
    /// result from naively joining separate group and edge class strings.
    #[allow(clippy::too_many_arguments)]
    fn edge_tailwind_classes_build<'id>(
        edge_groups: &EdgeGroups<'id>,
        edge_group_to_steps: &Map<&EdgeGroupId<'id>, Vec<&ProcessStepId<'id>>>,
        theme_ctx: ThemeResolveCtx<'_, 'id>,
        css_theme_vars: &mut CssThemeVars,
        focus_mode: TailwindFocusMode<'_, 'id>,
        processes_is_empty: bool,
        render_options: &RenderOptions,
    ) -> Vec<(Id<'id>, String)> {
        let ThemeResolveCtx {
            entity_types,
            theme_default,
            theme_types_styles,
        } = theme_ctx;

        // The interaction edge halo's (and its outline's) classes are
        // identical for every forward (request) edge, and identical for
        // every reverse (response) edge, so resolve each once and reuse the
        // strings, rather than re-resolving per edge.
        let InteractionEdgeHaloClasses {
            halo_forward: halo_classes_forward,
            halo_reverse: halo_classes_reverse,
            outline_forward: halo_outline_classes_forward,
            outline_reverse: halo_outline_classes_reverse,
        } = Self::interaction_edge_halo_classes_resolve(
            render_options,
            theme_default,
            theme_types_styles,
            css_theme_vars,
        );

        let mut edge_classes: Vec<(Id<'id>, String)> = Vec::new();
        edge_groups.iter().for_each(|(edge_group_id, edges)| {
            // Get the process steps that interact with this edge group.
            let interaction_steps = edge_group_to_steps
                .get(edge_group_id)
                .cloned()
                .unwrap_or_default();

            let (edge_group_state, edge_group_peer_classes, is_interaction_group) =
                Self::build_edge_group_tailwind_class_state(
                    edge_group_id,
                    entity_types,
                    theme_default,
                    theme_types_styles,
                    &interaction_steps,
                    css_theme_vars,
                    focus_mode,
                );

            edges.iter().enumerate().for_each(|(index, _edge)| {
                let edge_id_str = format!("{edge_group_id}__{index}");
                let edge_id = Id::try_from(edge_id_str).expect("valid ID string");

                let classes = Self::build_edge_tailwind_classes(
                    edge_group_id,
                    &edge_group_state,
                    &edge_group_peer_classes,
                    &edge_id,
                    entity_types,
                    theme_default,
                    theme_types_styles,
                    css_theme_vars,
                    processes_is_empty,
                    is_interaction_group,
                );

                if is_interaction_group {
                    let is_reverse = entity_types.get(&edge_id).is_some_and(|types| {
                        types.contains(&EntityType::InteractionEdgeSymmetricReverseDefault)
                    });
                    let (halo_classes, halo_outline_classes) = if is_reverse {
                        (
                            halo_classes_reverse.as_ref(),
                            halo_outline_classes_reverse.as_ref(),
                        )
                    } else {
                        (
                            halo_classes_forward.as_ref(),
                            halo_outline_classes_forward.as_ref(),
                        )
                    };
                    let edge_id_for_halo: EdgeId<'id> = edge_id.clone().into();
                    if let Some(halo_classes) = halo_classes {
                        let halo_id = EdgeHaloIdGenerator::generate(&edge_id_for_halo);
                        edge_classes.push((halo_id, halo_classes.clone()));
                    }
                    if let Some(halo_outline_classes) = halo_outline_classes {
                        let halo_outline_id =
                            EdgeHaloOutlineIdGenerator::generate(&edge_id_for_halo);
                        edge_classes.push((halo_outline_id, halo_outline_classes.clone()));
                    }
                }

                edge_classes.push((edge_id, classes));
            });
        });
        edge_classes
    }

    /// Resolves the (edge-independent, but forward/reverse-specific)
    /// tailwind classes for the interaction edge halo and its outline rails.
    ///
    /// Both are gated on the same `RenderOptions::interaction_edge_halo`
    /// toggle -- the outline rails paint on top of the halo fill, so they
    /// don't make sense without it.
    fn interaction_edge_halo_classes_resolve<'id>(
        render_options: &RenderOptions,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
        css_theme_vars: &mut CssThemeVars,
    ) -> InteractionEdgeHaloClasses {
        if !render_options.interaction_edge_halo.is_enabled() {
            return InteractionEdgeHaloClasses::default();
        }

        let halo_forward = Self::interaction_edge_halo_classes_build(
            EntityType::InteractionEdgeHaloForward,
            theme_default,
            theme_types_styles,
            css_theme_vars,
        );
        let halo_reverse = Self::interaction_edge_halo_classes_build(
            EntityType::InteractionEdgeHaloReverse,
            theme_default,
            theme_types_styles,
            css_theme_vars,
        );
        let outline_forward = Self::interaction_edge_halo_outline_classes_build(
            EntityType::InteractionEdgeHaloOutlineForward,
            theme_default,
            theme_types_styles,
            css_theme_vars,
        );
        let outline_reverse = Self::interaction_edge_halo_outline_classes_build(
            EntityType::InteractionEdgeHaloOutlineReverse,
            theme_default,
            theme_types_styles,
            css_theme_vars,
        );

        InteractionEdgeHaloClasses {
            halo_forward: Some(halo_forward),
            halo_reverse: Some(halo_reverse),
            outline_forward: Some(outline_forward),
            outline_reverse: Some(outline_reverse),
        }
    }

    /// Builds the (edge-independent, but forward/reverse-specific) tailwind
    /// classes for the interaction edge halo.
    ///
    /// Resolves `type_interaction_edge_halo` as the base, then overlays
    /// `overlay_entity_type` (`InteractionEdgeHaloForward` for requests, or
    /// `InteractionEdgeHaloReverse` for responses) on top -- any theme
    /// attribute set on the overlay type replaces the corresponding
    /// attribute inherited from the base.
    fn interaction_edge_halo_classes_build<'id>(
        overlay_entity_type: EntityType,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
        css_theme_vars: &mut CssThemeVars,
    ) -> String {
        Self::interaction_edge_halo_or_outline_classes_build(
            EntityType::InteractionEdgeHalo,
            overlay_entity_type,
            theme_default,
            theme_types_styles,
            css_theme_vars,
        )
    }

    /// Builds the (edge-independent, but forward/reverse-specific) tailwind
    /// classes for the interaction edge halo's outline rails.
    ///
    /// Resolves `type_interaction_edge_halo_outline` as the base, then
    /// overlays `overlay_entity_type` (`InteractionEdgeHaloOutlineForward`
    /// for requests, or `InteractionEdgeHaloOutlineReverse` for responses) on
    /// top, mirroring `interaction_edge_halo_classes_build`.
    fn interaction_edge_halo_outline_classes_build<'id>(
        overlay_entity_type: EntityType,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
        css_theme_vars: &mut CssThemeVars,
    ) -> String {
        Self::interaction_edge_halo_or_outline_classes_build(
            EntityType::InteractionEdgeHaloOutline,
            overlay_entity_type,
            theme_default,
            theme_types_styles,
            css_theme_vars,
        )
    }

    /// Shared implementation for `interaction_edge_halo_classes_build` and
    /// `interaction_edge_halo_outline_classes_build` -- resolves
    /// `base_entity_type` then overlays `overlay_entity_type` on top into
    /// one `TailwindClassState`.
    fn interaction_edge_halo_or_outline_classes_build<'id>(
        base_entity_type: EntityType,
        overlay_entity_type: EntityType,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
        css_theme_vars: &mut CssThemeVars,
    ) -> String {
        let mut tailwind_class_state = TailwindClassState {
            entity_type: Some(base_entity_type.clone()),
            ..Default::default()
        };

        [base_entity_type, overlay_entity_type]
            .into_iter()
            .for_each(|entity_type| {
                let type_id = EntityTypeId::from(entity_type.into_id());
                if let Some(halo_partials) = theme_types_styles
                    .get(&type_id)
                    .and_then(|type_styles| type_styles.get(&IdOrDefaults::EdgeDefaults))
                {
                    Self::apply_tailwind_from_partials(
                        halo_partials,
                        &theme_default.style_aliases,
                        &mut tailwind_class_state,
                    );
                }
            });

        let mut classes = String::new();
        tailwind_class_state.write_classes(
            &mut classes,
            css_theme_vars,
            theme_default.dark_mode_config.shade,
        );

        classes
    }

    // === Focus Mode Helpers === //

    /// Writes a focus state's classes according to the focus mode.
    ///
    /// In [`TailwindFocusMode::Interactive`] mode the classes are written with
    /// the given `peer_prefix` so the focus styling is toggled at render time
    /// via CSS `:focus-within`. In [`TailwindFocusMode::Baked`] mode the
    /// classes are written without any prefix (so they apply
    /// unconditionally) when this focus is the active one, and skipped
    /// otherwise.
    fn focus_state_write(
        classes: &mut String,
        focus_state: &TailwindClassState<'_>,
        peer_prefix: &str,
        focus_is_active: bool,
        focus_mode: TailwindFocusMode<'_, '_>,
        css_theme_vars: &mut CssThemeVars,
        dark_mode_shade_config: DarkModeShadeConfig,
    ) {
        match focus_mode {
            TailwindFocusMode::Interactive => {
                focus_state.write_peer_classes(
                    classes,
                    peer_prefix,
                    css_theme_vars,
                    dark_mode_shade_config,
                );
            }
            TailwindFocusMode::Baked { .. } => {
                if focus_is_active {
                    focus_state.write_classes(classes, css_theme_vars, dark_mode_shade_config);
                }
            }
        }
    }

    /// Returns whether the focus mode bakes the given process step as the
    /// active focus.
    fn focus_active_is_step<'id>(
        focus_mode: TailwindFocusMode<'_, 'id>,
        process_step_id: &Id<'id>,
    ) -> bool {
        matches!(
            focus_mode,
            TailwindFocusMode::Baked {
                active: DiagramFocus::ProcessStep {
                    process_step_id: active_step_id,
                    ..
                },
            } if active_step_id.as_ref() == process_step_id
        )
    }

    /// Returns whether `active` focuses the given process (directly, or via one
    /// of its steps), so that the process's steps should be revealed.
    fn focus_active_in_process<'id>(
        active: &DiagramFocus<'id>,
        parent_process_id: Option<&Id<'id>>,
    ) -> bool {
        let Some(parent_process_id) = parent_process_id else {
            return false;
        };
        match active {
            DiagramFocus::Process(process_id) => process_id.as_ref() == parent_process_id,
            DiagramFocus::ProcessStep { process_id, .. } => {
                process_id.as_ref() == parent_process_id
            }
            DiagramFocus::None | DiagramFocus::Tag(_) => false,
        }
    }

    /// Overrides the resolved `Visibility` attribute to `invisible` when the
    /// process renders collapsed, shared by process step nodes and process
    /// step connector edges.
    ///
    /// Must be called before `tailwind_class_state.write_classes(..)`, so
    /// the override replaces (rather than duplicates alongside) any default
    /// `visible` resolved from the theme's base styles.
    ///
    /// * `process_render_expanded`: when `true`, nothing is hidden (the
    ///   attribute is left untouched).
    /// * In [`TailwindFocusMode::Baked`] mode, `invisible` is written only when
    ///   `active` does not target this process (via
    ///   `Self::focus_active_in_process`).
    /// * In [`TailwindFocusMode::Interactive`] mode, `invisible` is always
    ///   written when collapsed (revealed at render time via the
    ///   `group-has-[...]:visible` classes written by
    ///   [`Self::process_step_visibility_reveal_classes_write`]).
    fn process_step_visibility_attr_write<'id>(
        tailwind_class_state: &mut TailwindClassState<'_>,
        process_render_expanded: bool,
        focus_mode: TailwindFocusMode<'_, 'id>,
        process_id: &Id<'id>,
    ) {
        let visible = process_render_expanded
            || matches!(
                focus_mode,
                TailwindFocusMode::Baked { active }
                    if Self::focus_active_in_process(active, Some(process_id))
            );
        if !visible {
            tailwind_class_state
                .attrs
                .insert(ThemeAttr::Visibility, Cow::Borrowed("invisible"));
        }
    }

    /// Writes the `group-has-[...]:visible` classes that reveal a process
    /// step node or process step connector edge when the process (or any of
    /// its steps) gains focus, shared by both.
    ///
    /// Must be called after `tailwind_class_state.write_classes(..)`. In
    /// [`TailwindFocusMode::Baked`] mode this writes nothing -- visibility is
    /// already resolved statically by
    /// [`Self::process_step_visibility_attr_write`].
    fn process_step_visibility_reveal_classes_write<'a, 'id>(
        classes: &mut String,
        focus_mode: TailwindFocusMode<'_, 'id>,
        process_id: &Id<'id>,
        process_step_ids: impl Iterator<Item = &'a ProcessStepId<'id>>,
    ) where
        'id: 'a,
    {
        if matches!(focus_mode, TailwindFocusMode::Interactive) {
            writeln!(classes, "group-has-[#{process_id}:focus-within]:visible")
                .expect(CLASSES_BUFFER_WRITE_FAIL);

            process_step_ids.for_each(|process_step_id| {
                writeln!(
                    classes,
                    "group-has-[#{process_step_id}:focus-within]:visible"
                )
                .expect(CLASSES_BUFFER_WRITE_FAIL);
            });
        }
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
        tag_id: &Id<'id>,
        entity_types: &EntityTypes<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
        css_theme_vars: &mut CssThemeVars,
        focus_mode: TailwindFocusMode<'_, 'id>,
    ) -> String {
        let entity_type = entity_types
            .get(tag_id)
            .and_then(|types| types.iter().next())
            .cloned();
        let mut tailwind_class_state = TailwindClassState {
            entity_type,
            ..Default::default()
        };

        Self::resolve_tailwind_attrs(
            tag_id,
            entity_types,
            theme_default,
            theme_types_styles,
            IdOrDefaults::NodeDefaults,
            true,
            &mut tailwind_class_state,
        );

        let mut classes = String::new();
        tailwind_class_state.write_classes(
            &mut classes,
            css_theme_vars,
            theme_default.dark_mode_config.shade,
        );

        // Tags get a `peer/{id}` class so other entities can react to the tag
        // being focused. In baked mode there is no interactive focus, so it is
        // omitted.
        if matches!(focus_mode, TailwindFocusMode::Interactive) {
            writeln!(&mut classes, "peer/{tag_id}").expect(CLASSES_BUFFER_WRITE_FAIL);
        }

        classes
    }

    /// Build tailwind classes for a process node.
    fn build_process_tailwind_classes<'id>(
        process_id: &Id<'id>,
        entity_types: &EntityTypes<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
        css_theme_vars: &mut CssThemeVars,
        focus_mode: TailwindFocusMode<'_, 'id>,
    ) -> String {
        let entity_type = entity_types
            .get(process_id)
            .and_then(|types| types.iter().next())
            .cloned();
        let mut tailwind_class_state = TailwindClassState {
            entity_type,
            ..Default::default()
        };

        Self::resolve_tailwind_attrs(
            process_id,
            entity_types,
            theme_default,
            theme_types_styles,
            IdOrDefaults::NodeDefaults,
            true,
            &mut tailwind_class_state,
        );

        let mut classes = String::new();
        tailwind_class_state.write_classes(
            &mut classes,
            css_theme_vars,
            theme_default.dark_mode_config.shade,
        );

        // Processes get a `peer/{id}` class so steps can react to the process
        // being focused. In baked mode there is no interactive focus, so it is
        // omitted.
        if matches!(focus_mode, TailwindFocusMode::Interactive) {
            writeln!(&mut classes, "peer/{process_id}").expect(CLASSES_BUFFER_WRITE_FAIL);
        }

        classes
    }

    /// Build tailwind classes for a process step node.
    #[allow(clippy::too_many_arguments)]
    fn build_process_step_tailwind_classes<'id>(
        process_step_id: &Id<'id>,
        parent_process_id_and_diagram: Option<(&ProcessId<'id>, &ProcessDiagram<'id>)>,
        entity_types: &EntityTypes<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
        css_theme_vars: &mut CssThemeVars,
        process_render_expanded: bool,
        focus_mode: TailwindFocusMode<'_, 'id>,
    ) -> String {
        let entity_type = entity_types
            .get(process_step_id)
            .and_then(|types| types.iter().next())
            .cloned();
        let mut tailwind_class_state = TailwindClassState {
            entity_type,
            ..Default::default()
        };

        Self::resolve_tailwind_attrs(
            process_step_id,
            entity_types,
            theme_default,
            theme_types_styles,
            IdOrDefaults::NodeDefaults,
            true,
            &mut tailwind_class_state,
        );

        // When processes are rendered collapsed, their steps are hidden by
        // default. Must run before `write_classes` so the `invisible`
        // override replaces (not duplicates alongside) the resolved default
        // `visible` attribute.
        if let Some((process_id, _)) = parent_process_id_and_diagram {
            Self::process_step_visibility_attr_write(
                &mut tailwind_class_state,
                process_render_expanded,
                focus_mode,
                process_id.as_ref(),
            );
        }

        let mut classes = String::new();
        tailwind_class_state.write_classes(
            &mut classes,
            css_theme_vars,
            theme_default.dark_mode_config.shade,
        );

        // Revealed when the process or a sibling step is focused. When a
        // process step is selected, `thing`s receive styles
        // `theme_default.process_step_selected_styles` -- see
        // `build_thing_tailwind_classes`.
        if let Some((process_id, process_diagram)) = parent_process_id_and_diagram {
            Self::process_step_visibility_reveal_classes_write(
                &mut classes,
                focus_mode,
                process_id.as_ref(),
                process_diagram.steps.keys(),
            );
        }

        if matches!(focus_mode, TailwindFocusMode::Interactive) {
            writeln!(&mut classes, "peer/{process_step_id}").expect(CLASSES_BUFFER_WRITE_FAIL);
        }

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
        css_theme_vars: &mut CssThemeVars,
        focus_mode: TailwindFocusMode<'_, 'id>,
    ) -> String {
        let entity_type = entity_types
            .get(node_id.as_ref())
            .and_then(|types| types.iter().next())
            .cloned();
        let mut tailwind_class_state = TailwindClassState {
            entity_type,
            ..Default::default()
        };

        Self::resolve_tailwind_attrs(
            node_id.as_ref(),
            entity_types,
            theme_default,
            theme_types_styles,
            IdOrDefaults::NodeDefaults,
            true,
            &mut tailwind_class_state,
        );

        let mut classes = String::new();
        tailwind_class_state.write_classes(
            &mut classes,
            css_theme_vars,
            theme_default.dark_mode_config.shade,
        );

        // Add tag focus styles (peer classes in interactive mode, baked classes
        // for the active tag in baked mode).
        Self::build_thing_tailwind_classes_tags(
            &tailwind_class_state,
            &mut classes,
            node_id,
            theme_default,
            theme_tag_things_focus,
            tags,
            tag_things,
            css_theme_vars,
            focus_mode,
        );

        // Add process step interaction styles for process steps that interact
        // with edges involving this thing, using styles from
        // `theme_default.process_step_selected_styles`.
        Self::build_thing_tailwind_classes_interactions(
            &tailwind_class_state,
            &mut classes,
            node_id,
            theme_default,
            thing_to_interaction_steps,
            css_theme_vars,
            focus_mode,
        );

        classes
    }

    /// Write tag-related peer classes for a thing node.
    #[allow(clippy::too_many_arguments)]
    fn build_thing_tailwind_classes_tags<'id>(
        tailwind_class_state: &TailwindClassState<'_>,
        classes: &mut String,
        node_id: &NodeId<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_tag_things_focus: &ThemeTagThingsFocus<'id>,
        tags: &TagNames<'id>,
        tag_things: &TagThings<'id>,
        css_theme_vars: &mut CssThemeVars,
        focus_mode: TailwindFocusMode<'_, 'id>,
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
            if let Some(shape_color) = tailwind_class_state.attrs.get(&ThemeAttr::ShapeColor) {
                tag_focus_state
                    .attrs
                    .insert(ThemeAttr::ShapeColor, shape_color.clone());
            };
            if let Some(fill_color) = tailwind_class_state.attrs.get(&ThemeAttr::FillColor) {
                tag_focus_state
                    .attrs
                    .insert(ThemeAttr::FillColor, fill_color.clone());
            };
            if let Some(stroke_color) = tailwind_class_state.attrs.get(&ThemeAttr::StrokeColor) {
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
            let focus_is_active = matches!(
                focus_mode,
                TailwindFocusMode::Baked {
                    active: DiagramFocus::Tag(active_tag_id),
                } if active_tag_id.as_ref() == tag_id.as_ref()
            );
            Self::focus_state_write(
                classes,
                &tag_focus_state,
                &peer_prefix,
                focus_is_active,
                focus_mode,
                css_theme_vars,
                theme_default.dark_mode_config.shade,
            );
        });
    }

    /// Write interaction-related peer classes for a thing node.
    fn build_thing_tailwind_classes_interactions<'f, 'id>(
        tailwind_class_state: &TailwindClassState<'_>,
        classes: &mut String,
        node_id: &NodeId<'id>,
        theme_default: &ThemeDefault<'id>,
        thing_to_interaction_steps: &Map<&'f NodeId<'id>, Set<&'f ProcessStepId<'id>>>,
        css_theme_vars: &mut CssThemeVars,
        focus_mode: TailwindFocusMode<'_, 'id>,
    ) {
        if let Some(interaction_steps) = thing_to_interaction_steps.get(node_id) {
            interaction_steps.iter().for_each(|step_id| {
                // Build a state from the thing's current colors + process_step_selected_styles
                let mut step_selected_state = TailwindClassState::default();

                // Copy the thing's colors
                if let Some(shape_color) = tailwind_class_state.attrs.get(&ThemeAttr::ShapeColor) {
                    step_selected_state
                        .attrs
                        .insert(ThemeAttr::ShapeColor, shape_color.clone());
                };
                if let Some(fill_color) = tailwind_class_state.attrs.get(&ThemeAttr::FillColor) {
                    step_selected_state
                        .attrs
                        .insert(ThemeAttr::FillColor, fill_color.clone());
                };
                if let Some(stroke_color) = tailwind_class_state.attrs.get(&ThemeAttr::StrokeColor)
                {
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
                let focus_is_active = Self::focus_active_is_step(focus_mode, step_id.as_ref());
                Self::focus_state_write(
                    classes,
                    &step_selected_state,
                    &peer_prefix,
                    focus_is_active,
                    focus_mode,
                    css_theme_vars,
                    theme_default.dark_mode_config.shade,
                );
            });
        }
    }

    /// Build the tailwind class state for an edge group, along with a
    /// pre-built peer classes string for process step interactions, and
    /// whether the group is an interaction edge group.
    ///
    /// The returned `TailwindClassState` represents the resolved base styling
    /// for the group (EdgeDefaults + group entity-type + group ID overrides)
    /// but has not yet been written to a `String`. It should be cloned for
    /// each individual edge within the group, allowing edge-specific overrides
    /// to be applied before stringification.
    ///
    /// # Parameters
    ///
    /// * `edge_group_id`: The ID of the edge group.
    /// * `entity_types`: The entity types of the edge group.
    /// * `theme_default`: The theme with styling information.
    /// * `theme_types_styles`: Styles for each entity type.
    /// * `interaction_process_step_ids`: The process step IDs that interact
    ///   with this edge.
    /// * `css_theme_vars`: Collector for CSS variable definitions.
    #[allow(clippy::too_many_arguments)]
    fn build_edge_group_tailwind_class_state<'id, 'tw_state>(
        edge_group_id: &EdgeGroupId<'id>,
        entity_types: &'tw_state EntityTypes<'id>,
        theme_default: &'tw_state ThemeDefault<'id>,
        theme_types_styles: &'tw_state ThemeTypesStyles<'id>,
        interaction_process_step_ids: &[&ProcessStepId<'id>],
        css_theme_vars: &mut CssThemeVars,
        focus_mode: TailwindFocusMode<'_, 'id>,
    ) -> (TailwindClassState<'tw_state>, String, bool)
    where
        'id: 'tw_state,
    {
        let entity_type = entity_types
            .get(edge_group_id.as_ref())
            .and_then(|types| types.iter().next())
            .cloned();
        let mut tailwind_class_state = TailwindClassState {
            entity_type,
            ..Default::default()
        };

        // The group ID override is intentionally NOT applied here (last
        // parameter `false`): it must run after the individual edge's own
        // entity types (see `build_edge_tailwind_classes`), which are
        // applied on top of this group state, so it stays the higher
        // priority as documented, instead of being shadowed by them.
        Self::resolve_tailwind_attrs(
            edge_group_id,
            entity_types,
            theme_default,
            theme_types_styles,
            IdOrDefaults::EdgeDefaults,
            false,
            &mut tailwind_class_state,
        );

        let is_interaction_edge = entity_types
            .get(edge_group_id.as_ref())
            .is_some_and(|types| types.iter().any(EntityType::is_interaction_edge));

        // Build peer classes string for process step interactions.
        //
        // These are the same for all edges in the group and will be appended
        // to each individual edge's classes string.
        let mut peer_classes = String::new();
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
            let focus_is_active = Self::focus_active_is_step(focus_mode, step_id.as_ref());
            Self::focus_state_write(
                &mut peer_classes,
                &step_selected_state,
                &peer_prefix,
                focus_is_active,
                focus_mode,
                css_theme_vars,
                theme_default.dark_mode_config.shade,
            );
        });

        (tailwind_class_state, peer_classes, is_interaction_edge)
    }

    /// Build tailwind classes for an individual edge within an edge group.
    ///
    /// Starts from a clone of the edge group's resolved `TailwindClassState`,
    /// applies any edge-specific entity-type and ID-style overrides, then
    /// writes the combined attributes to a string. The pre-built peer classes
    /// from the edge group are appended at the end.
    ///
    /// This ensures each edge's final class string already contains the full
    /// resolved styling -- group-level attributes overlaid with edge-specific
    /// overrides -- so callers do not need to join separate group and edge
    /// class strings.
    ///
    /// # Parameters
    ///
    /// * `edge_group_id`: The ID of the edge group, used to apply the group's
    ///   own ID-specific style override at the correct priority (see below).
    /// * `edge_group_state`: The resolved state for the edge group (not yet
    ///   stringified). Does NOT include the edge group's own ID override --
    ///   that is applied here instead, see below.
    /// * `edge_group_peer_classes`: Pre-built peer classes for process step
    ///   interactions, shared by all edges in the group.
    /// * `edge_id`: The ID of the individual edge (e.g., `edge_group_id__0`).
    /// * `entity_types`: Entity types map for resolving type-based styles.
    /// * `theme_default`: The default theme configuration.
    /// * `theme_types_styles`: Styles defined per entity type.
    /// * `css_theme_vars`: Collector for CSS variable definitions.
    /// * `processes_is_empty`: Whether the diagram has no processes, in which
    ///   case interaction edges are made visible by default.
    /// * `is_interaction_edge`: Whether this edge belongs to an interaction
    ///   edge group.
    #[allow(clippy::too_many_arguments)]
    fn build_edge_tailwind_classes<'id, 'tw_state>(
        edge_group_id: &EdgeGroupId<'id>,
        edge_group_state: &TailwindClassState<'tw_state>,
        edge_group_peer_classes: &str,
        edge_id: &Id<'id>,
        entity_types: &'tw_state EntityTypes<'id>,
        theme_default: &'tw_state ThemeDefault<'id>,
        theme_types_styles: &'tw_state ThemeTypesStyles<'id>,
        css_theme_vars: &mut CssThemeVars,
        processes_is_empty: bool,
        is_interaction_edge: bool,
    ) -> String
    where
        'id: 'tw_state,
    {
        // Start from the edge group's resolved state.
        let mut tailwind_class_state = edge_group_state.clone();

        // Update entity_type if the edge has its own entity type defined.
        if let Some(entity_type) = entity_types
            .get(edge_id)
            .and_then(|types| types.iter().next())
            .cloned()
        {
            tailwind_class_state.entity_type = Some(entity_type);
        }

        // Apply the edge's own entity-type styles on top of the group state.
        Self::resolve_tailwind_attrs_edge_types(
            edge_id,
            entity_types,
            theme_default,
            theme_types_styles,
            &mut tailwind_class_state,
        );

        // Interaction edges default to `invisible` (via
        // `type_interaction_edge_default`, applied above) because they are
        // meant to be revealed by focusing a process step. When the diagram has
        // no processes at all, there is nothing to reveal them, so make them
        // visible by default (their animation is likewise forced on in
        // `TaffyToSvgElementsMapper`). This must run after the edge-specific
        // overrides above so it always wins, regardless of the edge's own
        // entity types.
        if processes_is_empty && is_interaction_edge {
            tailwind_class_state
                .attrs
                .insert(ThemeAttr::Visibility, Cow::Borrowed("visible"));
        }

        // ID overrides are applied last, in increasing specificity, so
        // explicit per-ID customisation always wins over entity-type styling
        // regardless of which tier (group or edge) the type came from.
        Self::apply_id_specific_partials(
            edge_group_id.as_ref(),
            theme_default,
            &mut tailwind_class_state,
        );
        Self::apply_id_specific_partials(edge_id, theme_default, &mut tailwind_class_state);

        let mut classes = String::new();
        tailwind_class_state.write_classes(
            &mut classes,
            css_theme_vars,
            theme_default.dark_mode_config.shade,
        );

        // Append the pre-built peer classes from the edge group.
        if !edge_group_peer_classes.is_empty() {
            classes.push('\n');
            classes.push_str(edge_group_peer_classes);
        }

        classes
    }

    /// Resolves the tailwind classes for the connector edges of a single
    /// process's step graph.
    ///
    /// Process step connectors are styled like dependency edges: the theme's
    /// base `edge_defaults` overlaid with the `type_dependency_edge_default`
    /// edge styling. The resulting classes carry no per-edge overrides, so a
    /// single resolved string is shared across all of one process's
    /// connectors -- but connectors of different processes hide/reveal
    /// independently (mirroring their own process's steps), so the string is
    /// resolved once per process rather than once for the whole diagram.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build_process_step_connector_classes<'id>(
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
        css_theme_vars: &mut CssThemeVars,
        process_render_expanded: bool,
        focus_mode: TailwindFocusMode<'_, 'id>,
        process_id: &Id<'id>,
        process_diagram: &ProcessDiagram<'id>,
    ) -> String {
        let entity_type = EntityType::DependencyEdgeDefault;
        let mut tailwind_class_state = TailwindClassState {
            entity_type: Some(entity_type.clone()),
            ..Default::default()
        };

        // 1. Base EdgeDefaults (lowest priority).
        if let Some(defaults_partials) = theme_default.base_styles.get(&IdOrDefaults::EdgeDefaults)
        {
            Self::apply_tailwind_from_partials(
                defaults_partials,
                &theme_default.style_aliases,
                &mut tailwind_class_state,
            );
        }

        // 2. Dependency-edge entity-type EdgeDefaults.
        let type_id = EntityTypeId::from(entity_type.into_id());
        if let Some(type_partials) = theme_types_styles
            .get(&type_id)
            .and_then(|type_styles| type_styles.get(&IdOrDefaults::EdgeDefaults))
        {
            Self::apply_tailwind_from_partials(
                type_partials,
                &theme_default.style_aliases,
                &mut tailwind_class_state,
            );
        }

        // Must run before `write_classes` so the `invisible` override
        // replaces (not duplicates alongside) the resolved default `visible`
        // attribute.
        Self::process_step_visibility_attr_write(
            &mut tailwind_class_state,
            process_render_expanded,
            focus_mode,
            process_id,
        );

        let mut classes = String::new();
        tailwind_class_state.write_classes(
            &mut classes,
            css_theme_vars,
            theme_default.dark_mode_config.shade,
        );

        Self::process_step_visibility_reveal_classes_write(
            &mut classes,
            focus_mode,
            process_id,
            process_diagram.steps.keys(),
        );

        classes
    }

    // === Tailwind Attribute Resolution === //

    /// Resolve entity-type-based tailwind attribute overrides specific to an
    /// individual edge, without re-applying EdgeDefaults or any ID override.
    ///
    /// This is used when building an edge's classes on top of an
    /// already-resolved edge group state. Only the entity-type-specific
    /// styles for the individual edge are applied; EdgeDefaults are
    /// intentionally skipped because they are already present in the cloned
    /// group state, and ID overrides (both the edge group's and this edge's
    /// own) are applied afterwards by the caller via
    /// [`Self::apply_id_specific_partials`], so they always win over
    /// entity-type styling regardless of tier.
    ///
    /// # Parameters
    ///
    /// * `edge_id`: The individual edge ID (e.g., `edge_group_id__0`).
    /// * `entity_types`: The entity types map.
    /// * `theme_default`: The theme defaults.
    /// * `theme_types_styles`: Styles defined per entity type.
    /// * `tailwind_class_state`: The cloned edge group state to apply overrides
    ///   onto.
    fn resolve_tailwind_attrs_edge_types<'partials, 'tw_state, 'id>(
        edge_id: &Id<'id>,
        entity_types: &'partials EntityTypes<'id>,
        theme_default: &'partials ThemeDefault<'id>,
        theme_types_styles: &'partials ThemeTypesStyles<'id>,
        tailwind_class_state: &mut TailwindClassState<'tw_state>,
    ) where
        'partials: 'tw_state,
    {
        // Apply entity type styles for the edge ID.
        if let Some(types) = entity_types.get(edge_id) {
            types
                .iter()
                .filter_map(|entity_type| {
                    let type_id = EntityTypeId::from(entity_type.clone().into_id());
                    theme_types_styles
                        .get(&type_id)
                        .and_then(|type_styles| type_styles.get(&IdOrDefaults::EdgeDefaults))
                })
                .for_each(|type_partials| {
                    Self::apply_tailwind_from_partials(
                        type_partials,
                        &theme_default.style_aliases,
                        tailwind_class_state,
                    );
                });
        }
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
    /// * `apply_id_override`: Whether to apply the entity ID's own style
    ///   override (step 3). Edge groups pass `false` here and apply their ID
    ///   override later via [`Self::apply_id_specific_partials`], after the
    ///   individual edge's own entity types have been layered on -- so the
    ///   override isn't shadowed by them. All other callers pass `true`.
    /// * `state`: Tailwind class state to write the resolved classes to.
    #[allow(clippy::too_many_arguments)]
    fn resolve_tailwind_attrs<'partials, 'tw_state, 'id>(
        entity_id: &Id<'id>,
        entity_types: &'partials EntityTypes<'id>,
        theme_default: &'partials ThemeDefault<'id>,
        theme_types_styles: &'partials ThemeTypesStyles<'id>,
        id_or_defaults_key: IdOrDefaults<'id>,
        apply_id_override: bool,
        tailwind_class_state: &mut TailwindClassState<'tw_state>,
    ) where
        'partials: 'tw_state,
    {
        // 1. Start with NodeDefaults/EdgeDefaults (lowest priority)
        if let Some(defaults_partials) = theme_default.base_styles.get(&id_or_defaults_key) {
            Self::apply_tailwind_from_partials(
                defaults_partials,
                &theme_default.style_aliases,
                tailwind_class_state,
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
                        tailwind_class_state,
                    );
                });
        }

        // 3. Apply the entity ID itself (highest priority), unless deferred.
        if apply_id_override {
            Self::apply_id_specific_partials(entity_id, theme_default, tailwind_class_state);
        }
    }

    /// Applies the style override registered for a specific entity ID (via
    /// `theme_default.base_styles.get(IdOrDefaults::Id(entity_id))`), if any.
    ///
    /// This is the highest-priority style layer for a given entity -- it
    /// should always be applied after any entity-type-based styling for that
    /// entity, so explicit per-ID customisation always wins.
    fn apply_id_specific_partials<'partials, 'tw_state, 'id>(
        entity_id: &Id<'id>,
        theme_default: &'partials ThemeDefault<'id>,
        tailwind_class_state: &mut TailwindClassState<'tw_state>,
    ) where
        'partials: 'tw_state,
    {
        if let Some(partials) = theme_default
            .base_styles
            .get(&IdOrDefaults::Id(entity_id.clone()))
        {
            Self::apply_tailwind_from_partials(
                partials,
                &theme_default.style_aliases,
                tailwind_class_state,
            );
        }
    }

    /// Apply tailwind attribute values from CssClassPartials.
    fn apply_tailwind_from_partials<'partials, 'tw_state, 'id>(
        partials: &'partials CssClassPartials<'id>,
        style_aliases: &'partials StyleAliases<'id>,
        tailwind_class_state: &mut TailwindClassState<'tw_state>,
    ) where
        'partials: 'tw_state,
    {
        // First, check style_aliases_applied (lower priority within this partials)
        partials
            .style_aliases_applied()
            .iter()
            .filter_map(|alias| style_aliases.get(alias))
            .for_each(|alias_partials| {
                Self::extract_tailwind_from_map(alias_partials, tailwind_class_state)
            });

        // Then, check direct attributes (higher priority within this partials)
        Self::extract_tailwind_from_map(partials, tailwind_class_state);
    }

    /// Extract tailwind attribute values from a CssClassPartials map.
    fn extract_tailwind_from_map<'partials, 'tw_state, 'id>(
        partials: &'partials CssClassPartials<'id>,
        tailwind_class_state: &mut TailwindClassState<'tw_state>,
    ) where
        'partials: 'tw_state,
    {
        partials.iter().for_each(|(theme_attr, value)| {
            tailwind_class_state
                .attrs
                .insert(*theme_attr, Cow::Borrowed(value));
        });
    }
}
