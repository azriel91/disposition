use std::fmt::Write;

use disposition_ir_model::node::{NodeId, NodeShape};
use disposition_model_common::Map;
use disposition_svg_model::SvgProcessInfo;

use crate::taffy_to_svg_elements_mapper::{
    process_step_heights::{self, ProcessStepsHeight},
    StringCharReplacer, SvgNodeRectPathBuilder,
};

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
        writeln!(&mut classes, "[&>path]:[d:path('{path_d}')]").unwrap();

        classes
    }

    /// Builds the translation tailwind classes for a process or process step
    /// node.
    ///
    /// This creates:
    /// 1. A `translate-x-*` class for horizontal positioning
    /// 2. A base `translate-y-*` class for the collapsed state
    /// 3. `group-has-[#id:focus-within]:translate-y-[..]` classes for when
    ///    previous processes are focused
    /// 4. transition-transform and duration classes for smooth animation
    /// 5. `[d:path(..)]` classes for collapsed and expanded path shapes
    #[allow(clippy::too_many_arguments)]
    fn build_for_process(
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
            "[&>path]:[d:path('{path_d_collapsed_escaped}')]"
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
                "group-has-[#{process_id}:focus-within]:[&>path]:[d:path('{path_d_expanded_escaped}')]"
            )
            .unwrap();

            // Add classes for when any of the process's steps are focused
            process_step_ids.iter().for_each(|process_step_id| {
                writeln!(
                    &mut classes,
                    "group-has-[#{process_step_id}:focus-within]:[&>path]:[d:path('{path_d_expanded_escaped}')]"
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
}
