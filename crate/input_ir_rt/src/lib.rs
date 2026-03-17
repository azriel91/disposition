//! Logic to map `disposition` input model to intermediate representation.

pub use disposition_input_ir_model::EdgeAnimationActive;
pub use disposition_input_rt::id_parse;

pub use crate::{
    input_diagram_merger::InputDiagramMerger,
    input_diagram_theme_sources::InputDiagramThemeSources,
    input_to_ir_diagram_mapper::InputToIrDiagramMapper, ir_to_taffy_builder::IrToTaffyBuilder,
    svg_elements_to_svg_mapper::SvgElementsToSvgMapper,
    taffy_to_svg_elements_mapper::TaffyToSvgElementsMapper, theme_value_source::ThemeValueSource,
};

// Used by `cosmic-text` for calculating text layout, and `base64` for encoding
// the font data.
const NOTO_SANS_MONO_TTF: &[u8] =
    include_bytes!("../fonts/noto_sans_mono/NotoSansMono-Regular.ttf");

mod input_diagram_merger;
mod input_diagram_theme_sources;
mod input_to_ir_diagram_mapper;
mod ir_to_taffy_builder;
mod svg_elements_to_svg_mapper;
mod taffy_to_svg_elements_mapper;
mod theme_value_source;
