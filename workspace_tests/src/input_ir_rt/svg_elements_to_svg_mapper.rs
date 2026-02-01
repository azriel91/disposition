use disposition::{
    ir_model::IrDiagram,
    taffy_model::{taffy::TaffyError, DimensionAndLod},
};
use disposition_input_ir_rt::{IrToTaffyBuilder, SvgElementsToSvgMapper};

use crate::input_ir_rt::EXAMPLE_IR;

#[test]
fn test_example_ir_mapping_to_taffy_node_mappings() -> Result<(), TaffyError> {
    let ir_example = serde_saphyr::from_str::<IrDiagram>(EXAMPLE_IR).unwrap();
    let ir_to_taffy_builder = IrToTaffyBuilder::builder()
        .with_ir_diagram(&ir_example)
        .with_dimension_and_lods(vec![DimensionAndLod::default_2xl()])
        .build();
    ir_to_taffy_builder
        .build()
        .expect("Expected `taffy_node_mappings` to be built.")
        .map(|taffy_node_mappings| SvgElementsToSvgMapper::map(&ir_example, taffy_node_mappings))
        .for_each(|svg| {
            eprintln!("\n------------------------\n{svg}\n\n-----------------------\n");
        });

    Ok(())
}
