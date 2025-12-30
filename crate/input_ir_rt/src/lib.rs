//! Logic to map `disposition` input model to intermediate representation.

pub use crate::{
    input_to_ir_diagram_mapper::InputToIrDiagramMapper, ir_to_taffy_builder::IrToTaffyBuilder,
    taffy_to_svg_mapper::TaffyToSvgMapper,
};

mod input_to_ir_diagram_mapper;
mod ir_to_taffy_builder;
mod taffy_to_svg_mapper;
