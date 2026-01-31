use disposition_ir_model::IrDiagram;
use disposition_svg_model::SvgElements;
use disposition_taffy_model::TaffyNodeMappings;

/// Maps the IR diagram and `TaffyNodeMappings` to SVG elements.
///
/// These include nodes with simple coordinates and edges.
#[derive(Clone, Copy, Debug)]
pub struct TaffyToSvgElementsMapper;

impl TaffyToSvgElementsMapper {
    pub fn map<'id>(
        ir_diagram: &IrDiagram<'id>,
        taffy_node_mappings: &TaffyNodeMappings<'id>,
    ) -> SvgElements<'id> {
        todo!();
    }
}
