use disposition::{
    ir_model::IrDiagram,
    taffy_model::{taffy::TaffyError, DimensionAndLod},
};
use disposition_input_ir_rt::{IrToTaffyBuilder, TaffyToSvgElementsMapper};

use crate::input_ir_rt::EXAMPLE_IR;

#[test]
fn test_example_ir_mapping_to_svg_elements() -> Result<(), TaffyError> {
    let ir_example = serde_saphyr::from_str::<IrDiagram>(EXAMPLE_IR).unwrap();
    let ir_to_taffy_builder = IrToTaffyBuilder::builder()
        .with_ir_diagram(&ir_example)
        .with_dimension_and_lods(vec![DimensionAndLod::default_2xl()])
        .build();
    ir_to_taffy_builder
        .build()
        .expect("Expected `taffy_node_mappings` to be built.")
        .map(|taffy_node_mappings| TaffyToSvgElementsMapper::map(&ir_example, &taffy_node_mappings))
        .for_each(|svg_elements| {
            // Verify SVG dimensions are set
            assert!(svg_elements.svg_width > 0.0);
            assert!(svg_elements.svg_height > 0.0);

            // Verify we have node infos
            assert!(
                !svg_elements.svg_node_infos.is_empty(),
                "Expected svg_node_infos to be non-empty"
            );

            // Verify each node has valid coordinates and dimensions
            for node_info in &svg_elements.svg_node_infos {
                assert!(node_info.width > 0.0, "Node width should be positive");
                assert!(
                    node_info.height_collapsed > 0.0,
                    "Node height should be positive"
                );
                assert!(
                    !node_info.path_d_collapsed.is_empty(),
                    "Path d attribute should be non-empty"
                );
            }

            // Verify additional tailwind classes were generated
            assert!(
                !svg_elements.additional_tailwind_classes.is_empty(),
                "Expected additional_tailwind_classes to be non-empty"
            );

            eprintln!(
                "\n------------------------\nSvgElements:\n  svg_width: {}\n  svg_height: {}\n  node_count: {}\n  tailwind_class_count: {}\n-----------------------\n",
                svg_elements.svg_width,
                svg_elements.svg_height,
                svg_elements.svg_node_infos.len(),
                svg_elements.additional_tailwind_classes.len()
            );
        });

    Ok(())
}

#[test]
fn test_svg_elements_node_info_structure() -> Result<(), TaffyError> {
    let ir_example = serde_saphyr::from_str::<IrDiagram>(EXAMPLE_IR).unwrap();
    let ir_to_taffy_builder = IrToTaffyBuilder::builder()
        .with_ir_diagram(&ir_example)
        .with_dimension_and_lods(vec![DimensionAndLod::default_2xl()])
        .build();
    ir_to_taffy_builder
        .build()
        .expect("Expected `taffy_node_mappings` to be built.")
        .map(|taffy_node_mappings| TaffyToSvgElementsMapper::map(&ir_example, &taffy_node_mappings))
        .for_each(|svg_elements| {
            // Check that all nodes from ir_diagram.node_ordering are present
            let svg_node_ids: Vec<_> = svg_elements
                .svg_node_infos
                .iter()
                .map(|info| &info.node_id)
                .collect();

            for (node_id, _) in ir_example.node_ordering.iter() {
                assert!(
                    svg_node_ids.contains(&node_id),
                    "Node {node_id} should be in svg_node_infos"
                );
            }

            // Verify tab indices are preserved
            for svg_node_info in &svg_elements.svg_node_infos {
                if let Some(&expected_tab_index) =
                    ir_example.node_ordering.get(&svg_node_info.node_id)
                {
                    assert_eq!(
                        svg_node_info.tab_index, expected_tab_index,
                        "Tab index should match for node {}",
                        svg_node_info.node_id
                    );
                }
            }
        });

    Ok(())
}
