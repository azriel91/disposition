use disposition_ir_model::IrDiagram;
use disposition_taffy_model::TaffyNodeMappings;

#[derive(Clone, Copy, Debug)]
pub struct TaffyToSvgMapper;

impl TaffyToSvgMapper {
    pub fn map(_ir_diagram: &IrDiagram, _taffy_node_mappings: TaffyNodeMappings) -> String {
        let mut buffer = String::with_capacity(2048);

        todo!("write each SVG element to buffer");

        buffer
    }
}
