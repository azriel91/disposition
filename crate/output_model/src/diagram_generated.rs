use std::time::Duration;

use disposition_input_ir_model::issue::ModelToIrIssue;
use disposition_input_model::InputDiagram;
use disposition_ir_model::IrDiagram;
use disposition_svg_model::{EdgeRoutingDiagnostics, SvgElements};
use disposition_taffy_model::TaffyNodeMappings;

/// All intermediate and final transformations produced by generating a diagram.
///
/// This is the output of `DiagramGenerator::generate` (in
/// `disposition_input_ir_rt`), which runs the full diagram generation pipeline.
/// Each transformation is paired with the wall-clock [`Duration`] taken to
/// produce it, so consumers can surface timing information.
///
/// All fields are owned with a `'static` lifetime, as the pipeline operates on
/// a `'static` input diagram.
#[derive(Clone, Debug, PartialEq)]
pub struct DiagramGenerated {
    /// The user's input diagram merged over `InputDiagram::base()`, produced by
    /// `InputDiagramMerger::merge`.
    pub input_diagram_merged: InputDiagram<'static>,
    /// Time taken to merge the input diagram over the base diagram.
    pub input_diagram_merged_merge_duration: Duration,
    /// The intermediate representation diagram, produced by
    /// `InputToIrDiagramMapper::map`.
    pub ir_diagram: IrDiagram<'static>,
    /// Issues encountered while mapping the input diagram to the IR diagram.
    ///
    /// These are not errors -- generation still succeeds -- but consumers may
    /// wish to surface them as warnings.
    pub ir_diagram_issues: Vec<ModelToIrIssue>,
    /// Time taken to map the merged input diagram to the IR diagram.
    pub ir_diagram_map_duration: Duration,
    /// The taffy layout node mappings, produced by `IrToTaffyBuilder`.
    pub taffy_node_mappings: TaffyNodeMappings<'static>,
    /// Time taken to build the taffy node mappings.
    pub taffy_node_mappings_build_duration: Duration,
    /// The SVG elements, produced by `TaffyToSvgElementsMapper::map`.
    pub svg_elements: SvgElements<'static>,
    /// Diagnostic snapshot of the edge-routing calculation, produced
    /// alongside `svg_elements` by `TaffyToSvgElementsMapper`.
    ///
    /// Captures the pass-1, offset, slot-index, rank-gap, and protrusion
    /// values the edge router computes internally. Nothing in the render
    /// pipeline reads it back; it exists for inspection / diagnosis.
    pub edge_routing_diagnostics: EdgeRoutingDiagnostics<'static>,
    /// Time taken to map the taffy node mappings to SVG elements.
    ///
    /// This covers both `svg_elements` and `edge_routing_diagnostics`, which
    /// are produced together in the same mapping pass.
    pub svg_elements_map_duration: Duration,
    /// The final SVG markup, produced by
    /// `SvgElementsToSvgMapper::map_with_input`.
    pub svg: String,
    /// Time taken to map the SVG elements to the final SVG.
    pub svg_map_duration: Duration,
}
