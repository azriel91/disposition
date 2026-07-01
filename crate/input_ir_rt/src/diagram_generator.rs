#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

use disposition_input_ir_model::{EdgeAnimationActive, IrDiagramAndIssues};
use disposition_input_model::{DiagramFocus, InputDiagram};
use disposition_output_model::{DiagramFocusGenerated, DiagramGenerated};
use disposition_taffy_model::DimensionAndLod;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

use crate::{
    input_to_ir_diagram_mapper::tailwind_focus_mode::TailwindFocusMode, DiagramGenerateError,
    InputDiagramMerger, InputToIrDiagramMapper, IrToTaffyBuilder, SvgElementsToSvgMapper,
    TaffyToSvgElementsMapper, TaffyToSvgElementsOutcome,
};

/// Runs the full diagram generation pipeline.
///
/// This calls each processor in order -- `InputDiagramMerger`,
/// `InputToIrDiagramMapper`, `IrToTaffyBuilder`, `TaffyToSvgElementsMapper`,
/// and `SvgElementsToSvgMapper` -- and collects every intermediate and final
/// transformation, along with the time taken for each step, into a
/// [`DiagramGenerated`].
#[derive(Clone, Copy, Debug)]
pub struct DiagramGenerator;

impl DiagramGenerator {
    /// Generates a diagram from the given input diagram.
    ///
    /// The input diagram is first merged over `InputDiagram::base()`, then
    /// mapped through the IR, taffy layout, SVG elements, and finally the SVG
    /// markup. The original (unmerged) `input_diagram` is embedded as the
    /// `<source>` of the generated SVG.
    ///
    /// # Parameters
    ///
    /// * `input_diagram`: The user's input diagram to generate from.
    /// * `edge_animation_active`: When edge animations should be active in the
    ///   generated SVG elements.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use disposition_input_ir_rt::{DiagramGenerator, EdgeAnimationActive};
    /// # use disposition_input_model::InputDiagram;
    /// #
    /// let input_diagram = InputDiagram::base();
    /// let diagram_generated =
    ///     DiagramGenerator::generate(&input_diagram, EdgeAnimationActive::OnProcessStepFocus)?;
    /// # Ok::<(), disposition_input_ir_rt::DiagramGenerateError>(())
    /// ```
    pub fn generate(
        input_diagram: &InputDiagram<'static>,
        edge_animation_active: EdgeAnimationActive,
    ) -> Result<DiagramGenerated, DiagramGenerateError> {
        // === Merge input diagram over base === //
        let input_diagram_merged_merge_start = Instant::now();
        let input_diagram_merged = InputDiagramMerger::merge(InputDiagram::base(), input_diagram);
        let input_diagram_merged_merge_duration = input_diagram_merged_merge_start.elapsed();

        // === Map merged input diagram to IR diagram === //
        let ir_diagram_map_start = Instant::now();
        let IrDiagramAndIssues {
            diagram: ir_diagram,
            issues: ir_diagram_issues,
        } = InputToIrDiagramMapper::map(&input_diagram_merged);
        let ir_diagram_map_duration = ir_diagram_map_start.elapsed();

        // === Build taffy node mappings === //
        //
        // `ir_diagram` is bound before building the taffy tree so it outlives
        // the borrow held by the builder iterator, and is only moved into
        // `DiagramGenerated` at the end.
        let taffy_node_mappings_build_start = Instant::now();
        let taffy_node_mappings = IrToTaffyBuilder::builder()
            .with_ir_diagram(&ir_diagram)
            .with_dimension_and_lods(vec![DimensionAndLod::default_no_limit()])
            .build()
            .build()?
            .next()
            .ok_or(DiagramGenerateError::NoTaffyMappings)?;
        let taffy_node_mappings_build_duration = taffy_node_mappings_build_start.elapsed();

        // === Map taffy node mappings to SVG elements === //
        let svg_elements_map_start = Instant::now();
        let TaffyToSvgElementsOutcome {
            svg_elements,
            edge_routing_diagnostics,
        } = TaffyToSvgElementsMapper::map_with_diagnostics(
            &ir_diagram,
            &taffy_node_mappings,
            edge_animation_active,
        );
        let svg_elements_map_duration = svg_elements_map_start.elapsed();

        // === Map SVG elements to SVG markup === //
        let svg_map_start = Instant::now();
        let svg = SvgElementsToSvgMapper::map_with_input(input_diagram, &svg_elements);
        let svg_map_duration = svg_map_start.elapsed();

        Ok(DiagramGenerated {
            input_diagram_merged,
            input_diagram_merged_merge_duration,
            ir_diagram,
            ir_diagram_issues,
            ir_diagram_map_duration,
            taffy_node_mappings,
            taffy_node_mappings_build_duration,
            svg_elements,
            edge_routing_diagnostics,
            svg_elements_map_duration,
            svg,
            svg_map_duration,
        })
    }

