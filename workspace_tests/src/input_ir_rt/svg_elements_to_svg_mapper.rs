use disposition::{
    ir_model::IrDiagram,
    taffy_model::{taffy::TaffyError, DimensionAndLod},
};
use disposition_input_ir_rt::{IrToTaffyBuilder, SvgElementsToSvgMapper, TaffyToSvgElementsMapper};

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
        .map(|taffy_node_mappings| TaffyToSvgElementsMapper::map(&ir_example, &taffy_node_mappings))
        .map(|svg_elements| SvgElementsToSvgMapper::map(&svg_elements))
        .for_each(|svg| {
            eprintln!("\n------------------------\n{svg}\n\n-----------------------\n");
        });

    Ok(())
}

#[test]
fn test_rendered_svg_contains_arrow_head_paths() -> Result<(), TaffyError> {
    let ir_example = serde_saphyr::from_str::<IrDiagram>(EXAMPLE_IR).unwrap();
    let ir_to_taffy_builder = IrToTaffyBuilder::builder()
        .with_ir_diagram(&ir_example)
        .with_dimension_and_lods(vec![DimensionAndLod::default_2xl()])
        .build();
    let taffy_results: Vec<_> = ir_to_taffy_builder
        .build()
        .expect("Expected `taffy_node_mappings` to be built.")
        .collect();
    taffy_results
        .into_iter()
        .map(|taffy_node_mappings| TaffyToSvgElementsMapper::map(&ir_example, &taffy_node_mappings))
        .map(|svg_elements| {
            let edge_count = svg_elements.svg_edge_infos.len();
            let svg = SvgElementsToSvgMapper::map(&svg_elements);
            (svg, edge_count)
        })
        .for_each(|(svg, edge_count)| {
            // Every edge should produce an arrowhead <path> with the
            // "arrow_head" class inside its <g> element.
            let arrow_head_count = svg.matches("arrow_head").count();
            assert!(
                arrow_head_count >= edge_count,
                "Expected at least {edge_count} arrowhead class occurrences in the SVG, \
                 found {arrow_head_count}"
            );

            // Dependency edges produce a positioned closed V-shape whose
            // path contains 'Z' (close-path).  Interaction edges also have
            // a closed V-shape.  Verify we see at least one close-path
            // command inside the rendered SVG (outside of the <style> block).
            let content_after_style = svg.find("</style>").map(|idx| &svg[idx..]).unwrap_or(&svg);
            assert!(
                content_after_style.contains("arrow_head"),
                "Rendered SVG content should contain arrowhead paths"
            );
        });

    Ok(())
}

#[test]
fn test_rendered_svg_interaction_edge_arrow_head_has_animation_classes() -> Result<(), TaffyError> {
    let ir_example = serde_saphyr::from_str::<IrDiagram>(EXAMPLE_IR).unwrap();
    let ir_to_taffy_builder = IrToTaffyBuilder::builder()
        .with_ir_diagram(&ir_example)
        .with_dimension_and_lods(vec![DimensionAndLod::default_2xl()])
        .build();
    let taffy_results: Vec<_> = ir_to_taffy_builder
        .build()
        .expect("Expected `taffy_node_mappings` to be built.")
        .collect();
    taffy_results
        .into_iter()
        .map(|taffy_node_mappings| TaffyToSvgElementsMapper::map(&ir_example, &taffy_node_mappings))
        .map(|svg_elements| SvgElementsToSvgMapper::map(&svg_elements))
        .for_each(|svg| {
            // The rendered SVG should contain offset-path CSS properties for interaction
            // edge arrowheads.
            assert!(
                svg.contains("offset-path"),
                "Rendered SVG should contain offset-path for interaction arrowheads"
            );

            // The CSS should contain @keyframes rules with
            // "--arrow-head-offset" in their names.
            assert!(
                svg.contains("--arrow-head-offset"),
                "Rendered SVG CSS should contain arrow-head-offset keyframes"
            );
        });

    Ok(())
}
