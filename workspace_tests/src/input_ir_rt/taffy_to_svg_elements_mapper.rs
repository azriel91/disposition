use disposition::{
    ir_model::IrDiagram,
    model_common::{id, Id},
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

            // Verify process nodes have tailwind classes for translation appended to their entity tailwind classes.
            let Some(proc_app_dev_tailwind_classes) = svg_elements.tailwind_classes.get(&id!("proc_app_dev")) else {
                panic!("Expected tailwind classes for process node 'proc_app_dev'");
            };
            assert!(proc_app_dev_tailwind_classes.contains("translate-x-"), "Expected process node to have 'translate-x-' tailwind class");
            assert!(proc_app_dev_tailwind_classes.contains("[&>path]:[d:path('"), "Expected process node to have '[&>path]:[d:path('] tailwind class");

            eprintln!(
                "\n------------------------\nSvgElements:\n  svg_width: {}\n  svg_height: {}\n  node_count: {}\n  process_info_count: {}\n-----------------------\n",
                svg_elements.svg_width,
                svg_elements.svg_height,
                svg_elements.svg_node_infos.len(),
                svg_elements.svg_process_infos.len(),
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

#[test]
fn test_svg_edge_infos_are_generated() -> Result<(), TaffyError> {
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
            // Verify we have edge infos generated
            assert!(
                !svg_elements.svg_edge_infos.is_empty(),
                "Expected svg_edge_infos to be non-empty since EXAMPLE_IR has edge_groups"
            );

            // Verify each edge has valid data
            for edge_info in &svg_elements.svg_edge_infos {
                assert!(
                    !edge_info.path_d.is_empty(),
                    "Edge path_d should not be empty for edge {:?}",
                    edge_info.edge_id
                );
                // Path should start with M (moveto command)
                assert!(
                    edge_info.path_d.starts_with('M'),
                    "Edge path_d should start with 'M' (moveto), got: {}",
                    edge_info.path_d
                );
                // Path should contain C (curveto command) for Bezier curves
                assert!(
                    edge_info.path_d.contains('C'),
                    "Edge path_d should contain 'C' (curveto) for curves, got: {}",
                    edge_info.path_d
                );
            }

            eprintln!(
                "\n------------------------\nEdge Infos:\n  edge_count: {}\n-----------------------\n",
                svg_elements.svg_edge_infos.len()
            );
        });

    Ok(())
}

#[test]
fn test_svg_edge_infos_self_loop() -> Result<(), TaffyError> {
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
            // Find self-loop edges (from == to)
            let self_loops: Vec<_> = svg_elements
                .svg_edge_infos
                .iter()
                .filter(|edge| edge.from_node_id == edge.to_node_id)
                .collect();

            // EXAMPLE_IR has self-loop edges like t_localhost -> t_localhost
            assert!(
                !self_loops.is_empty(),
                "Expected at least one self-loop edge in EXAMPLE_IR"
            );

            for edge in &self_loops {
                // Self-loop paths should have multiple curve commands for the loop shape
                let curve_count = edge.path_d.matches('C').count();
                assert!(
                    curve_count >= 2,
                    "Self-loop edge should have at least 2 curve commands, got {} for edge {:?}",
                    curve_count,
                    edge.edge_id
                );
            }
        });

    Ok(())
}

#[test]
fn test_svg_edge_infos_bidirectional() -> Result<(), TaffyError> {
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
            // Look for bidirectional edges (A->B and B->A in same edge group)
            // EXAMPLE_IR has t_localhost <-> t_github_user_repo bidirectional edges

            // Find edges where we have both directions
            let localhost_to_github: Vec<_> = svg_elements
                .svg_edge_infos
                .iter()
                .filter(|e| {
                    e.from_node_id.as_str() == "t_localhost"
                        && e.to_node_id.as_str() == "t_github_user_repo"
                })
                .collect();

            let github_to_localhost: Vec<_> = svg_elements
                .svg_edge_infos
                .iter()
                .filter(|e| {
                    e.from_node_id.as_str() == "t_github_user_repo"
                        && e.to_node_id.as_str() == "t_localhost"
                })
                .collect();

            // Both directions should exist
            assert!(
                !localhost_to_github.is_empty(),
                "Expected edges from t_localhost to t_github_user_repo"
            );
            assert!(
                !github_to_localhost.is_empty(),
                "Expected edges from t_github_user_repo to t_localhost"
            );

            // The paths should be different (offset for bidirectional)
            if !localhost_to_github.is_empty() && !github_to_localhost.is_empty() {
                assert_ne!(
                    localhost_to_github[0].path_d, github_to_localhost[0].path_d,
                    "Bidirectional edges should have different paths to avoid overlap"
                );
            }
        });

    Ok(())
}

#[test]
fn test_svg_edge_infos_edge_group_id_preserved() -> Result<(), TaffyError> {
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
            // Verify that edge_group_id is properly set for all edges
            for edge_info in &svg_elements.svg_edge_infos {
                // edge_group_id should not be empty
                assert!(
                    !edge_info.edge_group_id.as_str().is_empty(),
                    "edge_group_id should not be empty"
                );

                // edge_id should contain the edge_group_id as a prefix
                assert!(
                    edge_info
                        .edge_id
                        .as_str()
                        .starts_with(edge_info.edge_group_id.as_str()),
                    "edge_id '{}' should start with edge_group_id '{}'",
                    edge_info.edge_id.as_str(),
                    edge_info.edge_group_id.as_str()
                );
            }
        });

    Ok(())
}

#[test]
fn test_process_infos_map_structure() -> Result<(), TaffyError> {
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
            // Verify process_infos is keyed by process node ID
            for (process_id, process_info) in &svg_elements.svg_process_infos {
                // The key should match the process_id in the value
                assert_eq!(
                    process_id, &process_info.process_id,
                    "Map key should match process_info.process_id"
                );

                // Process info should have valid data
                assert!(
                    process_info.height_to_expand_to > 0.0,
                    "height_to_expand_to should be positive"
                );
                assert!(
                    !process_info.path_d_expanded.is_empty(),
                    "path_d_expanded should be non-empty"
                );
                assert!(
                    process_info.total_height >= 0.0,
                    "total_height should be non-negative"
                );
            }

            // Verify that nodes with process_id can look up their process info
            for svg_node_info in &svg_elements.svg_node_infos {
                if let Some(ref proc_id) = svg_node_info.process_id {
                    assert!(
                        svg_elements.svg_process_infos.contains_key(proc_id),
                        "process_id {:?} in node {} should exist in process_infos map",
                        proc_id,
                        svg_node_info.node_id
                    );
                }
            }
        });

    Ok(())
}
