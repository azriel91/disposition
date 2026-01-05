use disposition::{
    ir_model::{node::NodeInbuilt, IrDiagram},
    taffy_model::{taffy::TaffyError, TaffyNodeMappings},
};
use disposition_input_ir_rt::IrToTaffyBuilder;

use crate::input_ir_rt::EXAMPLE_IR;

#[test]
fn test_example_ir_mapping_to_taffy_tree_and_root() -> Result<(), TaffyError> {
    let ir_example = serde_saphyr::from_str::<IrDiagram>(EXAMPLE_IR).unwrap();
    let ir_to_taffy_builder = IrToTaffyBuilder::builder()
        .with_ir_diagram(&ir_example)
        .build();
    let mut taffy_tree_and_root_iter = ir_to_taffy_builder
        .build()
        .expect("Expected `taffy_tree_and_root` to be built.");

    let Some(taffy_node_mappings_sm) = taffy_tree_and_root_iter.next() else {
        panic!("Expected small `taffy_tree_and_root` to exist.");
    };
    assert_taffy_measurements(
        taffy_node_mappings_sm,
        MeasurementsExpected {
            diagram_width: 640.0,
            diagram_height: 480.0,
        },
    )?;

    let Some(taffy_node_mappings_md) = taffy_tree_and_root_iter.next() else {
        panic!("Expected medium `taffy_tree_and_root` to exist.");
    };
    assert_taffy_measurements(
        taffy_node_mappings_md,
        MeasurementsExpected {
            diagram_width: 768.0,
            diagram_height: 512.0,
        },
    )?;

    let Some(taffy_node_mappings_lg) = taffy_tree_and_root_iter.next() else {
        panic!("Expected large `taffy_tree_and_root` to exist.");
    };
    assert_taffy_measurements(
        taffy_node_mappings_lg,
        MeasurementsExpected {
            diagram_width: 1024.0,
            diagram_height: 768.0,
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

    let TaffyNodeMappings {
        taffy_tree,
        // TODO: assert:
        //
        // * `_processes_container` and `_things_container` have the same `y` coordinate.
        // * `_processes_container` has a greater x coordinate than `_things_container`.
        node_inbuilt_to_taffy,
        // TODO: make assertions for node positioning
        node_id_to_taffy: _,
        entity_highlighted_spans: _,
    } = taffy_node_mappings;
    let root_layout = node_inbuilt_to_taffy
        .get(&NodeInbuilt::Root)
        .copied()
        .map(|root| taffy_tree.layout(root))
        .transpose()?
        .expect("Failed to get `taffy` root node layout");
    let distance_tolerance = 15.0f32;

    let root_width = root_layout.size.width;
    let root_width_expected_min = diagram_width - distance_tolerance;
    let root_width_expected_max = diagram_width;
    assert!(
        root_width > root_width_expected_min
            && root_width <= root_width_expected_max,
        "Expected root container width `{root_width}` to be between {root_width_expected_min} and {root_width_expected_max}"
    );

    let root_height = root_layout.size.height;
    let root_height_expected_min = diagram_height - distance_tolerance;
    let root_height_expected_max = diagram_height;
    assert!(
        root_height > root_height_expected_min
            && root_height <= root_height_expected_max,
        "Expected root container height `{root_height}` to be between {root_height_expected_min} and {root_height_expected_max}"
    );

    Ok(())
}

struct MeasurementsExpected {
    diagram_width: f32,
    diagram_height: f32,
}
