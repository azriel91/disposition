use disposition::{
    ir_model::IrDiagram,
    model_common::{id, Id},
    taffy_model::{taffy::TaffyError, DimensionAndLod},
};
use disposition_input_ir_rt::{IrToTaffyBuilder, TaffyToSvgElementsMapper};

use crate::input_ir_rt::EXAMPLE_IR;

/// Helper: build `SvgElements` from the example IR fixture.
fn build_svg_elements_from_example_ir(
) -> impl Iterator<Item = disposition::svg_model::SvgElements<'static>> {
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
        .map(move |taffy_node_mappings| {
            TaffyToSvgElementsMapper::map(&ir_example, &taffy_node_mappings)
        })
        .collect::<Vec<_>>()
        .into_iter()
}

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
fn test_svg_edge_infos_arrow_head_path_d_non_empty() -> Result<(), TaffyError> {
    for svg_elements in build_svg_elements_from_example_ir() {
        for edge_info in &svg_elements.svg_edge_infos {
            assert!(
                !edge_info.arrow_head_path_d.is_empty(),
                "arrow_head_path_d should not be empty for edge {:?}",
                edge_info.edge_id
            );
        }
    }
    Ok(())
}

#[test]
fn test_svg_edge_infos_dependency_arrow_head_is_positioned() -> Result<(), TaffyError> {
    for svg_elements in build_svg_elements_from_example_ir() {
        // Dependency edges have IDs starting with "edge_dep_".
        let dep_edges: Vec<_> = svg_elements
            .svg_edge_infos
            .iter()
            .filter(|e| e.edge_id.as_str().starts_with("edge_dep_"))
            .collect();

        assert!(
            !dep_edges.is_empty(),
            "Expected at least one dependency edge in EXAMPLE_IR"
        );

        for edge_info in &dep_edges {
            let d = &edge_info.arrow_head_path_d;
            // A positioned arrowhead is a closed V-shape: it should start
            // with 'M' (moveto) and contain 'Z' (closepath).
            assert!(
                d.starts_with('M'),
                "Dependency arrowhead should start with 'M', got: {d}"
            );
            assert!(
                d.contains('Z') || d.contains('z'),
                "Dependency arrowhead should be a closed path (contain 'Z'), got: {d}"
            );
            // It should contain line-to commands (L or l) for the V wings.
            assert!(
                d.contains('L') || d.contains('l'),
                "Dependency arrowhead should contain line-to commands, got: {d}"
            );

            // The arrowhead path coordinates should NOT be at the origin.
            // Parse the first M command to verify it is positioned in the
            // SVG canvas (not at 0,0).
            // We just check that the path is non-trivially positioned by
            // verifying it is not exactly the origin-centred template.
            assert!(
                d != "M-8,-4L0,0L-8,4Z",
                "Dependency arrowhead should be positioned, not origin-centred"
            );
        }
    }
    Ok(())
}

#[test]
fn test_svg_edge_infos_interaction_arrow_head_is_origin_centred() -> Result<(), TaffyError> {
    for svg_elements in build_svg_elements_from_example_ir() {
        // Interaction edges have IDs starting with "edge_ix_".
        let ix_edges: Vec<_> = svg_elements
            .svg_edge_infos
            .iter()
            .filter(|e| e.edge_id.as_str().starts_with("edge_ix_"))
            .collect();

        assert!(
            !ix_edges.is_empty(),
            "Expected at least one interaction edge in EXAMPLE_IR"
        );

        // All interaction edges should share the same origin-centred
        // arrowhead path.
        let first_d = &ix_edges[0].arrow_head_path_d;
        for edge_info in &ix_edges {
            assert_eq!(
                &edge_info.arrow_head_path_d, first_d,
                "All interaction arrowheads should use the same origin-centred path"
            );
        }

        // The origin-centred V-shape should be a closed path.
        assert!(
            first_d.starts_with('M'),
            "Interaction arrowhead should start with 'M', got: {first_d}"
        );
        assert!(
            first_d.contains('Z') || first_d.contains('z'),
            "Interaction arrowhead should be closed (contain 'Z'), got: {first_d}"
        );
    }
    Ok(())
}