    /// Generates one diagram per focus state, with the focused entity's styles
    /// baked in statically.
    ///
    /// Instead of emitting interactive `peer` / `group-has` CSS classes (as
    /// [`Self::generate`] does), this produces a separate [`DiagramGenerated`]
    /// for each [`DiagramFocus`], in the following order:
    ///
    /// 1. Nothing focused (always present, even with no processes or tags).
    /// 2. For each process (declaration order): the process focused, then each
    ///    of its steps focused (declaration order).
    /// 3. For each tag (declaration order): the tag focused.
    ///
    /// The focus-independent pipeline steps (input merge, IR structure, and
    /// taffy layout) are computed once and reused across every focus -- only
    /// the tailwind classes, SVG elements, and SVG markup are recomputed
    /// per focus.
    ///
    /// # Parameters
    ///
    /// * `input_diagram`: The user's input diagram to generate from.
    /// * `edge_animation_active`: When edge animations should be active in the
    ///   generated SVG elements.
    pub fn generate_per_process_step_or_tag(
        input_diagram: &InputDiagram<'static>,
        edge_animation_active: EdgeAnimationActive,
    ) -> Result<Vec<DiagramFocusGenerated>, DiagramGenerateError> {
        // === Merge input diagram over base (once) === //
        let input_diagram_merged_merge_start = Instant::now();
        let input_diagram_merged = InputDiagramMerger::merge(InputDiagram::base(), input_diagram);
        let input_diagram_merged_merge_duration = input_diagram_merged_merge_start.elapsed();

        // === Map merged input diagram to the focus-independent IR (once) === //
        let ir_structure_map_start = Instant::now();
        let IrDiagramAndIssues {
            diagram: ir_diagram_structure,
            issues: ir_diagram_issues,
        } = InputToIrDiagramMapper::map_structure(&input_diagram_merged);
        let ir_structure_map_duration = ir_structure_map_start.elapsed();

        // === Build taffy node mappings (once) === //
        //
        // The taffy layout is focus-independent: focus only changes
        // colour/visibility/animation, never node positions or sizes.
        let taffy_node_mappings_build_start = Instant::now();
        let taffy_node_mappings = IrToTaffyBuilder::builder()
            .with_ir_diagram(&ir_diagram_structure)
            .with_dimension_and_lods(vec![DimensionAndLod::default_no_limit()])
            .build()
            .build()?
            .next()
            .ok_or(DiagramGenerateError::NoTaffyMappings)?;
        let taffy_node_mappings_build_duration = taffy_node_mappings_build_start.elapsed();

        // === Generate one diagram per focus state === //
        let diagrams_focus_generated = Self::focuses_collect(&input_diagram_merged)
            .into_iter()
            .map(|focus| {
                let focus_mode = TailwindFocusMode::Baked { active: &focus };

                // Apply the focus-dependent tailwind classes to a fresh clone of
                // the structural IR.
                let ir_diagram_map_start = Instant::now();
                let mut ir_diagram = ir_diagram_structure.clone();
                InputToIrDiagramMapper::tailwind_classes_apply(
                    &mut ir_diagram,
                    &input_diagram_merged,
                    focus_mode,
                );
                let ir_diagram_map_duration =
                    ir_structure_map_duration + ir_diagram_map_start.elapsed();

                // Map to SVG elements, baking the focus into edge animation.
                let svg_elements_map_start = Instant::now();
                let TaffyToSvgElementsOutcome {
                    svg_elements,
                    edge_routing_diagnostics,
                } = TaffyToSvgElementsMapper::map_with_focus(
                    &ir_diagram,
                    &taffy_node_mappings,
                    edge_animation_active,
                    focus_mode,
                );
                let svg_elements_map_duration = svg_elements_map_start.elapsed();

                // Map to SVG markup.
                let svg_map_start = Instant::now();
                let svg = SvgElementsToSvgMapper::map_with_input(input_diagram, &svg_elements);
                let svg_map_duration = svg_map_start.elapsed();

                let diagram_generated = DiagramGenerated {
                    input_diagram_merged: input_diagram_merged.clone(),
                    input_diagram_merged_merge_duration,
                    ir_diagram,
                    ir_diagram_issues: ir_diagram_issues.clone(),
                    ir_diagram_map_duration,
                    taffy_node_mappings: taffy_node_mappings.clone(),
                    taffy_node_mappings_build_duration,
                    svg_elements,
                    edge_routing_diagnostics,
                    svg_elements_map_duration,
                    svg,
                    svg_map_duration,
                };

                DiagramFocusGenerated {
                    focus,
                    diagram_generated,
                }
            })
            .collect::<Vec<_>>();

        Ok(diagrams_focus_generated)
    }

    /// Collects the ordered focus states to generate diagrams for.
    ///
    /// The order is: nothing focused, then each process followed by its steps,
    /// then each tag -- all in declaration (insertion) order. See
    /// [`Self::generate_per_process_step_or_tag`].
    fn focuses_collect(input_diagram: &InputDiagram<'static>) -> Vec<DiagramFocus<'static>> {
        let process_step_count: usize = input_diagram
            .processes
            .values()
            .map(|process_diagram| process_diagram.steps.len())
            .sum();
        let mut focuses = Vec::with_capacity(
            1 + input_diagram.processes.len() + process_step_count + input_diagram.tags.len(),
        );

        // 1. Nothing focused.
        focuses.push(DiagramFocus::None);

        // 2. Each process, then each of its steps.
        input_diagram
            .processes
            .iter()
            .for_each(|(process_id, process_diagram)| {
                focuses.push(DiagramFocus::Process(process_id.clone()));
                process_diagram.steps.keys().for_each(|process_step_id| {
                    focuses.push(DiagramFocus::ProcessStep {
                        process_id: process_id.clone(),
                        process_step_id: process_step_id.clone(),
                    });
                });
            });

        // 3. Each tag.
        input_diagram.tags.keys().for_each(|tag_id| {
            focuses.push(DiagramFocus::Tag(tag_id.clone()));
        });

        focuses
    }
}
