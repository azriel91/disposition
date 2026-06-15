#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

use disposition_input_ir_model::{EdgeAnimationActive, IrDiagramAndIssues};
use disposition_input_model::InputDiagram;
use disposition_output_model::DiagramGenerated;
use disposition_taffy_model::DimensionAndLod;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

use crate::{
    DiagramGenerateError, InputDiagramMerger, InputToIrDiagramMapper, IrToTaffyBuilder,
    SvgElementsToSvgMapper, TaffyToSvgElementsMapper,
};

/// Runs the full diagram generation pipeline.
///
/// This calls each processor in order -- `InputDiagramMerger`,
/// `InputToIrDiagramMapper`, `IrToTaffyBuilder`, `TaffyToSvgElementsMapper`, and
/// `SvgElementsToSvgMapper` -- and collects every intermediate and final
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
        let svg_elements =
            TaffyToSvgElementsMapper::map(&ir_diagram, &taffy_node_mappings, edge_animation_active);
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
            svg_elements_map_duration,
            svg,
            svg_map_duration,
        })
    }
}