#[test]
fn test_svg_edge_infos_interaction_arrow_head_tailwind_classes() -> Result<(), TaffyError> {
    for svg_elements in build_svg_elements_from_example_ir() {
        let ix_edges: Vec<_> = svg_elements
            .svg_edge_infos
            .iter()
            .filter(|e| e.edge_id.as_str().starts_with("edge_ix_"))
            .collect();

        for edge_info in &ix_edges {
            // The arrowhead entity ID is `{edge_id}__arrow_head` (with
            // underscores, since `Id` only allows [a-zA-Z0-9_]).
            let arrow_head_key_str = format!("{}_arrow_head", edge_info.edge_id.as_str());
            let arrow_head_key =
                Id::try_from(arrow_head_key_str.clone()).expect("arrow head ID should be valid");

            let classes = svg_elements
                .tailwind_classes
                .get(&arrow_head_key)
                .unwrap_or_else(|| {
                    panic!("Expected tailwind classes for arrowhead entity '{arrow_head_key_str}'")
                });

            // Should contain offset-path with a path(...) value.
            assert!(
                classes.contains("[offset-path:path('"),
                "Arrowhead classes should contain `[offset-path:path('`, got: {classes}"
            );

            // Should contain offset-rotate auto.
            assert!(
                classes.contains("[offset-rotate]:[auto]"),
                "Arrowhead classes should contain [offset-rotate]:[auto], got: {classes}"
            );

            // Should contain an animate-[...] class.
            assert!(
                classes.contains("animate-["),
                "Arrowhead classes should contain an animate-[] class, got: {classes}"
            );

            // The animation name should contain "--arrow-head-offset".
            assert!(
                classes.contains("--arrow-head-offset"),
                "Arrowhead animation name should contain '--arrow-head-offset', got: {classes}"
            );
        }
    }
    Ok(())
}

#[test]
fn test_svg_edge_infos_interaction_arrow_head_css_keyframes() -> Result<(), TaffyError> {
    for svg_elements in build_svg_elements_from_example_ir() {
        let ix_edges: Vec<_> = svg_elements
            .svg_edge_infos
            .iter()
            .filter(|e| e.edge_id.as_str().starts_with("edge_ix_"))
            .collect();

        for edge_info in &ix_edges {
            let edge_id_with_hyphens = edge_info.edge_id.as_str().replace('_', "-");
            let expected_animation_name = format!("{edge_id_with_hyphens}--arrow-head-offset");

            // The CSS should contain an @keyframes rule for this arrowhead.
            assert!(
                svg_elements.css.contains(&expected_animation_name),
                "CSS should contain @keyframes for '{expected_animation_name}'"
            );

            // The keyframes should reference offset-distance and opacity.
            // Find the keyframes block for this animation.
            let keyframes_prefix = format!("@keyframes {expected_animation_name}");
            assert!(
                svg_elements.css.contains(&keyframes_prefix),
                "CSS should contain '{keyframes_prefix}'"
            );

            // Check that the keyframes contain the expected properties.
            let css = &svg_elements.css;
            let start_idx = css
                .find(&keyframes_prefix)
                .expect("keyframes prefix must exist");
            let block = &css[start_idx..];
            let end_idx = block.find('}').expect("keyframes must have closing brace");
            let keyframes_block = &block[..=end_idx];

            assert!(
                keyframes_block.contains("opacity:"),
                "Arrow head keyframes should contain opacity, got: {keyframes_block}"
            );
            assert!(
                keyframes_block.contains("offset-distance:"),
                "Arrow head keyframes should contain offset-distance, got: {keyframes_block}"
            );
        }
    }
    Ok(())
}

#[test]
fn test_svg_edge_infos_dependency_no_arrow_head_animation_classes() -> Result<(), TaffyError> {
    for svg_elements in build_svg_elements_from_example_ir() {
        let dep_edges: Vec<_> = svg_elements
            .svg_edge_infos
            .iter()
            .filter(|e| e.edge_id.as_str().starts_with("edge_dep_"))
            .collect();

        for edge_info in &dep_edges {
            // Dependency edges should NOT have arrowhead animation tailwind
            // classes – there should be no entity key for them.
            let arrow_head_key_str = format!("{}_arrow_head", edge_info.edge_id.as_str());
            if let Ok(arrow_head_key) = Id::try_from(arrow_head_key_str) {
                assert!(
                    svg_elements.tailwind_classes.get(&arrow_head_key).is_none(),
                    "Dependency edge {:?} should NOT have arrowhead animation tailwind classes",
                    edge_info.edge_id
                );
            }
        }
    }
    Ok(())
}

#[test]
fn test_svg_edge_infos_self_loop_arrow_head() -> Result<(), TaffyError> {
    for svg_elements in build_svg_elements_from_example_ir() {
        // Self-loop edges (from == to) should still have arrowheads.
        let self_loops: Vec<_> = svg_elements
            .svg_edge_infos
            .iter()
            .filter(|edge| edge.from_node_id == edge.to_node_id)
            .collect();

        assert!(
            !self_loops.is_empty(),
            "Expected at least one self-loop edge in EXAMPLE_IR"
        );

        for edge in &self_loops {
            assert!(
                !edge.arrow_head_path_d.is_empty(),
                "Self-loop edge {:?} should have a non-empty arrow_head_path_d",
                edge.edge_id
            );
            assert!(
                edge.arrow_head_path_d.contains('Z') || edge.arrow_head_path_d.contains('z'),
                "Self-loop arrowhead should be a closed path for edge {:?}",
                edge.edge_id
            );
        }
    }
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
