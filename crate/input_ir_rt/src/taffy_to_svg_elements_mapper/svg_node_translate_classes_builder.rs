use std::fmt::Write;

use disposition_input_model::DiagramFocus;
use disposition_ir_model::node::{NodeId, NodeShape};
use disposition_model_common::Map;
use disposition_svg_model::SvgProcessInfo;

use crate::{
    input_to_ir_diagram_mapper::tailwind_focus_mode::TailwindFocusMode,
    taffy_to_svg_elements_mapper::{
        process_step_heights::{self, ProcessStepsHeight},
        StringCharReplacer, SvgNodeRectPathBuilder,
    },
};

/// Tailwind qualifier class for the SVG element representing the wrapper taffy
/// node's coordinates.
const TW_QUALIFIER_NODE_WRAPPER: &str = "[&>path.wrapper]";

/// Builds translate-x and translate-y tailwind classes for nodes.
///
/// * Process nodes will have classes that collapse depending on focus on them
///   or their steps.
/// * Non-process nodes will have simple translate-x and translate-y classes.
#[derive(Clone, Copy, Debug)]
pub struct SvgNodeTranslateClassesBuilder;

impl SvgNodeTranslateClassesBuilder {
    /// Builds translate-x and translate-y tailwind classes for a node.
    ///
    /// * Process nodes will have classes that collapse depending on focus on
    ///   them or their steps.
    /// * Non-process nodes will have simple translate-x and translate-y
    ///   classes.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn build<'id>(
        process_steps_heights: &[ProcessStepsHeight<'_>],
        process_infos: &Map<NodeId<'id>, SvgProcessInfo<'id>>,
        x: f32,
        y: f32,
        process_id: &Option<NodeId<'_>>,
        width: f32,
        height_expanded: f32,
        height_to_expand_to: Option<f32>,
        node_shape: &NodeShape,
        path_d_collapsed: &str,
        focus_mode: TailwindFocusMode<'_, 'id>,
    ) -> String {
        if let Some(ref proc_id) = *process_id
            && let Some(proc_info) = process_infos.get(proc_id)
        {
            // Calculate base_y for this specific node
            let process_steps_height_predecessors_cumulative =
                process_step_heights::predecessors_cumulative_height(
                    process_steps_heights,
                    proc_info.process_index,
                );
            let base_y = y - process_steps_height_predecessors_cumulative;

            // Build path_d_expanded for this node if it's a process
            let path_d_expanded = if height_to_expand_to.is_some() {
                SvgNodeRectPathBuilder::build(width, height_expanded, node_shape)
            } else {
                path_d_collapsed.to_string()
            };

            Self::build_for_process(
                x,
                base_y,
                path_d_collapsed,
                height_to_expand_to,
                &path_d_expanded,
                proc_info.process_index,
                process_steps_heights,
                focus_mode,
            )
        } else {
            Self::build_for_node(x, y, path_d_collapsed)
        }
    }

    /// Builds simple translate-x and translate-y tailwind classes for
    /// non-process/step nodes.
    fn build_for_node(x: f32, y: f32, path_d_collapsed: &str) -> String {
        let mut classes = String::new();
        writeln!(&mut classes, "translate-x-[{x}px]").unwrap();
        writeln!(&mut classes, "translate-y-[{y}px]").unwrap();

        let mut path_d = path_d_collapsed.to_string();
        StringCharReplacer::replace_inplace(&mut path_d, ' ', '_');

        // The `d` attribute only applies to the wrapper `<path>`, not the `<path>`
        // representing the circle.
        writeln!(
            &mut classes,
            "{TW_QUALIFIER_NODE_WRAPPER}:[d:path('{path_d}')]"
        )
        .unwrap();

        classes
    }

    /// Builds the translation tailwind classes for a process or process step
    /// node, dispatching on the [`TailwindFocusMode`].
    #[allow(clippy::too_many_arguments)]
    fn build_for_process<'id>(
        x: f32,
        base_y: f32,
        path_d_collapsed: &str,
        height_to_expand_to: Option<f32>,
        path_d_expanded: &str,
        process_index: usize,
        process_steps_height: &[ProcessStepsHeight<'id>],
        focus_mode: TailwindFocusMode<'_, 'id>,
    ) -> String {
        match focus_mode {
            TailwindFocusMode::Interactive => Self::build_for_process_interactive(
                x,
                base_y,
                path_d_collapsed,
                height_to_expand_to,
                path_d_expanded,
                process_index,
                process_steps_height,
            ),
            TailwindFocusMode::Baked { active } => Self::build_for_process_baked(
                x,
                base_y,
                path_d_collapsed,
                height_to_expand_to,
                path_d_expanded,
                process_index,
                process_steps_height,
                active,
            ),
        }
    }

