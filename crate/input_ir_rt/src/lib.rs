//! Logic to map `disposition` input model to intermediate representation.

pub use crate::{
    entity_highlighted_spans::EntityHighlightedSpans,
    input_to_ir_diagram_mapper::InputToIrDiagramMapper, ir_to_taffy_builder::IrToTaffyBuilder,
};

mod entity_highlighted_spans;
mod input_to_ir_diagram_mapper;
mod ir_to_taffy_builder;
