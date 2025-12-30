use disposition::{
    ir_model::IrDiagram,
    taffy_model::{taffy::TaffyError, TaffyNodeMappings},
};
use disposition_input_ir_rt::IrToTaffyBuilder;

use crate::input_ir_rt::EXAMPLE_IR;

#[test]
fn test_example_ir_mapping_to_taffy_tree_and_root() -> Result<(), TaffyError> {
    let ir_example = serde_saphyr::from_str::<IrDiagram>(EXAMPLE_IR).unwrap();
    let ir_to_taffy_builder = IrToTaffyBuilder::builder()
        .with_ir_diagram(ir_example)
        .build();
    let mut taffy_tree_and_root_iter = ir_to_taffy_builder
        .build()
        .expect("Expected `taffy_tree_and_root` to be built.");

    let Some(taffy_tree_and_diagram_sm) = taffy_tree_and_root_iter.next() else {
        panic!("Expected small `taffy_tree_and_root` to exist.");
    };
    assert_taffy_measurements(
        taffy_tree_and_diagram_sm,
        MeasurementsExpected {
            diagram_width: 640.0,
            diagram_height: 480.0,
        },
    )?;

    let Some(taffy_tree_and_diagram_md) = taffy_tree_and_root_iter.next() else {
        panic!("Expected medium `taffy_tree_and_root` to exist.");
    };
    assert_taffy_measurements(
        taffy_tree_and_diagram_md,
        MeasurementsExpected {
            diagram_width: 768.0,
            diagram_height: 512.0,
        },
    )?;

    let Some(taffy_tree_and_diagram_lg) = taffy_tree_and_root_iter.next() else {
        panic!("Expected large `taffy_tree_and_root` to exist.");
    };
    assert_taffy_measurements(
        taffy_tree_and_diagram_lg,
        MeasurementsExpected {
            diagram_width: 1024.0,
            diagram_height: 722.0,
        },
    )?;

    Ok(())
}

fn assert_taffy_measurements(
    taffy_node_mappings: TaffyNodeMappings,
    measurements_expected: MeasurementsExpected,
) -> Result<(), TaffyError> {
    let MeasurementsExpected {
        diagram_width,
        diagram_height,
    } = measurements_expected;

    let TaffyNodeMappings { taffy_tree, root } = taffy_node_mappings;
    let root_layout = taffy_tree.layout(root)?;
    let distance_tolerance = 15.0f32;
    assert!(
        root_layout.size.width > diagram_width - distance_tolerance
            && root_layout.size.width <= diagram_width
    );

    assert!(
        root_layout.size.height > diagram_height - distance_tolerance
            && root_layout.size.height <= diagram_height
    );
    Ok(())
}

struct MeasurementsExpected {
    diagram_width: f32,
    diagram_height: f32,
}
