use std::fmt::Write;

use disposition_input_model::DiagramFocus;
use disposition_ir_model::{entity::EntityTailwindClasses, IrDiagram};

use crate::{
    input_to_ir_diagram_mapper::tailwind_focus_mode::TailwindFocusMode,
    taffy_to_svg_elements_mapper::{
        process_step_heights::{self, ProcessStepsHeight},
        NodeIdToSvgProcessInfo,
    },
};

/// Builds the delta-based `translate-y` tailwind classes for process step
/// connectors, so they shift by the same amount their own process's step
/// nodes shift when the process collapses/expands.
///
/// Unlike [`SvgNodeTranslateClassesBuilder`](super::SvgNodeTranslateClassesBuilder),
/// which computes an absolute `translate-y` target (node paths are in local
/// coordinates), connector paths are already baked in absolute pixel
/// coordinates (see
/// [`ProcessStepGraphEdgesBuilder`](super::process_step_graph_edges_builder::ProcessStepGraphEdgesBuilder)),
/// so this builder emits a relative delta instead:
/// `-predecessors_cumulative_height(...)` by default, adjusted by
/// `+ prev_total_height` under each earlier process's `group-has` reveal.
#[derive(Clone, Copy, Debug)]
pub(super) struct SvgConnectorTranslateClassesBuilder;

impl SvgConnectorTranslateClassesBuilder {
    /// Appends translate-y classes onto every process step connector's
    /// existing (Pass-A-populated) tailwind class entry.
    ///
    /// No-op when `process_steps_heights` is empty -- which happens exactly
    /// when processes render fully expanded (see
    /// `TaffyToSvgElementsMapper::map_with_focus`), so connectors keep their
    /// untranslated, already-correct absolute coordinates.
    pub(super) fn build<'id>(
        ir_diagram: &IrDiagram<'id>,
        process_steps_heights: &[ProcessStepsHeight<'id>],
        svg_process_infos: &NodeIdToSvgProcessInfo<'id>,
        focus_mode: TailwindFocusMode<'_, 'id>,
        tailwind_classes: &mut EntityTailwindClasses<'id>,
    ) {
        if process_steps_heights.is_empty() {
            return;
        }

        ir_diagram
            .process_step_graphs
            .iter()
            .for_each(|(process_node_id, process_step_graph)| {
                if process_step_graph.edges.is_empty() {
                    return;
                }
                let Some(svg_process_info) = svg_process_infos.get(process_node_id) else {
                    return;
                };

                let translate_classes = Self::translate_classes_build(
                    process_steps_heights,
                    svg_process_info.process_index,
                    focus_mode,
                );

                process_step_graph.edges.iter().for_each(|edge| {
                    let edge_id = edge.edge_id().into_inner();
                    if let Some(existing_classes) = tailwind_classes.get_mut(&edge_id) {
                        existing_classes.push(' ');
                        existing_classes.push_str(&translate_classes);
                    } else {
                        tailwind_classes.insert(edge_id, translate_classes.clone());
                    }
                });
            });
    }

    /// Builds the delta translate-y classes for every connector of a single
    /// process, dispatching on `TailwindFocusMode`. Computed once per
    /// process and reused for all its connectors (see [`Self::build`]).
    fn translate_classes_build<'id>(
        process_steps_heights: &[ProcessStepsHeight<'id>],
        process_index: usize,
        focus_mode: TailwindFocusMode<'_, 'id>,
    ) -> String {
        let delta_y_default = -process_step_heights::predecessors_cumulative_height(
            process_steps_heights,
            process_index,
        );

        match focus_mode {
            TailwindFocusMode::Interactive => Self::translate_classes_interactive_build(
                process_steps_heights,
                process_index,
                delta_y_default,
            ),
            TailwindFocusMode::Baked { active } => Self::translate_classes_baked_build(
                process_steps_heights,
                process_index,
                delta_y_default,
                active,
            ),
        }
    }

    /// Interactive-mode translate classes: a default delta, plus one
    /// `group-has-[...]:translate-y-[...]` override per earlier process (and
    /// per earlier process's own steps), mirroring
    /// `SvgNodeTranslateClassesBuilder::build_for_process_interactive`'s
    /// predecessor loop.
    fn translate_classes_interactive_build<'id>(
        process_steps_heights: &[ProcessStepsHeight<'id>],
        process_index: usize,
        delta_y_default: f32,
    ) -> String {
        let mut classes = String::new();
        writeln!(&mut classes, "translate-y-[{delta_y_default}px]").unwrap();

        (0..process_index).for_each(|prev_idx| {
            let ProcessStepsHeight {
                process_id,
                process_step_ids,
                total_height,
            } = &process_steps_heights[prev_idx];
            let delta_y_revealed = delta_y_default + total_height;

            writeln!(
                &mut classes,
                "group-has-[#{process_id}:focus-within]:translate-y-[{delta_y_revealed}px]"
            )
            .unwrap();

            process_step_ids.iter().for_each(|process_step_id| {
                writeln!(
                    &mut classes,
                    "group-has-[#{process_step_id}:focus-within]:translate-y-[{delta_y_revealed}px]"
                )
                .unwrap();
            });
        });

        classes
    }

    /// Baked-mode translate class: a single resolved delta, adding back the
    /// total height of every earlier process that `active` targets.
    fn translate_classes_baked_build<'id>(
        process_steps_heights: &[ProcessStepsHeight<'id>],
        process_index: usize,
        delta_y_default: f32,
        active: &DiagramFocus<'id>,
    ) -> String {
        let delta_y_offset = (0..process_index)
            .map(|prev_idx| &process_steps_heights[prev_idx])
            .filter(|prev| process_step_heights::focus_active_targets_process(active, prev))
            .map(|prev| prev.total_height)
            .sum::<f32>();

        let mut classes = String::new();
        writeln!(
            &mut classes,
            "translate-y-[{}px]",
            delta_y_default + delta_y_offset
        )
        .unwrap();
        classes
    }
}
