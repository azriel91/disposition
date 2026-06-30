//! Logic to map `disposition` input model to intermediate representation.

pub use disposition_input_ir_model::EdgeAnimationActive;
pub use disposition_input_rt::id_parse;

pub(crate) use crate::{
    absolute_coordinates::AbsoluteCoordinates,
    taffy_node_absolute_coordinates_calculator::TaffyNodeAbsoluteCoordinatesCalculator,
};
pub use crate::{
    diagram_generate_error::DiagramGenerateError,
    diagram_generator::DiagramGenerator,
    edge_face_assigner::EdgeFaceAssigner,
    edge_id_generator::EdgeIdGenerator,
    input_diagram_merger::InputDiagramMerger,
    input_diagram_theme_sources::InputDiagramThemeSources,
    input_to_ir_diagram_mapper::{
        tailwind_color_shade::{TailwindColorShade, TailwindColorShadeInvalid},
        tailwind_colors::{TailwindColor, TAILWIND_COLORS},
        InputToIrDiagramMapper,
    },
    ir_to_taffy_builder::IrToTaffyBuilder,
    node_ranks_calculator::NodeRanksCalculator,
    process_step_graph_calculator::ProcessStepGraphCalculator,
    string_xml_escaper::StringXmlEscaper,
    svg_elements_to_svg_mapper::SvgElementsToSvgMapper,
    taffy_to_svg_elements_mapper::{TaffyToSvgElementsMapper, TaffyToSvgElementsOutcome},
    theme_value_source::ThemeValueSource,
};

// Used by `cosmic-text` for calculating text layout, and `base64` for encoding
// the font data.
const NOTO_SANS_MONO_TTF: &[u8] =
    include_bytes!("../fonts/noto_sans_mono/NotoSansMono-Regular.ttf");

mod absolute_coordinates;
mod diagram_generate_error;
mod diagram_generator;
mod edge_face_assigner;
mod edge_id_generator;
mod input_diagram_merger;
mod input_diagram_theme_sources;
mod input_to_ir_diagram_mapper;
mod ir_to_taffy_builder;
mod md_text;
mod node_ranks_calculator;
mod process_step_graph_calculator;
mod string_xml_escaper;
mod svg_elements_to_svg_mapper;
mod taffy_node_absolute_coordinates_calculator;
mod taffy_to_svg_elements_mapper;
mod theme_value_source;