    /// Builds the interactive translation tailwind classes for a process node.
    ///
    /// This creates:
    /// 1. A `translate-x-*` class for horizontal positioning
    /// 2. A base `translate-y-*` class for the collapsed state
    /// 3. `group-has-[#id:focus-within]:translate-y-[..]` classes for when
    ///    previous processes are focused
    /// 4. transition-transform and duration classes for smooth animation
    /// 5. `[d:path(..)]` classes for collapsed and expanded path shapes
    fn build_for_process_interactive(
        x: f32,
        base_y: f32,
        path_d_collapsed: &str,
        height_to_expand_to: Option<f32>,
        path_d_expanded: &str,
        process_index: usize,
        process_steps_height: &[ProcessStepsHeight],
    ) -> String {
        let mut classes = String::new();

        // Add translate-x for horizontal positioning
        writeln!(&mut classes, "translate-x-[{x}px]").unwrap();

        // Build path d attribute with collapsed height
        let mut path_d_collapsed_escaped = path_d_collapsed.to_string();
        StringCharReplacer::replace_inplace(&mut path_d_collapsed_escaped, ' ', '_');
        writeln!(
            &mut classes,
            "{TW_QUALIFIER_NODE_WRAPPER}:[d:path('{path_d_collapsed_escaped}')]"
        )
        .unwrap();

        // When this process or any of its steps are focused, expand the height
        if height_to_expand_to.is_some() {
            let ProcessStepsHeight {
                process_id,
                process_step_ids,
                total_height: _,
            } = &process_steps_height[process_index];

            // Build path d attribute with expanded height
            let mut path_d_expanded_escaped = path_d_expanded.to_string();
            StringCharReplacer::replace_inplace(&mut path_d_expanded_escaped, ' ', '_');

            writeln!(
                &mut classes,
                "group-has-[#{process_id}:focus-within]:{TW_QUALIFIER_NODE_WRAPPER}:[d:path('{path_d_expanded_escaped}')]"
            )
            .unwrap();

            // Add classes for when any of the process's steps are focused
            process_step_ids.iter().for_each(|process_step_id| {
                writeln!(
                    &mut classes,
                    "group-has-[#{process_step_id}:focus-within]:{TW_QUALIFIER_NODE_WRAPPER}:[d:path('{path_d_expanded_escaped}')]"
                )
                .unwrap();
            });
        }

        // Add transition class for smooth animation
        writeln!(&mut classes, "transition-all").unwrap();
        writeln!(&mut classes, "duration-200").unwrap();

        // Base translate-y for collapsed state
        writeln!(&mut classes, "translate-y-[{base_y}px]").unwrap();

        // For each previous process, add a class that moves this node down when that
        // process is focused
        (0..process_index).for_each(|prev_idx| {
            let process_steps_height_prev = &process_steps_height[prev_idx];
            let ProcessStepsHeight {
                process_id,
                process_step_ids,
                total_height,
            } = process_steps_height_prev;

            // When this previous process (or any of its steps) is focused,
            // we need to add back that process's steps' height
            let y_when_prev_focused = base_y + total_height;

            // Add class for when the process itself is focused
            writeln!(
                &mut classes,
                "group-has-[#{process_id}:focus-within]:translate-y-[{y_when_prev_focused}px]"
            )
            .unwrap();

            // Add classes for when any of the process's steps are focused
            process_step_ids.iter().for_each(|process_step_id| {
                writeln!(
                    &mut classes,
                    "group-has-[#{process_step_id}:focus-within]:translate-y-[{y_when_prev_focused}px]"
                )
                .unwrap();
            });
        });

        classes
    }

    /// Builds the baked translation tailwind classes for a process node.
    ///
    /// The expanded / translated state for the `active` focus is resolved
    /// directly (no `group-has-[...]` classes): the node uses its expanded path
    /// when it (or one of its steps) is the active focus, and its
    /// `translate-y` accounts for any earlier process that the active focus
    /// expands.
    #[allow(clippy::too_many_arguments)]
    fn build_for_process_baked<'id>(
        x: f32,
        base_y: f32,
        path_d_collapsed: &str,
        height_to_expand_to: Option<f32>,
        path_d_expanded: &str,
        process_index: usize,
        process_steps_height: &[ProcessStepsHeight<'id>],
        active: &DiagramFocus<'id>,
    ) -> String {
        let mut classes = String::new();

        // Add translate-x for horizontal positioning.
        writeln!(&mut classes, "translate-x-[{x}px]").unwrap();

        // Use the expanded path when this process (or one of its steps) is the
        // active focus, otherwise the collapsed path.
        let self_focused = height_to_expand_to.is_some()
            && Self::focus_active_targets_process(active, &process_steps_height[process_index]);
        let path_d = if self_focused {
            path_d_expanded
        } else {
            path_d_collapsed
        };
        let mut path_d_escaped = path_d.to_string();
        StringCharReplacer::replace_inplace(&mut path_d_escaped, ' ', '_');
        writeln!(
            &mut classes,
            "{TW_QUALIFIER_NODE_WRAPPER}:[d:path('{path_d_escaped}')]"
        )
        .unwrap();

        // When an earlier process (or one of its steps) is the active focus,
        // that process expands and pushes this node down by its steps' total
        // height.
        let y_offset_from_focused_predecessor = (0..process_index)
            .map(|prev_idx| &process_steps_height[prev_idx])
            .filter(|process_steps_height_prev| {
                Self::focus_active_targets_process(active, process_steps_height_prev)
            })
            .map(|process_steps_height_prev| process_steps_height_prev.total_height)
            .sum::<f32>();
        let y = base_y + y_offset_from_focused_predecessor;
        writeln!(&mut classes, "translate-y-[{y}px]").unwrap();

        classes
    }

    /// Returns whether `active` focuses the given process directly, or via one
    /// of its steps.
    fn focus_active_targets_process<'id>(
        active: &DiagramFocus<'id>,
        process_steps_height: &ProcessStepsHeight<'id>,
    ) -> bool {
        let ProcessStepsHeight {
            process_id,
            process_step_ids,
            ..
        } = process_steps_height;
        match active {
            DiagramFocus::Process(active_process_id) => {
                active_process_id.as_ref() == process_id.as_ref()
            }
            DiagramFocus::ProcessStep {
                process_step_id: active_step_id,
                ..
            } => process_step_ids
                .iter()
                .any(|process_step_id| process_step_id.as_ref() == active_step_id.as_ref()),
            DiagramFocus::None | DiagramFocus::Tag(_) => false,
        }
    }
}
