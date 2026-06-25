use disposition::{
    input_ir_model::IrDiagramAndIssues,
    input_model::InputDiagram,
    ir_model::IrDiagram,
    model_common::{
        edge::{ARC_RADIUS, MAX_GAP_FRACTION, MIN_PROTRUSION_PX, TO_PROTRUSION_MIN_PX},
        id, Id, ProcessRenderCollapse,
    },
    svg_model::SvgElements,
    taffy_model::{taffy::TaffyError, DimensionAndLod, TEXT_LINE_HEIGHT},
};
use disposition_input_ir_rt::{
    EdgeAnimationActive, InputDiagramMerger, InputToIrDiagramMapper, IrToTaffyBuilder,
    SvgElementsToSvgMapper, TaffyToSvgElementsMapper,
};

use crate::input_ir_rt::{
    EXAMPLE_IR, INPUT_DIAGRAM_0001_NESTED_NODE_EDGE_PROTRUSION,
    INPUT_DIAGRAM_0002_NESTED_NODE_EDGE_PROTRUSION, INPUT_DIAGRAM_0003_EDGES_SYMMETRIC_2_NODES,
    INPUT_DIAGRAM_0004_EDGES_SYMMETRIC_3_NODES, INPUT_DIAGRAM_0005_TAG_NODES_CYCLIC_EDGE,
    INPUT_DIAGRAM_0006_PROCESS_STEP_NODES_CYCLIC_EDGE,
    INPUT_DIAGRAM_0007_EDGE_FROM_NODE_TO_NESTED_NODE,
    INPUT_DIAGRAM_0008_EDGE_FROM_NODE_TO_NESTED_RANK_1_NODE,
    INPUT_DIAGRAM_0009_EDGE_WITH_DESCRIPTION, INPUT_DIAGRAM_0010_SELF_LOOP_EDGE_WITH_DESCRIPTION,
    INPUT_DIAGRAM_0011_CONTAINED_EDGE_WITH_DESCRIPTION,
    INPUT_DIAGRAM_0012_EDGE_FROM_NESTED_NODE_TO_OUTER_NODE_CYCLIC,
    INPUT_DIAGRAM_0013_EDGE_FROM_NESTED_NODE_TO_OUTER_NODE_CYCLIC_2,
    INPUT_DIAGRAM_0017_EDGE_INNER_TO_INNER, INPUT_DIAGRAM_0018_PROCESS_STEP_BRANCH_MERGE,
    INPUT_DIAGRAM_0019_RANK_DIR_REVERSED_SIBLINGS,
    INPUT_DIAGRAM_0020_SELF_LOOP_CYCLIC_TWO_NODE_LEFT_TO_RIGHT,
    INPUT_DIAGRAM_0021_SELF_LOOP_EDGE_LEFT_TO_RIGHT_WITH_EDGE_DESC,
    INPUT_DIAGRAM_0022_EDGES_FAN_IN_3_TO_1, INPUT_DIAGRAM_0023_NESTED_EDGES_RANK_DIR_TOP_TO_BOTTOM,
    INPUT_DIAGRAM_0024_NESTED_EDGES_RANK_DIR_LEFT_TO_RIGHT,
    INPUT_DIAGRAM_0025_NESTED_EDGES_RANK_DIR_RIGHT_TO_LEFT,
    INPUT_DIAGRAM_0026_NESTED_EDGES_RANK_DIR_BOTTOM_TO_TOP,
    INPUT_DIAGRAM_0027_NESTED_NODE_EDGE_PROTRUSION_TO_NESTED_NODE_1,
    INPUT_DIAGRAM_0028_NESTED_NODE_EDGE_PROTRUSION_TO_NESTED_NODE_2,
    INPUT_DIAGRAM_0029_NESTED_EDGE_OVERLAP_WITH_DIFFERENT_RANK_NESTED_EDGE,
    INPUT_DIAGRAM_0030_NESTED_EDGE_OVERLAP_WITH_DIFFERENT_RANK_NESTED_EDGE_WITH_NODE_DESC,
    INPUT_DIAGRAM_0031_NESTED_NODE_HIGH_RANK_EDGE_TO_NEXT_NODE_TOP_TO_BOTTOM,
    INPUT_DIAGRAM_0032_NESTED_NODE_HIGH_RANK_EDGE_TO_NEXT_NODE_LEFT_TO_RIGHT,
    INPUT_DIAGRAM_0033_NESTED_NODE_HIGH_RANK_EDGE_TO_NEXT_NODE_RIGHT_TO_LEFT,
    INPUT_DIAGRAM_0034_NESTED_NODE_HIGH_RANK_EDGE_TO_NEXT_NODE_BOTTOM_TO_TOP,
    INPUT_DIAGRAM_0035_NESTED_NODE_MID_RANK_EDGE_TO_NEXT_NODE_TOP_TO_BOTTOM,
    INPUT_DIAGRAM_0036_NESTED_NODE_MID_RANK_EDGE_TO_NEXT_HIGH_RANK_NODE_TOP_TO_BOTTOM,
    INPUT_DIAGRAM_0037_NESTED_NODE_MID_RANK_EDGE_TO_NEXT_HIGH_RANK_NODE_LEFT_TO_RIGHT,
    INPUT_DIAGRAM_0038_NESTED_NODE_MID_RANK_EDGE_TO_NEXT_HIGH_RANK_NODE_RIGHT_TO_LEFT,
    INPUT_DIAGRAM_0039_NESTED_NODE_MID_RANK_EDGE_TO_NEXT_HIGH_RANK_NODE_BOTTOM_TO_TOP,
    INPUT_DIAGRAM_0040_MD_CODE_BLOCK, INPUT_DIAGRAM_0041_MD_CODE_BLOCK_IN_LIST,
    INPUT_DIAGRAM_0042_MD_BLOCKQUOTE, INPUT_DIAGRAM_0043_EDGE_OFFSETS_AND_PROTRUSION_COMPLEX_1,
};

/// Helper: build `SvgElements` from the example IR fixture.
fn build_svg_elements_from_example_ir() -> impl Iterator<Item = SvgElements<'static>> {
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
            TaffyToSvgElementsMapper::map(
                &ir_example,
                &taffy_node_mappings,
                EdgeAnimationActive::Always,
            )
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
        .map(|taffy_node_mappings| {
            TaffyToSvgElementsMapper::map(
                &ir_example,
                &taffy_node_mappings,
                EdgeAnimationActive::Always,
            )
        })
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
            assert!(proc_app_dev_tailwind_classes.contains("[&>path.wrapper]:[d:path('"), "Expected process node to have '[&>path.wrapper]:[d:path('] tailwind class");

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
        .map(|taffy_node_mappings| {
            TaffyToSvgElementsMapper::map(
                &ir_example,
                &taffy_node_mappings,
                EdgeAnimationActive::Always,
            )
        })
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
        .map(|taffy_node_mappings| {
            TaffyToSvgElementsMapper::map(
                &ir_example,
                &taffy_node_mappings,
                EdgeAnimationActive::Always,
            )
        })
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
                // Path should contain C (curveto command) for Bezier curves or L (lineto command) for straight lines
                assert!(
                    edge_info.path_d.contains('C') || edge_info.path_d.contains('L'),
                    "Edge path_d should contain 'C' (curveto) for curves or 'L' (lineto) for straight lines, got: {}",
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
        .map(|taffy_node_mappings| {
            TaffyToSvgElementsMapper::map(
                &ir_example,
                &taffy_node_mappings,
                EdgeAnimationActive::Always,
            )
        })
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

                // Self-loops participate in the cycle-edge protrusion
                // assignment: both contacts protrude by the same depth.
                let from_protrusion = edge.ortho_protrusion_params.from_protrusion;
                let to_protrusion = edge.ortho_protrusion_params.to_protrusion;
                assert!(
                    from_protrusion > 0.0,
                    "Self-loop edge {:?} should have a positive from_protrusion, got {from_protrusion}",
                    edge.edge_id
                );
                assert_eq!(
                    from_protrusion, to_protrusion,
                    "Self-loop edge {:?} should have equal from/to protrusions",
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
        .map(|taffy_node_mappings| {
            TaffyToSvgElementsMapper::map(
                &ir_example,
                &taffy_node_mappings,
                EdgeAnimationActive::Always,
            )
        })
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
        .map(|taffy_node_mappings| {
            TaffyToSvgElementsMapper::map(
                &ir_example,
                &taffy_node_mappings,
                EdgeAnimationActive::Always,
            )
        })
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
        let svg_edge_infos_ix: Vec<_> = svg_elements
            .svg_edge_infos
            .iter()
            .filter(|svg_edge_info| svg_edge_info.edge_id.as_str().starts_with("edge_ix_"))
            .collect();

        for svg_edge_info in &svg_edge_infos_ix {
            // The arrowhead entity ID is `{edge_id}__arrow_head` (with
            // underscores, since `Id` only allows [a-zA-Z0-9_]).
            let edge_id = &svg_edge_info.edge_id;
            let arrow_head_key_str = format!("{edge_id}__arrow_head");
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
        .map(|taffy_node_mappings| {
            TaffyToSvgElementsMapper::map(
                &ir_example,
                &taffy_node_mappings,
                EdgeAnimationActive::Always,
            )
        })
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

#[test]
fn test_processes_rendered_expanded_skips_collapse_logic() -> Result<(), TaffyError> {
    let mut ir_example = serde_saphyr::from_str::<IrDiagram>(EXAMPLE_IR).unwrap();
    ir_example.render_options.process_render_collapse = ProcessRenderCollapse::ExpandAlways;

    let ir_to_taffy_builder = IrToTaffyBuilder::builder()
        .with_ir_diagram(&ir_example)
        .with_dimension_and_lods(vec![DimensionAndLod::default_2xl()])
        .build();
    ir_to_taffy_builder
        .build()
        .expect("Expected `taffy_node_mappings` to be built.")
        .map(|taffy_node_mappings| {
            TaffyToSvgElementsMapper::map(
                &ir_example,
                &taffy_node_mappings,
                EdgeAnimationActive::Always,
            )
        })
        .for_each(|svg_elements| {
            // Process step heights / infos are not computed when expanded.
            assert!(
                svg_elements.svg_process_infos.is_empty(),
                "Expected svg_process_infos to be empty when processes are rendered expanded"
            );

            // Process nodes still get a translate and a collapsed (full-height)
            // path, but no focus-driven expand classes.
            let Some(proc_app_dev_tailwind_classes) =
                svg_elements.tailwind_classes.get(&id!("proc_app_dev"))
            else {
                panic!("Expected tailwind classes for process node 'proc_app_dev'");
            };
            assert!(
                proc_app_dev_tailwind_classes.contains("translate-x-"),
                "Expected process node to have 'translate-x-' tailwind class"
            );
            assert!(
                !proc_app_dev_tailwind_classes
                    .contains(":focus-within]:[&>path.wrapper]:[d:path('"),
                "Expected process node to NOT have focus-driven expand classes when expanded. \
                 Got: {proc_app_dev_tailwind_classes}"
            );
        });

    Ok(())
}

/// Helper: run the full input-diagram -> IR -> taffy -> SVG pipeline for the
/// given input diagram.
fn build_svg_elements_for_diagram(
    input_diagram: &str,
) -> impl Iterator<Item = SvgElements<'static>> {
    let overlay_diagram = serde_saphyr::from_str::<InputDiagram>(input_diagram).unwrap();
    let merged = InputDiagramMerger::merge(InputDiagram::base(), &overlay_diagram);
    let IrDiagramAndIssues { diagram, .. } = InputToIrDiagramMapper::map(&merged);
    let diagram: IrDiagram<'static> = diagram.into_static();
    let ir_to_taffy_builder = IrToTaffyBuilder::builder()
        .with_ir_diagram(&diagram)
        .with_dimension_and_lods(vec![DimensionAndLod::default_2xl()])
        .build();
    let taffy_results: Vec<_> = ir_to_taffy_builder
        .build()
        .expect("Expected taffy_node_mappings to be built.")
        .collect();
    taffy_results
        .into_iter()
        .map(move |taffy_node_mappings| {
            TaffyToSvgElementsMapper::map(
                &diagram,
                &taffy_node_mappings,
                EdgeAnimationActive::Always,
            )
        })
        .collect::<Vec<_>>()
        .into_iter()
}

/// A fenced code block in a `thing_desc` renders as monospace line text sitting
/// inside a single unified `code` background box.
///
/// The box is one empty-text `code` span sized to the whole block (so the SVG
/// mapper draws its rounded background `<path>` once), and the code lines are
/// separate non-`code` spans that preserve their indentation.
#[test]
fn test_md_code_block_renders_unified_background_box() {
    for svg_elements in build_svg_elements_for_diagram(INPUT_DIAGRAM_0040_MD_CODE_BLOCK) {
        let node_info = svg_elements
            .svg_node_infos
            .iter()
            .find(|node_info| node_info.node_id.as_str() == "t_code")
            .expect("Expected t_code in svg_node_infos");

        // Exactly one unified background box: an empty-text `code` span taller
        // than a single line (it spans the three code lines plus padding).
        let code_bg_spans: Vec<_> = node_info
            .text_spans
            .iter()
            .filter(|span| span.md_style.as_ref().is_some_and(|md_style| md_style.code))
            .collect();
        assert_eq!(
            1,
            code_bg_spans.len(),
            "Expected exactly one unified code background span"
        );
        let code_bg = code_bg_spans[0];
        assert!(
            code_bg.text.is_empty(),
            "Expected the code background span to carry no text, got {:?}",
            code_bg.text
        );
        assert!(
            code_bg.height > TEXT_LINE_HEIGHT,
            "Expected the code box to span multiple lines, got height {}",
            code_bg.height
        );

        // The code lines render as (non-`code`) monospace spans, with leading
        // indentation preserved on the nested list item.
        let span_texts: Vec<&str> = node_info
            .text_spans
            .iter()
            .map(|span| span.text.as_str())
            .collect();
        assert!(
            span_texts.contains(&"string: hello"),
            "Expected a `string: hello` code line span, got {span_texts:?}"
        );

        let item_line = node_info
            .text_spans
            .iter()
            .find(|span| span.text == "  - item 1")
            .unwrap_or_else(|| {
                panic!("Expected the indented `  - item 1` code line span, got {span_texts:?}")
            });
        assert!(
            item_line
                .md_style
                .as_ref()
                .is_some_and(|md_style| !md_style.code),
            "Expected code line text spans to use the non-code style"
        );
    }
}

/// A fenced code block nested inside a list item is indented so it aligns
/// under the item's content (past the list marker), and still renders its
/// unified `code` background box with interior indentation preserved.
#[test]
fn test_md_code_block_in_list_is_indented_under_item() {
    for svg_elements in build_svg_elements_for_diagram(INPUT_DIAGRAM_0041_MD_CODE_BLOCK_IN_LIST) {
        let node_info = svg_elements
            .svg_node_infos
            .iter()
            .find(|node_info| node_info.node_id.as_str() == "t_code")
            .expect("Expected t_code in svg_node_infos");

        // The top-level paragraph `Steps:` marks the un-indented left edge.
        let steps_span = node_info
            .text_spans
            .iter()
            .find(|span| span.text == "Steps:")
            .expect("Expected the `Steps:` paragraph span");

        // The unified code background box (empty-text `code` span).
        let code_bg = node_info
            .text_spans
            .iter()
            .find(|span| span.md_style.as_ref().is_some_and(|md_style| md_style.code))
            .expect("Expected the unified code background span");

        // The box is indented well past the un-indented paragraph (one list
        // tab, ~4 character widths) rather than sitting at the left edge.
        assert!(
            code_bg.x > steps_span.x + 4.0,
            "Expected the nested code box to be indented past `Steps:` \
             (box x {} vs steps x {})",
            code_bg.x,
            steps_span.x
        );

        // Interior indentation within the code block is preserved.
        let span_texts: Vec<&str> = node_info
            .text_spans
            .iter()
            .map(|span| span.text.as_str())
            .collect();
        assert!(
            span_texts.iter().any(|text| *text == "  nested: value"),
            "Expected the indented `  nested: value` code line span, got {span_texts:?}"
        );
    }
}

/// A blockquote renders as a single bordered box spanning all its lines, with
/// the quoted content indented past the left bar.
#[test]
fn test_md_blockquote_renders_bordered_box_with_indented_content() {
    for svg_elements in build_svg_elements_for_diagram(INPUT_DIAGRAM_0042_MD_BLOCKQUOTE) {
        let node_info = svg_elements
            .svg_node_infos
            .iter()
            .find(|node_info| node_info.node_id.as_str() == "t_quote")
            .expect("Expected t_quote in svg_node_infos");

        // Exactly one blockquote frame span: empty text, taller than a single
        // line (it wraps a paragraph plus a two-item list).
        let blockquote_spans: Vec<_> = node_info
            .text_spans
            .iter()
            .filter(|span| {
                span.md_style
                    .as_ref()
                    .is_some_and(|md_style| md_style.blockquote)
            })
            .collect();
        assert_eq!(
            1,
            blockquote_spans.len(),
            "Expected exactly one blockquote frame span"
        );
        let blockquote = blockquote_spans[0];
        assert!(
            blockquote.text.is_empty(),
            "Expected the blockquote frame span to carry no text, got {:?}",
            blockquote.text
        );
        assert!(
            blockquote.height > TEXT_LINE_HEIGHT,
            "Expected the blockquote box to span multiple lines, got height {}",
            blockquote.height
        );

        // The un-quoted `Intro:` paragraph marks the box's left edge; the
        // quoted content is indented to the right of the left bar.
        let intro_x = node_info
            .text_spans
            .iter()
            .find(|span| span.text == "Intro:")
            .expect("Expected the `Intro:` paragraph span")
            .x;
        let quoted_x = node_info
            .text_spans
            .iter()
            .find(|span| span.text == "A quoted")
            .expect("Expected the `A quoted` span")
            .x;
        // The box left aligns with un-quoted content; quoted text sits past the
        // 7px bar plus its gap.
        assert!(
            (blockquote.x - intro_x).abs() < 0.5,
            "Expected the blockquote box left ({}) to align with `Intro:` ({intro_x})",
            blockquote.x
        );
        assert!(
            quoted_x > blockquote.x + 7.0,
            "Expected quoted content ({quoted_x}) to be indented past the left bar \
             (box x {})",
            blockquote.x
        );

        // The frame is only visible if its fill class is actually generated by
        // `encre_css`. Render to SVG and assert both the `evenodd` frame path
        // and the `fill: var(--tw-neutral-400-500)` rule are present (without
        // the rule the path falls back to the node's inherited fill -- the
        // colour of the node background -- and the box is invisible).
        let svg = SvgElementsToSvgMapper::map(&svg_elements);
        assert!(
            svg.contains("fill-rule=\"evenodd\""),
            "Expected the blockquote frame path in the rendered SVG"
        );
        assert!(
            svg.contains("fill: var(--tw-neutral-400-500)"),
            "Expected the blockquote fill rule to be generated so the frame is visible"
        );
    }
}

/// The from-protrusion for `edge_dep_bob_charlie__0` must be large enough to
/// clear all sibling nodes at the same rank as `t_bob`'s Divergent ancestor.
///
/// In this diagram, `t_bob` and `t_alice_outer` share rank 0 at the root level.
/// `t_alice_outer` is taller than `t_bob` (it contains a child node). The edge
/// from `t_bob` to `t_charlie` must protrude far enough downward that its
/// routing horizontal segment falls in the gap between `t_alice_outer`'s
/// bottom edge and `t_charlie`'s top edge -- not through `t_alice_outer`.
#[test]
fn test_nested_node_edge_protrusion_from_bob_clears_alice_outer() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0001_NESTED_NODE_EDGE_PROTRUSION)
    {
        // Find the relevant nodes.
        let alice_outer = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_alice_outer")
            .expect("Expected t_alice_outer in svg_node_infos");
        let bob = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_bob")
            .expect("Expected t_bob in svg_node_infos");

        // Compute the expected minimum from_protrusion for t_bob.
        // The protrusion from t_bob's bottom face (y + height) must reach at
        // least t_alice_outer's bottom face so the routing segment is in the
        // gap below all rank-0 siblings.
        let alice_outer_bottom = alice_outer.y + alice_outer.height_collapsed;
        let bob_bottom = bob.y + bob.height_collapsed;
        let expected_min_from_protrusion = (alice_outer_bottom - bob_bottom).max(0.0);

        // Find the edge from t_bob to t_charlie.
        let bob_charlie_edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.from_node_id.as_str() == "t_bob" && e.to_node_id.as_str() == "t_charlie")
            .expect("Expected edge from t_bob to t_charlie");

        assert!(
            bob_charlie_edge.ortho_protrusion_params.from_protrusion
                >= expected_min_from_protrusion,
            "from_protrusion {:.2} for edge t_bob->t_charlie should be >= {:.2} \
             (t_alice_outer bottom {:.2} - t_bob bottom {:.2})",
            bob_charlie_edge.ortho_protrusion_params.from_protrusion,
            expected_min_from_protrusion,
            alice_outer_bottom,
            bob_bottom,
        );
    }
}

/// The from-protrusion for `edge_dep_bob_charlie__0` must push the routing
/// horizontal segment below `t_alice_outer`'s bottom edge.
///
/// The horizontal routing y-coordinate for an orthogonal edge is:
///   `routing_y = bob_bottom + from_protrusion + arc_radius`
///
/// This must be > `alice_outer_bottom` for the path not to overlap
/// `t_alice_outer`. Because `from_protrusion >= alice_outer_bottom -
/// bob_bottom`, we have `routing_y > alice_outer_bottom`.
#[test]
fn test_nested_node_edge_bob_charlie_routing_clears_alice_outer() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0001_NESTED_NODE_EDGE_PROTRUSION)
    {
        let alice_outer = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_alice_outer")
            .expect("Expected t_alice_outer in svg_node_infos");
        let bob = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_bob")
            .expect("Expected t_bob in svg_node_infos");

        let alice_outer_bottom = alice_outer.y + alice_outer.height_collapsed;
        let bob_bottom = bob.y + bob.height_collapsed;

        let bob_charlie_edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.from_node_id.as_str() == "t_bob" && e.to_node_id.as_str() == "t_charlie")
            .expect("Expected edge from t_bob to t_charlie");

        // The horizontal routing segment is at:
        //   routing_y = bob_bottom + from_protrusion + ARC_RADIUS
        // For the routing to clear t_alice_outer, we need routing_y >
        // alice_outer_bottom.
        let from_protrusion = bob_charlie_edge.ortho_protrusion_params.from_protrusion;
        let routing_y = bob_bottom + from_protrusion + ARC_RADIUS;

        assert!(
            routing_y > alice_outer_bottom,
            "Routing y {:.2} (bob_bottom {:.2} + from_protrusion {:.2} + arc_radius {:.2}) \
             must be > alice_outer_bottom {:.2} so the path does not overlap t_alice_outer",
            routing_y,
            bob_bottom,
            from_protrusion,
            ARC_RADIUS,
            alice_outer_bottom,
        );
    }
}

/// Two edges from nested nodes into other nested nodes, sharing the same rank
/// gap, must receive **distinct** `from_protrusion` and `to_protrusion` so
/// their lateral routing segments do not overlap.
///
/// Both edges clear the same divergent-ancestor sibling row, so before the
/// row-grouped staggering their protrusions collapsed onto a single value
/// (`from=23`, `to=73`). The fix in
/// `OrthoProtrusionCalculator::protrusions_adjust_for_divergent_siblings`
/// staggers endpoints clearing the same row `MIN_PROTRUSION_PX` apart.
fn assert_nested_node_edge_protrusions_distinct(
    input_diagram: &str,
    edge_a: (&str, &str),
    edge_b: (&str, &str),
) {
    for svg_elements in build_svg_elements_for_diagram(input_diagram) {
        let edge_find = |from: &str, to: &str| {
            svg_elements
                .svg_edge_infos
                .iter()
                .find(|e| e.from_node_id.as_str() == from && e.to_node_id.as_str() == to)
                .unwrap_or_else(|| panic!("Expected edge {from}->{to} in svg_edge_infos"))
        };
        let edge_info_a = edge_find(edge_a.0, edge_a.1);
        let edge_info_b = edge_find(edge_b.0, edge_b.1);

        let from_protrusion_a = edge_info_a.ortho_protrusion_params.from_protrusion;
        let from_protrusion_b = edge_info_b.ortho_protrusion_params.from_protrusion;
        let to_protrusion_a = edge_info_a.ortho_protrusion_params.to_protrusion;
        let to_protrusion_b = edge_info_b.ortho_protrusion_params.to_protrusion;

        assert!(
            (from_protrusion_a - from_protrusion_b).abs() >= MIN_PROTRUSION_PX - 1e-3,
            "from_protrusion for {edge_a:?} ({from_protrusion_a:.2}) and {edge_b:?} \
             ({from_protrusion_b:.2}) must differ by >= {MIN_PROTRUSION_PX} so their lateral \
             segments do not overlap",
        );
        assert!(
            (to_protrusion_a - to_protrusion_b).abs() >= MIN_PROTRUSION_PX - 1e-3,
            "to_protrusion for {edge_a:?} ({to_protrusion_a:.2}) and {edge_b:?} \
             ({to_protrusion_b:.2}) must differ by >= {MIN_PROTRUSION_PX} so their lateral \
             segments do not overlap",
        );
    }
}

/// `0027`: two edges into the **same** nested node (`t_c_00`) from nested nodes
/// in different sibling containers must not share protrusion depths.
#[test]
fn test_nested_node_edge_protrusion_to_same_nested_node_distinct() {
    assert_nested_node_edge_protrusions_distinct(
        INPUT_DIAGRAM_0027_NESTED_NODE_EDGE_PROTRUSION_TO_NESTED_NODE_1,
        ("t_b_00", "t_c_00"),
        ("t_a_00", "t_c_00"),
    );
}

/// `0028`: two edges into **different** nested nodes (`t_c_00` / `t_d_00`) at
/// the same rank must not share protrusion depths -- both still clear the same
/// divergent-ancestor sibling rows.
#[test]
fn test_nested_node_edge_protrusion_to_different_nested_nodes_distinct() {
    assert_nested_node_edge_protrusions_distinct(
        INPUT_DIAGRAM_0028_NESTED_NODE_EDGE_PROTRUSION_TO_NESTED_NODE_2,
        ("t_b_00", "t_c_00"),
        ("t_a_00", "t_d_00"),
    );
}

// === Cycle edge routing tests === //

/// Parse all SVG path endpoint coordinates (from `M` and `L` commands) from a
/// path `d` attribute string.
///
/// Returns a `Vec<(f32, f32)>` of `(x, y)` pairs.
fn parse_path_endpoints(path_d: &str) -> Vec<(f32, f32)> {
    // kurbo concatenates each command letter with its first coordinate
    // (e.g. `M86,176`, `C86,209 84,211 82,211`), so strip a leading command
    // letter before parsing. Records the endpoint vertex of each command --
    // `MoveTo` / `LineTo` targets and the final point of `CurveTo` / `QuadTo`
    // segments -- skipping bezier control points.
    let mut result = Vec::new();
    let tokens: Vec<&str> = path_d.split_whitespace().collect();
    let mut i = 0;
    while i < tokens.len() {
        let token = tokens[i];
        match token.chars().next() {
            Some('M') | Some('L') => {
                if let Some(coords) = parse_coord_pair(&token[1..]) {
                    result.push(coords);
                }
                i += 1;
            }
            Some('C') => {
                // Curve: ctrl1 ctrl2 endpoint -- record only the endpoint.
                if let Some(coords) = tokens.get(i + 2).and_then(|t| parse_coord_pair(t)) {
                    result.push(coords);
                }
                i += 3;
            }
            Some('Q') => {
                // Quadratic: ctrl endpoint -- record only the endpoint.
                if let Some(coords) = tokens.get(i + 1).and_then(|t| parse_coord_pair(t)) {
                    result.push(coords);
                }
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }
    result
}

/// Parse a `"x,y"` token into a `(f32, f32)` pair.
fn parse_coord_pair(s: &str) -> Option<(f32, f32)> {
    let mut parts = s.splitn(2, ',');
    let x: f32 = parts.next()?.parse().ok()?;
    let y: f32 = parts.next()?.parse().ok()?;
    Some((x, y))
}

/// Signed area of the triangle `(a, b, c)`; sign gives the orientation.
fn orientation(a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> f32 {
    (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
}

/// Whether segments `p1-p2` and `p3-p4` cross at an interior point.
///
/// Uses strict orientation tests, so shared endpoints or collinear touches do
/// not count as a crossing -- only a genuine X-shaped intersection does.
fn segments_properly_intersect(
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
    p4: (f32, f32),
) -> bool {
    let d1 = orientation(p3, p4, p1);
    let d2 = orientation(p3, p4, p2);
    let d3 = orientation(p1, p2, p3);
    let d4 = orientation(p1, p2, p4);
    ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
        && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
}

/// Whether any segment of polyline `a` properly intersects any segment of
/// polyline `b`.
fn polylines_cross(a: &[(f32, f32)], b: &[(f32, f32)]) -> bool {
    a.windows(2).any(|sa| {
        b.windows(2)
            .any(|sb| segments_properly_intersect(sa[0], sa[1], sb[0], sb[1]))
    })
}

/// Shortest distance from point `p` to segment `a-b`.
fn point_segment_distance(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let abx = b.0 - a.0;
    let aby = b.1 - a.1;
    let len_sq = abx * abx + aby * aby;
    let t = if len_sq <= f32::EPSILON {
        0.0
    } else {
        (((p.0 - a.0) * abx + (p.1 - a.1) * aby) / len_sq).clamp(0.0, 1.0)
    };
    let foot = (a.0 + t * abx, a.1 + t * aby);
    ((p.0 - foot.0).powi(2) + (p.1 - foot.1).powi(2)).sqrt()
}

/// Shortest distance between segments `p1-p2` and `p3-p4` (0 if they cross).
fn segment_segment_distance(p1: (f32, f32), p2: (f32, f32), p3: (f32, f32), p4: (f32, f32)) -> f32 {
    if segments_properly_intersect(p1, p2, p3, p4) {
        return 0.0;
    }
    point_segment_distance(p1, p3, p4)
        .min(point_segment_distance(p2, p3, p4))
        .min(point_segment_distance(p3, p1, p2))
        .min(point_segment_distance(p4, p1, p2))
}

/// Minimum perpendicular distance between any axis-aligned segment of `a` and
/// any **parallel** (same-orientation) axis-aligned segment of `b` whose
/// extents overlap along the shared axis.
///
/// Unlike [`polylines_min_distance`], this ignores perpendicular segment pairs
/// (clean X-crossings, which are visually acceptable) and non-overlapping
/// parallel pairs. It therefore measures only coincident/parallel runs -- the
/// "two edges reading as one line" defect. Diagonal arc-corner segments are
/// skipped (they are neither horizontal nor vertical).
fn parallel_segment_min_gap(a: &[(f32, f32)], b: &[(f32, f32)]) -> f32 {
    let eps = 1e-2_f32;
    let mut min_gap = f32::INFINITY;
    for sa in a.windows(2) {
        let a_horiz = (sa[0].1 - sa[1].1).abs() < eps;
        let a_vert = (sa[0].0 - sa[1].0).abs() < eps;
        for sb in b.windows(2) {
            let b_horiz = (sb[0].1 - sb[1].1).abs() < eps;
            let b_vert = (sb[0].0 - sb[1].0).abs() < eps;
            if a_horiz && b_horiz {
                let a_lo = sa[0].0.min(sa[1].0);
                let a_hi = sa[0].0.max(sa[1].0);
                let b_lo = sb[0].0.min(sb[1].0);
                let b_hi = sb[0].0.max(sb[1].0);
                if a_hi.min(b_hi) - a_lo.max(b_lo) > eps {
                    min_gap = min_gap.min((sa[0].1 - sb[0].1).abs());
                }
            } else if a_vert && b_vert {
                let a_lo = sa[0].1.min(sa[1].1);
                let a_hi = sa[0].1.max(sa[1].1);
                let b_lo = sb[0].1.min(sb[1].1);
                let b_hi = sb[0].1.max(sb[1].1);
                if a_hi.min(b_hi) - a_lo.max(b_lo) > eps {
                    min_gap = min_gap.min((sa[0].0 - sb[0].0).abs());
                }
            }
        }
    }
    min_gap
}

/// Shortest distance between any segment of polyline `a` and any of `b`.
fn polylines_min_distance(a: &[(f32, f32)], b: &[(f32, f32)]) -> f32 {
    a.windows(2)
        .flat_map(|sa| {
            b.windows(2)
                .map(move |sb| segment_segment_distance(sa[0], sa[1], sb[0], sb[1]))
        })
        .fold(f32::INFINITY, f32::min)
}

// === Tag and process step node routing tests === //

/// Builds `SvgElements` from the tag-nodes cyclic edge fixture.
///
/// The fixture has 3 tags (`tag_a`, `tag_b`, `tag_c`) connected by a cyclic
/// edge group (`edge_dep_tags_cyclic`), producing edges `tag_a -> tag_b`,
/// `tag_b -> tag_c`, `tag_c -> tag_a`. All three tags end up at the same rank
/// due to the cycle.
fn build_svg_elements_from_tag_nodes_cyclic_edge() -> impl Iterator<Item = SvgElements<'static>> {
    build_svg_elements_for_diagram(INPUT_DIAGRAM_0005_TAG_NODES_CYCLIC_EDGE)
}

/// Builds `SvgElements` from the process-step-nodes cyclic edge fixture.
///
/// The fixture has:
/// - 3 thing nodes (`t_alice`, `t_bob`, `t_charlie`) connected by a symmetric
///   edge group.
/// - A process `proc_test` with 3 steps (`proc_test_step_a`,
///   `proc_test_step_b`, `proc_test_step_c`) connected by a cyclic edge group
///   (`edge_dep_proc_steps_cyclic`). All three steps end up at the same rank
///   due to the cycle.
fn build_svg_elements_from_process_step_nodes_cyclic_edge(
) -> impl Iterator<Item = SvgElements<'static>> {
    build_svg_elements_for_diagram(INPUT_DIAGRAM_0006_PROCESS_STEP_NODES_CYCLIC_EDGE)
}

/// Tag nodes use cycle routing around other tag nodes, and nothing else.
///
/// The fixture has 3 tags at the same rank connected by a cyclic edge group.
/// The wrapping edge `tag_c -> tag_a` (positions 2 and 0, diff = 2) triggers
/// cycle routing.
#[test]
fn test_tag_node_edges_protrusion_is_zero() {
    for svg_elements in build_svg_elements_from_tag_nodes_cyclic_edge() {
        // tag_a -> tag_b
        let edge_tag_a_b = svg_elements
            .svg_edge_infos
            .iter()
            .find(|edge_info| edge_info.edge_id.as_str() == "edge_dep_tags_cyclic__0")
            .expect("Expected edge to exist.");
        // tag_b -> tag_c
        let edge_tag_b_c = svg_elements
            .svg_edge_infos
            .iter()
            .find(|edge_info| edge_info.edge_id.as_str() == "edge_dep_tags_cyclic__1")
            .expect("Expected edge to exist.");
        // tag_c -> tag_a
        let edge_tag_c_a = svg_elements
            .svg_edge_infos
            .iter()
            .find(|edge_info| edge_info.edge_id.as_str() == "edge_dep_tags_cyclic__2")
            .expect("Expected edge to exist.");

        assert_eq!(
            0.0,
            edge_tag_a_b.ortho_protrusion_params.from_protrusion,
            "Tag-node edge {:?} ({} -> {}) from_protrusion {:.2} should be 0 \
             (direct edge)",
            edge_tag_a_b.edge_id,
            edge_tag_a_b.from_node_id,
            edge_tag_a_b.to_node_id,
            edge_tag_a_b.ortho_protrusion_params.from_protrusion,
        );
        assert_eq!(
            0.0,
            edge_tag_a_b.ortho_protrusion_params.to_protrusion,
            "Tag-node edge {:?} ({} -> {}) to_protrusion {:.2} should be 0 \
             (direct edge)",
            edge_tag_a_b.edge_id,
            edge_tag_a_b.from_node_id,
            edge_tag_a_b.to_node_id,
            edge_tag_a_b.ortho_protrusion_params.to_protrusion,
        );

        assert_eq!(
            0.0,
            edge_tag_b_c.ortho_protrusion_params.from_protrusion,
            "Tag-node edge {:?} ({} -> {}) from_protrusion {:.2} should be 0 \
             (direct edge)",
            edge_tag_b_c.edge_id,
            edge_tag_b_c.from_node_id,
            edge_tag_b_c.to_node_id,
            edge_tag_b_c.ortho_protrusion_params.from_protrusion,
        );
        assert_eq!(
            0.0,
            edge_tag_b_c.ortho_protrusion_params.to_protrusion,
            "Tag-node edge {:?} ({} -> {}) to_protrusion {:.2} should be 0 \
             (direct edge)",
            edge_tag_b_c.edge_id,
            edge_tag_b_c.from_node_id,
            edge_tag_b_c.to_node_id,
            edge_tag_b_c.ortho_protrusion_params.to_protrusion,
        );

        assert!(
            edge_tag_c_a.ortho_protrusion_params.from_protrusion > 0.0,
            "Tag-node edge {:?} ({} -> {}) from_protrusion {:.2} should be greater than 0 \
                (loops around b)",
            edge_tag_c_a.edge_id,
            edge_tag_c_a.from_node_id,
            edge_tag_c_a.to_node_id,
            edge_tag_c_a.ortho_protrusion_params.from_protrusion,
        );
        assert!(
            edge_tag_c_a.ortho_protrusion_params.to_protrusion > 0.0,
            "Tag-node edge {:?} ({} -> {}) to_protrusion {:.2} should be greater than 0 \
                (loops around b)",
            edge_tag_c_a.edge_id,
            edge_tag_c_a.from_node_id,
            edge_tag_c_a.to_node_id,
            edge_tag_c_a.ortho_protrusion_params.to_protrusion,
        );
    }
}

/// In a `LeftToRight` diagram containing both thing nodes and a process node,
/// thing-node cycle edges must be routed using only thing-node sibling extents
/// -- not clearing process nodes.
///
/// In the `0006` fixture, thing nodes sit in a single vertical column at
/// `x = 20`, `width = 83` (right face at `x = 103`). The process node starts
/// further to the right. The non-adjacent same-rank edge `alice -> charlie`
/// uses Right/Right face routing and protrudes to the right. Before the
/// grouping fix, the protrusion was computed as 179 px (clearing the process
/// node's right edge). After the fix, the protrusion is based only on
/// thing-node sibling extents, so every path coordinate remains to the left
/// of the process node.
#[test]
fn test_thing_node_cycle_edges_not_routed_around_process_node() {
    for svg_elements in build_svg_elements_from_process_step_nodes_cyclic_edge() {
        let Some(proc_node) = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "proc_test")
        else {
            continue;
        };
        // Any thing-node edge path coordinate must stay strictly to the left
        // of the process node's left boundary.
        let process_left_x = proc_node.x;

        for edge in &svg_elements.svg_edge_infos {
            // Skip process-step connectors (`edge_ps_*`): these route inside
            // the process node by design, and their step endpoints have
            // `process_id` None in `SvgNodeInfo` so the thing-node check below
            // would not otherwise exclude them.
            if edge.edge_id.as_str().starts_with("edge_ps_") {
                continue;
            }

            // Only check edges where both endpoints are thing nodes
            // (process_id is None for thing/tag nodes; Some for process nodes).
            let from_node = svg_elements
                .svg_node_infos
                .iter()
                .find(|n| n.node_id == edge.from_node_id);
            let to_node = svg_elements
                .svg_node_infos
                .iter()
                .find(|n| n.node_id == edge.to_node_id);
            let is_thing_edge = from_node.map_or(false, |n| n.process_id.is_none())
                && to_node.map_or(false, |n| n.process_id.is_none());
            if !is_thing_edge {
                continue;
            }

            let coords = parse_path_endpoints(&edge.path_d);
            for (x, _y) in &coords {
                assert!(
                    *x < process_left_x,
                    "Thing-node edge {:?} ({} -> {}) has path point x={:.2} >= \
                     process left boundary x={:.2}; edge is being routed around \
                     the process node instead of only around thing nodes",
                    edge.edge_id,
                    edge.from_node_id,
                    edge.to_node_id,
                    x,
                    process_left_x,
                );
            }
        }
    }
}

/// Process steps without explicit `process_step_dependencies` are laid out in
/// declaration order along the process's flex direction (assumed linear
/// dependencies).
///
/// The `proc_test` process declares steps `a`, `b`, `c` with no
/// `process_step_dependencies`, so they are assumed to depend linearly in
/// declaration order, giving ranks 0, 1, 2. The process container lays steps
/// out in a column, so the step `y` coordinates must increase `a < b < c`.
#[test]
fn test_process_steps_ordered_by_rank_when_dependencies_absent() {
    for svg_elements in build_svg_elements_from_process_step_nodes_cyclic_edge() {
        let step_y = |step: &str| {
            svg_elements
                .svg_node_infos
                .iter()
                .find(|node_info| node_info.node_id.as_str() == step)
                .unwrap_or_else(|| panic!("Expected node {step} to exist."))
                .y
        };

        let y_a = step_y("proc_test_step_a");
        let y_b = step_y("proc_test_step_b");
        let y_c = step_y("proc_test_step_c");

        assert!(
            y_a < y_b && y_b < y_c,
            "Expected process steps ordered by rank along the column \
             (a < b < c), got a={y_a:.2}, b={y_b:.2}, c={y_c:.2}"
        );
    }
}

/// Builds `SvgElements` from the process-step branch/merge fixture.
fn build_svg_elements_from_process_step_branch_merge() -> impl Iterator<Item = SvgElements<'static>>
{
    build_svg_elements_for_diagram(INPUT_DIAGRAM_0018_PROCESS_STEP_BRANCH_MERGE)
}

/// In the branch/merge process, the bypassed step C is shifted to a higher
/// lane (further right) than the lane-0 steps, and a git-style connector is
/// drawn for each of the four process step dependencies.
#[test]
fn test_process_step_graph_bypass_shifts_circle_right_and_draws_connectors() {
    for svg_elements in build_svg_elements_from_process_step_branch_merge() {
        // Absolute circle centre x for a step (node.x + circle.cx).
        let circle_cx = |step: &str| {
            let node_info = svg_elements
                .svg_node_infos
                .iter()
                .find(|node_info| node_info.node_id.as_str() == step)
                .unwrap_or_else(|| panic!("Expected node {step} to exist."));
            let circle = node_info
                .circle
                .as_ref()
                .unwrap_or_else(|| panic!("Expected {step} to have a circle."));
            node_info.x + circle.cx
        };

        let cx_a = circle_cx("proc_build_step_a");
        let cx_b = circle_cx("proc_build_step_b");
        let cx_c = circle_cx("proc_build_step_c");
        let cx_d = circle_cx("proc_build_step_d");

        // Lane-0 steps share the same lane x; C (lane 1) is shifted right.
        assert!(
            (cx_a - cx_b).abs() < 0.5 && (cx_a - cx_d).abs() < 0.5,
            "Lane-0 steps A, B, D should share an x lane, got a={cx_a:.2}, b={cx_b:.2}, d={cx_d:.2}"
        );
        assert!(
            cx_c > cx_a + 1.0,
            "Bypassed step C should be shifted right of lane 0, got a={cx_a:.2}, c={cx_c:.2}"
        );

        // One connector per process step dependency.
        let connector_count = svg_elements
            .svg_edge_infos
            .iter()
            .filter(|edge_info| edge_info.edge_id.as_str().starts_with("edge_ps_"))
            .count();
        assert_eq!(
            4, connector_count,
            "Expected 4 process step connectors (A->B, A->C, B->D, C->D)"
        );
    }
}

/// Process step labels are left-aligned in a single column regardless of each
/// step's lane.
///
/// The bypassed step C sits in lane 1, but its label must start at the same
/// absolute x as the lane-0 steps' labels (the circle is offset, not the text).
#[test]
fn test_process_step_graph_labels_aligned_across_lanes() {
    for svg_elements in build_svg_elements_from_process_step_branch_merge() {
        // Absolute x where a step's label text begins (node.x + first span x).
        let label_x = |step: &str| {
            let node_info = svg_elements
                .svg_node_infos
                .iter()
                .find(|node_info| node_info.node_id.as_str() == step)
                .unwrap_or_else(|| panic!("Expected node {step} to exist."));
            let span = node_info
                .text_spans
                .first()
                .unwrap_or_else(|| panic!("Expected {step} to have a label span."));
            node_info.x + span.x
        };

        let label_x_a = label_x("proc_build_step_a");
        for step in [
            "proc_build_step_b",
            "proc_build_step_c",
            "proc_build_step_d",
        ] {
            assert!(
                (label_x(step) - label_x_a).abs() < 0.5,
                "Label for {step} should be aligned with A's label, \
                 got {} vs {label_x_a}",
                label_x(step)
            );
        }
    }
}

/// Process step connectors carry an arrowhead, a locus path, and theme-derived
/// tailwind classes (resolved from the theme's edge_defaults), like dependency
/// edges.
#[test]
fn test_process_step_graph_connectors_have_arrow_locus_and_classes() {
    for svg_elements in build_svg_elements_from_process_step_branch_merge() {
        let connectors: Vec<_> = svg_elements
            .svg_edge_infos
            .iter()
            .filter(|edge_info| edge_info.edge_id.as_str().starts_with("edge_ps_"))
            .collect();
        assert_eq!(4, connectors.len(), "Expected 4 process step connectors");

        for connector in &connectors {
            assert!(
                connector.arrow_head_path_d.starts_with('M')
                    && connector.arrow_head_path_d.contains('Z'),
                "Connector {:?} should have a closed arrowhead, got: {}",
                connector.edge_id,
                connector.arrow_head_path_d
            );
            assert!(
                !connector.locus_path_d.is_empty(),
                "Connector {:?} should have a locus path",
                connector.edge_id
            );

            // The connector is styled from the theme's edge_defaults, so it has
            // a stroke (the line) resolved into its tailwind classes.
            let classes = svg_elements
                .tailwind_classes
                .get(connector.edge_id.as_ref())
                .unwrap_or_else(|| {
                    panic!(
                        "Connector {:?} should have tailwind classes",
                        connector.edge_id
                    )
                });
            assert!(
                classes.contains("stroke-"),
                "Connector {:?} classes should include a stroke, got: {classes}",
                connector.edge_id
            );
        }
    }
}

/// Builds `SvgElements` from the 2-node symmetric edge fixture.
fn build_svg_elements_from_symmetric_2_nodes() -> impl Iterator<Item = SvgElements<'static>> {
    build_svg_elements_for_diagram(INPUT_DIAGRAM_0003_EDGES_SYMMETRIC_2_NODES)
}

/// Builds `SvgElements` from the 3-node symmetric edge fixture.
fn build_svg_elements_from_symmetric_3_nodes() -> impl Iterator<Item = SvgElements<'static>> {
    build_svg_elements_for_diagram(INPUT_DIAGRAM_0004_EDGES_SYMMETRIC_3_NODES)
}

/// For the 2-node symmetric edge diagram, edges between adjacent siblings
/// must have zero protrusion.
///
/// `t_alice` (position 0) and `t_bob` (position 1) are adjacent siblings with
/// the same direct parent. Adjacent siblings use normal face-selection routing
/// (connecting the two closest `NodeFace`s) instead of clockwise cycle routing,
/// so no protrusion is needed and both `from_protrusion` and `to_protrusion`
/// must be exactly 0.
#[test]
fn test_adjacent_siblings_protrusion_is_zero() {
    for svg_elements in build_svg_elements_from_symmetric_2_nodes() {
        for edge in &svg_elements.svg_edge_infos {
            assert_eq!(
                edge.ortho_protrusion_params.from_protrusion, 0.0,
                "Adjacent-sibling edge {:?} from_protrusion {:.2} should be 0 \
                 (normal routing, no cycle protrusion)",
                edge.edge_id, edge.ortho_protrusion_params.from_protrusion,
            );
            assert_eq!(
                edge.ortho_protrusion_params.to_protrusion, 0.0,
                "Adjacent-sibling edge {:?} to_protrusion {:.2} should be 0 \
                 (normal routing, no cycle protrusion)",
                edge.edge_id, edge.ortho_protrusion_params.to_protrusion,
            );
        }
    }
}

/// For the 2-node symmetric edge diagram, no edge path coordinate must fall
/// strictly inside any node bounding box.
///
/// With normal (nearest-face) routing for adjacent siblings:
/// - `alice -> bob` (`alice.x < bob.x`) uses `Right`/`Left` faces: path
///   segments travel between alice's right edge and bob's left edge.
/// - `bob -> alice` (`bob.x > alice.x`) uses `Left`/`Right` faces: path
///   segments travel between bob's left edge and alice's right edge.
#[test]
fn test_cycle_edges_2_nodes_no_overlap_with_nodes() {
    for svg_elements in build_svg_elements_from_symmetric_2_nodes() {
        for edge in &svg_elements.svg_edge_infos {
            let from_node = svg_elements
                .svg_node_infos
                .iter()
                .find(|n| n.node_id == edge.from_node_id)
                .expect("from node");
            let to_node = svg_elements
                .svg_node_infos
                .iter()
                .find(|n| n.node_id == edge.to_node_id)
                .expect("to node");

            let coords = parse_path_endpoints(&edge.path_d);
            for (x, y) in coords {
                // Check against every node in the diagram.
                for node in &svg_elements.svg_node_infos {
                    let node_x_min = node.x;
                    let node_x_max = node.x + node.width;
                    let node_y_min = node.y;
                    let node_y_max = node.y + node.height_collapsed;

                    let strictly_inside =
                        x > node_x_min && x < node_x_max && y > node_y_min && y < node_y_max;

                    assert!(
                        !strictly_inside,
                        "Edge {:?} ({} -> {}) has path point ({:.2}, {:.2}) inside node {:?} \
                         bounding box x=[{:.2},{:.2}] y=[{:.2},{:.2}]",
                        edge.edge_id,
                        from_node.node_id,
                        to_node.node_id,
                        x,
                        y,
                        node.node_id,
                        node_x_min,
                        node_x_max,
                        node_y_min,
                        node_y_max,
                    );
                }
            }
        }
    }
}

/// For the 3-node symmetric edge diagram, non-adjacent same-rank edges must
/// have a non-zero protrusion.
///
/// The edge group uses `things: [t_bob, t_alice, t_charlie]` with
/// `kind: symmetric`, producing edges: `t_bob -> t_alice`, `t_alice ->
/// t_charlie`, `t_charlie -> t_alice`, `t_alice -> t_bob`. The hierarchy
/// positions are `t_alice=0`, `t_bob=1`, `t_charlie=2`.
///
/// Adjacent-sibling edges (`t_bob <-> t_alice`, position diff = 1) use normal
/// routing and have zero protrusion. Non-adjacent edges (`t_alice <->
/// t_charlie`, position diff = 2) are true cycle edges and must protrude by at
/// least `MIN_PROTRUSION_PX` (3.0 px).
#[test]
fn test_cycle_edges_3_nodes_protrusion_non_zero() {
    const MIN_PROTRUSION_PX: f32 = 3.0;

    for svg_elements in build_svg_elements_from_symmetric_3_nodes() {
        for edge in &svg_elements.svg_edge_infos {
            // Only check non-adjacent same-rank edges (t_alice <-> t_charlie).
            let is_non_adjacent_cycle_edge = (edge.from_node_id.as_str() == "t_alice"
                && edge.to_node_id.as_str() == "t_charlie")
                || (edge.from_node_id.as_str() == "t_charlie"
                    && edge.to_node_id.as_str() == "t_alice");
            if !is_non_adjacent_cycle_edge {
                continue;
            }
            assert!(
                edge.ortho_protrusion_params.from_protrusion >= MIN_PROTRUSION_PX,
                "Non-adjacent cycle edge {:?} from_protrusion {:.2} should be >= {:.2}",
                edge.edge_id,
                edge.ortho_protrusion_params.from_protrusion,
                MIN_PROTRUSION_PX,
            );
            assert!(
                edge.ortho_protrusion_params.to_protrusion >= MIN_PROTRUSION_PX,
                "Non-adjacent cycle edge {:?} to_protrusion {:.2} should be >= {:.2}",
                edge.edge_id,
                edge.ortho_protrusion_params.to_protrusion,
                MIN_PROTRUSION_PX,
            );
        }
    }
}

/// For the 3-node symmetric edge diagram, no edge path coordinate must fall
/// strictly inside any node bounding box.
///
/// For the 3-node symmetric edge diagram, each edge's `from_protrusion` must
/// equal its `to_protrusion` (symmetric U-shaped arc), and edges that route in
/// the same direction (same face) must have distinct protrusion depths so their
/// routing segments do not overlap.
#[test]
fn test_cycle_edges_3_nodes_symmetric_and_distinct_protrusions() {
    for svg_elements in build_svg_elements_from_symmetric_3_nodes() {
        // Verify from == to for every edge.
        for edge in &svg_elements.svg_edge_infos {
            assert_eq!(
                edge.ortho_protrusion_params.from_protrusion,
                edge.ortho_protrusion_params.to_protrusion,
                "Edge {:?} from_protrusion {:.2} != to_protrusion {:.2}",
                edge.edge_id,
                edge.ortho_protrusion_params.from_protrusion,
                edge.ortho_protrusion_params.to_protrusion,
            );
        }

        // Verify that not all cycle edges have the same protrusion depth.
        // With 3+ edges in the diagram there must be at least two distinct
        // protrusion values (edges in the same direction group are stacked).
        let mut protrusions: Vec<f32> = svg_elements
            .svg_edge_infos
            .iter()
            .map(|e| e.ortho_protrusion_params.from_protrusion)
            .collect();
        protrusions.sort_by(|a, b| a.partial_cmp(b).unwrap());
        protrusions.dedup();
        assert!(
            protrusions.len() > 1,
            "All cycle edges have the same protrusion {:.2}; expected distinct values",
            protrusions[0],
        );
    }
}

/// All nodes are in the same column (`x = 20`, `width = 83`). Downward edges
/// route to the right (`x >= node.x + node.width`) and upward edges route to
/// the left (`x <= node.x`).
#[test]
fn test_cycle_edges_3_nodes_no_overlap_with_nodes() {
    for svg_elements in build_svg_elements_from_symmetric_3_nodes() {
        for edge in &svg_elements.svg_edge_infos {
            let from_node = svg_elements
                .svg_node_infos
                .iter()
                .find(|n| n.node_id == edge.from_node_id)
                .expect("from node");
            let to_node = svg_elements
                .svg_node_infos
                .iter()
                .find(|n| n.node_id == edge.to_node_id)
                .expect("to node");

            let coords = parse_path_endpoints(&edge.path_d);
            for (x, y) in coords {
                for node in &svg_elements.svg_node_infos {
                    let node_x_min = node.x;
                    let node_x_max = node.x + node.width;
                    let node_y_min = node.y;
                    let node_y_max = node.y + node.height_collapsed;

                    let strictly_inside =
                        x > node_x_min && x < node_x_max && y > node_y_min && y < node_y_max;

                    assert!(
                        !strictly_inside,
                        "Edge {:?} ({} -> {}) has path point ({:.2}, {:.2}) inside node {:?} \
                         bounding box x=[{:.2},{:.2}] y=[{:.2},{:.2}]",
                        edge.edge_id,
                        from_node.node_id,
                        to_node.node_id,
                        x,
                        y,
                        node.node_id,
                        node_x_min,
                        node_x_max,
                        node_y_min,
                        node_y_max,
                    );
                }
            }
        }
    }
}

// === Edge from node to nested node (0007) === //

/// Loads `0007_edge_from_node_to_nested_node.yaml` and returns one
/// `SvgElements` per LOD.
fn build_svg_elements_from_edge_from_node_to_nested_node(
) -> impl Iterator<Item = SvgElements<'static>> {
    build_svg_elements_for_diagram(INPUT_DIAGRAM_0007_EDGE_FROM_NODE_TO_NESTED_NODE)
}

/// An edge from `t_alice` to `t_charlie_1` connects to a node at rank 0 inside
/// its parent container `t_charlie_outer`. The edge should NOT route through a
/// cross-container spacer alongside `t_charlie_2`, because both
/// `t_charlie_1` and `t_charlie_2` are at rank 0 (side-by-side) -- there are
/// no intermediate siblings between the container entry and the target.
#[test]
fn test_edge_to_nested_rank_0_node_has_no_cross_container_spacer() {
    for svg_elements in build_svg_elements_from_edge_from_node_to_nested_node() {
        let alice_charlie_1_edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| {
                e.from_node_id.as_str() == "t_alice" && e.to_node_id.as_str() == "t_charlie_1"
            })
            .expect("Expected edge from t_alice to t_charlie_1");

        assert!(
            alice_charlie_1_edge
                .ortho_protrusion_params
                .spacer_protrusions
                .is_empty(),
            "Expected no spacer protrusions for edge t_alice -> t_charlie_1 \
             (t_charlie_1 is at rank 0 inside t_charlie_outer, so no siblings \
             are between the container entry and the target): \
             spacer_protrusions = {:?}",
            alice_charlie_1_edge
                .ortho_protrusion_params
                .spacer_protrusions,
        );
    }
}

/// An edge from `t_bob` (top-level, rank 0) to `t_charlie_1` (rank 0 inside
/// `t_charlie_outer`, which is rank 1 at root level) should use normal face
/// routing -- Bottom of `t_bob` to Top of `t_charlie_1` -- not cycle-edge
/// clockwise routing.
///
/// The incorrect behaviour (before the fix) was to compare only the local
/// context rank of each node, both of which happen to be 0, triggering the
/// same-rank cycle edge detection. The fix uses LCA-level ranks instead:
/// `t_bob` is rank 0 and `t_charlie_outer` (the divergent ancestor of
/// `t_charlie_1` at the root LCA level) is rank 1, so the edge is correctly
/// classified as a forward edge.
#[test]
fn test_edge_from_toplevel_to_nested_rank_0_node_uses_normal_face_routing() {
    for svg_elements in build_svg_elements_from_edge_from_node_to_nested_node() {
        let bob = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_bob")
            .expect("Expected t_bob in svg_node_infos");
        let charlie_1 = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_charlie_1")
            .expect("Expected t_charlie_1 in svg_node_infos");

        let bob_charlie_1_edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.from_node_id.as_str() == "t_bob" && e.to_node_id.as_str() == "t_charlie_1")
            .expect("Expected edge from t_bob to t_charlie_1");

        // The path is built from from-node to to-node in the SVG direction.
        // For a Bottom (t_bob) -> Top (t_charlie_1) edge the first SVG `M`
        // command should have y near t_bob's bottom face
        // (bob.y + bob.height_collapsed) and the final `L` command should be
        // near t_charlie_1's top face (charlie_1.y).
        //
        // Note: kurbo generates concatenated path commands (e.g. `M80,210`
        // rather than `M 80,210`), so we parse the first/last tokens directly
        // rather than using the generic `parse_path_endpoints` helper.
        let path_tokens: Vec<&str> = bob_charlie_1_edge.path_d.split_whitespace().collect();
        assert!(
            !path_tokens.is_empty(),
            "Expected non-empty path for edge t_bob -> t_charlie_1"
        );

        let parse_suffixed = |s: &str, prefix: char| -> Option<(f32, f32)> {
            let s = s.strip_prefix(prefix)?;
            let (x_str, y_str) = s.split_once(',')?;
            Some((x_str.parse().ok()?, y_str.parse().ok()?))
        };

        // Allow a tolerance for protrusion stubs and face offsets.
        let tolerance = 20.0_f32;

        let (_, first_y) = path_tokens
            .first()
            .and_then(|t| parse_suffixed(t, 'M'))
            .expect("Path should start with M command (e.g. M80,210)");
        let expected_first_y = bob.y + bob.height_collapsed; // Bottom face of t_bob
        assert!(
            (first_y - expected_first_y).abs() <= tolerance,
            "First path point y={first_y:.2} should be near t_bob bottom face \
             y={expected_first_y:.2} (tolerance {tolerance:.0} px). \
             Got cycle-edge routing instead of Bottom->Top routing. \
             path_d = {:?}, ortho_protrusion_params = {:?}",
            bob_charlie_1_edge.path_d,
            bob_charlie_1_edge.ortho_protrusion_params,
        );

        let (_, last_y) = path_tokens
            .last()
            .and_then(|t| parse_suffixed(t, 'L').or_else(|| parse_suffixed(t, 'M')))
            .expect("Path should end with an L or M command");
        let expected_last_y = charlie_1.y; // Top face of t_charlie_1
        assert!(
            (last_y - expected_last_y).abs() <= tolerance,
            "Last path point y={last_y:.2} should be near t_charlie_1 top face \
             y={expected_last_y:.2} (tolerance {tolerance:.0} px). \
             path_d = {:?}",
            bob_charlie_1_edge.path_d,
        );
    }
}

/// For `edge_dep_alice_charlie_1`, the Z/S routing segment connecting the
/// two protrusion tips must stay within the gap between the two containers.
///
/// When the gap between the two protrusion tips is smaller than `ARC_RADIUS`,
/// the bend placement must be chosen carefully:
///
/// - **First bug**: bend placed *below* the to-protrusion tip (inside
///   `t_charlie_outer`) -- the path dipped into the container before routing
///   through the gap, creating an upward curve that contradicts the downward
///   flow direction.
///
/// - **Second bug**: bend placed *above* the from-protrusion tip (outside the
///   gap, further up than necessary) -- the path then had to loop back downward
///   to reach the from-protrusion tip, creating a visible backward movement in
///   the arrow (visual) direction.
///
/// The correct fix places the bend at the *midpoint* between the two
/// protrusion tips, keeping it strictly inside the routing gap. This ensures
/// both Leg 1 (from the to-protrusion tip) and Leg 3 (arriving at the
/// from-protrusion tip) travel in the same upward direction, matching the
/// edge's overall flow.
#[test]
fn test_edge_from_nested_routing_stays_within_gap() {
    for svg_elements in build_svg_elements_from_edge_from_node_to_nested_node() {
        let charlie_outer = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_charlie_outer")
            .expect("Expected t_charlie_outer in svg_node_infos");

        let alice_charlie_1_edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| {
                e.from_node_id.as_str() == "t_alice" && e.to_node_id.as_str() == "t_charlie_1"
            })
            .expect("Expected edge from t_alice to t_charlie_1");

        let charlie_outer_top_y = charlie_outer.y;

        // The path is built in SVG order from the from-node (alice, at the
        // top) to the to-node (charlie_1, at the bottom). All coordinates
        // between the first (alice contact y) and the last (charlie_1 contact
        // y) are the routing segment.
        let all_coords = parse_path_endpoints(&alice_charlie_1_edge.path_d);

        // The from-protrusion tip is the second coordinate (just after the
        // alice contact point). Its y is the upper bound of the routing gap --
        // no later point should overshoot above it.
        let from_protrusion_tip_y = all_coords.get(1).map(|&(_, y)| y).unwrap_or(0.0);

        // Skip the first (alice contact) and last (charlie_1 contact).
        let intermediate_coords = all_coords
            .iter()
            .skip(1)
            .take(all_coords.len().saturating_sub(2));

        for &(x, y) in intermediate_coords {
            // No intermediate point should fall below charlie_outer's top --
            // that means the Z/S dipped into the destination container.
            assert!(
                y <= charlie_outer_top_y + 0.5,
                "Intermediate routing coordinate ({x:.3}, {y:.3}) is below \
                 t_charlie_outer's top boundary (y={charlie_outer_top_y:.3}). \
                 The Z/S bend was placed inside the destination container. \
                 path_d = {:?}",
                alice_charlie_1_edge.path_d,
            );

            // No intermediate point should overshoot above the from-protrusion
            // tip -- that means the Z/S looped backward in the visual
            // (arrow) direction, going further up than needed and then
            // reversing to reach the from-protrusion tip.
            assert!(
                y >= from_protrusion_tip_y - 0.5,
                "Intermediate routing coordinate ({x:.3}, {y:.3}) overshoots \
                 above the from-protrusion tip (y={from_protrusion_tip_y:.3}). \
                 The Z/S bend was placed outside the routing gap, causing a \
                 backward loop in the visual arrow direction. \
                 path_d = {:?}",
                alice_charlie_1_edge.path_d,
            );
        }
    }
}

/// Loads `0008_edge_from_node_to_nested_rank_1_node.yaml` and returns one
/// `SvgElements` per LOD.
fn build_svg_elements_from_edge_from_node_to_nested_rank_1_node(
) -> impl Iterator<Item = SvgElements<'static>> {
    build_svg_elements_for_diagram(INPUT_DIAGRAM_0008_EDGE_FROM_NODE_TO_NESTED_RANK_1_NODE)
}

/// In `0008`, `t_charlie_3` is at rank 1 inside `t_charlie_outer` (because
/// `edge_dep_charlie_2_charlie_3` promotes `t_charlie_3` to rank 1 within
/// that container). An edge from `t_alice` to `t_charlie_3` therefore needs
/// cross-container spacers alongside the rank-0 siblings (`t_charlie_1` and
/// `t_charlie_2`) so that the edge path routes correctly around them.
#[test]
fn test_edge_to_nested_rank_1_node_has_cross_container_spacers() {
    for svg_elements in build_svg_elements_from_edge_from_node_to_nested_rank_1_node() {
        let alice_charlie_3_edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| {
                e.from_node_id.as_str() == "t_alice" && e.to_node_id.as_str() == "t_charlie_3"
            })
            .expect("Expected edge from t_alice to t_charlie_3");

        assert!(
            !alice_charlie_3_edge
                .ortho_protrusion_params
                .spacer_protrusions
                .is_empty(),
            "Expected cross-container spacer protrusions for edge \
             t_alice -> t_charlie_3 (t_charlie_3 is at rank 1 inside \
             t_charlie_outer; rank-0 siblings t_charlie_1 and t_charlie_2 \
             should produce spacers so the edge routes around them)",
        );
    }
}

/// In `0008`, the `t_alice -> t_charlie_3` edge has one cross-container
/// spacer inside `t_charlie_outer` that routes around the rank-0 siblings.
/// The to_protrusion from `t_charlie_3`'s Top face should be small enough
/// to only reach the spacer exit (inside `t_charlie_outer`), NOT overshoot
/// all the way to `t_charlie_outer`'s top boundary. Overshooting causes the
/// path to re-enter the container from the outside and produces a zigzag.
#[test]
fn test_edge_to_nested_rank_1_node_to_protrusion_stays_within_container() {
    for svg_elements in build_svg_elements_from_edge_from_node_to_nested_rank_1_node() {
        let charlie_outer = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_charlie_outer")
            .expect("Expected t_charlie_outer");
        let charlie_3 = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_charlie_3")
            .expect("Expected t_charlie_3");
        let alice_charlie_3_edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| {
                e.from_node_id.as_str() == "t_alice" && e.to_node_id.as_str() == "t_charlie_3"
            })
            .expect("Expected edge from t_alice to t_charlie_3");

        // The maximum sensible to_protrusion is the distance from
        // t_charlie_3's Top face (y = charlie_3.y) to the first
        // spacer exit, which is well inside t_charlie_outer. The
        // container-exit distance (charlie_3.y - charlie_outer.y)
        // is the pathological over-shoot value that causes the zigzag.
        let container_exit_distance = charlie_3.y - charlie_outer.y;
        let to_protrusion = alice_charlie_3_edge.ortho_protrusion_params.to_protrusion;
        assert!(
            to_protrusion < container_exit_distance,
            "to_protrusion ({to_protrusion:.2}) should be less than the \
             container-exit distance ({container_exit_distance:.2} = \
             charlie_3.y {:.2} - charlie_outer.y {:.2}). \
             A to_protrusion equal to the container-exit distance means the \
             path overshoots t_charlie_outer's top boundary, re-enters the \
             container from outside, and produces a zigzag.",
            charlie_3.y,
            charlie_outer.y,
        );
    }
}

/// In `0008`, `t_charlie_3` is at rank 1 inside `t_charlie_outer`, and the
/// only lower rank (rank 0) contains two siblings: `t_charlie_1` and
/// `t_charlie_2`. The edge from `t_alice` to `t_charlie_3` should route
/// around the rank-0 row as a whole -- one spacer per rank group is
/// sufficient, so exactly one spacer protrusion should be recorded.
#[test]
fn test_edge_to_nested_rank_1_node_has_exactly_one_cross_container_spacer() {
    for svg_elements in build_svg_elements_from_edge_from_node_to_nested_rank_1_node() {
        let alice_charlie_3_edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| {
                e.from_node_id.as_str() == "t_alice" && e.to_node_id.as_str() == "t_charlie_3"
            })
            .expect("Expected edge from t_alice to t_charlie_3");

        let spacer_count = alice_charlie_3_edge
            .ortho_protrusion_params
            .spacer_protrusions
            .len();
        assert_eq!(
            spacer_count,
            1,
            "Expected exactly one cross-container spacer protrusion for edge \
             t_alice -> t_charlie_3. Both rank-0 siblings t_charlie_1 and \
             t_charlie_2 belong to the same rank group and should share one \
             spacer. Got {spacer_count} spacer(s): {:?}",
            alice_charlie_3_edge
                .ortho_protrusion_params
                .spacer_protrusions,
        );
    }
}

// === Edge to nested higher-rank node with adjacent divergent ancestors (0029)
// === //

/// Loads `0029_nested_edge_overlap_with_different_rank_nested_edge.yaml` and
/// returns one `SvgElements` per LOD.
fn build_svg_elements_from_nested_edge_overlap_different_rank(
) -> impl Iterator<Item = SvgElements<'static>> {
    build_svg_elements_for_diagram(
        INPUT_DIAGRAM_0029_NESTED_EDGE_OVERLAP_WITH_DIFFERENT_RANK_NESTED_EDGE,
    )
}

/// In `0029` (`rank_dir: left_to_right`), `edge_dep_b_00_c_01` runs from
/// `t_b_00` to `t_c_01`, which is at internal rank 1 inside `t_c_0`. Even
/// though the divergent ancestors `t_b_0` and `t_c_0` are *adjacent* siblings
/// at the root level, the edge must still route around the rank-0 sibling
/// `t_c_00` inside `t_c_0`, so it needs a cross-container spacer.
#[test]
fn test_edge_to_nested_higher_rank_node_with_adjacent_divergent_ancestors_has_spacer() {
    for svg_elements in build_svg_elements_from_nested_edge_overlap_different_rank() {
        let edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.from_node_id.as_str() == "t_b_00" && e.to_node_id.as_str() == "t_c_01")
            .expect("Expected edge from t_b_00 to t_c_01");

        assert!(
            !edge.ortho_protrusion_params.spacer_protrusions.is_empty(),
            "Expected a cross-container spacer for edge t_b_00 -> t_c_01 \
             (t_c_01 is at rank 1 inside t_c_0; the edge must route around the \
             rank-0 sibling t_c_00 even though t_b_0 and t_c_0 are adjacent \
             root-level siblings): spacer_protrusions = {:?}",
            edge.ortho_protrusion_params.spacer_protrusions,
        );
    }
}

/// In `0029`, `edge_dep_a_00_c_00` runs from `t_a_00` to `t_c_00` (rank 0
/// inside `t_c_0`). Its `to_protrusion` must stay within the `t_a_00` ->
/// `t_c_00` gap.
///
/// Previously the sibling edge `t_b_00 -> t_c_01` (lacking a spacer) was given
/// a container-deep `to_protrusion`, and because both to-endpoints share the
/// same divergent-ancestor sibling row, the staggering inflated this edge's
/// `to_protrusion` so the path overshot back past `t_a_00` -- a backward spike.
#[test]
fn test_edge_to_nested_rank_0_node_protrusion_not_inflated_by_sibling_edge() {
    for svg_elements in build_svg_elements_from_nested_edge_overlap_different_rank() {
        let node_info = |node_id: &str| {
            svg_elements
                .svg_node_infos
                .iter()
                .find(|n| n.node_id.as_str() == node_id)
                .unwrap_or_else(|| panic!("Expected {node_id} in svg_node_infos"))
        };
        let t_a_00 = node_info("t_a_00");
        let t_c_00 = node_info("t_c_00");

        let edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.from_node_id.as_str() == "t_a_00" && e.to_node_id.as_str() == "t_c_00")
            .expect("Expected edge from t_a_00 to t_c_00");

        // rank_dir is left_to_right, so the from-to gap is along x: from the
        // from-node's right face to the to-node's left face.
        let node_gap = t_c_00.x - (t_a_00.x + t_a_00.width);
        let to_protrusion = edge.ortho_protrusion_params.to_protrusion;
        assert!(
            to_protrusion <= node_gap + 0.5,
            "edge t_a_00 -> t_c_00 to_protrusion ({to_protrusion:.2}) should not \
             exceed the node-to-node gap ({node_gap:.2}). A larger value means \
             the to-protrusion overshoots back past t_a_00 (backward spike), \
             caused by the sibling edge to t_c_01 lacking a cross-container \
             spacer. path_d = {:?}",
            edge.path_d,
        );
    }
}

// === Edge `from` protrusion not inflated by a wide same-rank sibling (0030)
// === //

/// Loads
/// `0030_nested_edge_overlap_with_different_rank_nested_edge_with_node_desc.
/// yaml` and returns one `SvgElements` per LOD.
fn build_svg_elements_from_nested_edge_overlap_different_rank_with_node_desc(
) -> impl Iterator<Item = SvgElements<'static>> {
    build_svg_elements_for_diagram(
        INPUT_DIAGRAM_0030_NESTED_EDGE_OVERLAP_WITH_DIFFERENT_RANK_NESTED_EDGE_WITH_NODE_DESC,
    )
}

/// In `0030` (`rank_dir: left_to_right`), `t_a_00` carries a long node
/// description, widening its container `t_a_0`. The edge `edge_dep_a_00_c_01`
/// (from `t_a_00` to `t_c_01`) shares a divergent-ancestor sibling row with
/// `edge_dep_b_00_c_01` (from the narrow `t_b_0`'s `t_b_00`).
///
/// Previously the row-group staggering used a single group-wide `max` of the
/// per-endpoint clearances as the base. That clearance is a *relative* delta to
/// the shared sibling-row extreme, so applying `t_b_00`'s large delta to
/// `t_a_00` (whose face is already near the extreme) drove `t_a_00`'s
/// `from_protrusion` far past the destination container `t_c_0`.
///
/// The `from` protrusion must not extend beyond the gap between this edge's
/// divergent ancestors `t_a_0` and `t_c_0`.
#[test]
fn test_edge_from_protrusion_not_inflated_by_wide_same_rank_sibling() {
    for svg_elements in build_svg_elements_from_nested_edge_overlap_different_rank_with_node_desc()
    {
        let node_info = |node_id: &str| {
            svg_elements
                .svg_node_infos
                .iter()
                .find(|n| n.node_id.as_str() == node_id)
                .unwrap_or_else(|| panic!("Expected {node_id} in svg_node_infos"))
        };
        let t_a_00 = node_info("t_a_00");
        let t_c_0 = node_info("t_c_0");

        let edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.from_node_id.as_str() == "t_a_00" && e.to_node_id.as_str() == "t_c_01")
            .expect("Expected edge from t_a_00 to t_c_01");

        // rank_dir is left_to_right, so the from protrusion extends rightward
        // from t_a_00's right face. It must not reach past the near (left) face
        // of the destination divergent ancestor t_c_0.
        let from_face_x = t_a_00.x + t_a_00.width;
        let divergent_ancestor_gap = t_c_0.x - from_face_x;
        let from_protrusion = edge.ortho_protrusion_params.from_protrusion;
        assert!(
            from_protrusion <= divergent_ancestor_gap + 0.5,
            "edge t_a_00 -> t_c_01 from_protrusion ({from_protrusion:.2}) should \
             not exceed the gap between divergent ancestors t_a_0 and t_c_0 \
             ({divergent_ancestor_gap:.2}). A larger value means the wide \
             same-rank sibling t_a_0 (widened by t_a_00's node description) \
             inflated this edge's protrusion via row-group staggering. \
             path_d = {:?}",
            edge.path_d,
        );
    }
}

/// In `0030`, both `edge_dep_a_00_c_01` and `edge_dep_b_00_c_01` enter
/// `t_c_01` (rank 1 inside `t_c_0`) via cross-container spacers that exit at
/// the same coordinate (the spacers are stacked in the same rank container
/// around `t_c_00`). Their `to` and last-spacer `exit` protrusions otherwise
/// floor to (near-)identical values, so their vertical approach legs in the
/// spacer-to-node gap overlap.
///
/// `protrusions_separate_spacer_approach_channels` assigns each edge a distinct
/// leg coordinate in the gap, so the two approach legs must be clearly
/// separated (and neither overshoots the gap between the spacer exit and the
/// to-node).
#[test]
fn test_spacer_edges_into_same_node_have_separated_approach_legs() {
    for svg_elements in build_svg_elements_from_nested_edge_overlap_different_rank_with_node_desc()
    {
        let edge = |from: &str, to: &str| {
            svg_elements
                .svg_edge_infos
                .iter()
                .find(|e| e.from_node_id.as_str() == from && e.to_node_id.as_str() == to)
                .unwrap_or_else(|| panic!("Expected edge from {from} to {to}"))
        };
        let edge_a = edge("t_a_00", "t_c_01");
        let edge_b = edge("t_b_00", "t_c_01");

        // Both edges route into t_c_01 via a cross-container spacer.
        assert!(
            !edge_a.ortho_protrusion_params.spacer_protrusions.is_empty()
                && !edge_b.ortho_protrusion_params.spacer_protrusions.is_empty(),
            "Expected both edges into t_c_01 to have cross-container spacers",
        );

        // rank_dir is left_to_right and both enter t_c_01's left face, so the
        // approach leg sits at `t_c_01.x - to_protrusion`. Distinct
        // to_protrusions => distinct legs. They must differ by a visible margin.
        let to_protrusion_a = edge_a.ortho_protrusion_params.to_protrusion;
        let to_protrusion_b = edge_b.ortho_protrusion_params.to_protrusion;
        assert!(
            (to_protrusion_a - to_protrusion_b).abs() >= 5.0,
            "edges t_a_00 -> t_c_01 and t_b_00 -> t_c_01 should have separated \
             approach legs, but their to_protrusions are too close \
             (a = {to_protrusion_a:.2}, b = {to_protrusion_b:.2}). Their vertical \
             approach legs in the t_c_00 -> t_c_01 gap overlap. \
             path_a = {:?}, path_b = {:?}",
            edge_a.path_d,
            edge_b.path_d,
        );
    }
}

/// Edges to `t_charlie_1` (rank 0 in `t_charlie_outer`) should have no
/// cross-container spacers, even in the presence of a rank-1 sibling
/// (`t_charlie_3`).
#[test]
fn test_edge_to_nested_rank_0_node_has_no_spacers_in_complex_diagram() {
    for svg_elements in build_svg_elements_from_edge_from_node_to_nested_rank_1_node() {
        // alice -> charlie_1 edge
        let alice_charlie_1_edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| {
                e.from_node_id.as_str() == "t_alice" && e.to_node_id.as_str() == "t_charlie_1"
            })
            .expect("Expected edge from t_alice to t_charlie_1");

        assert!(
            alice_charlie_1_edge
                .ortho_protrusion_params
                .spacer_protrusions
                .is_empty(),
            "Expected no spacer protrusions for edge t_alice -> t_charlie_1 \
             in the 0008 diagram (t_charlie_1 is at rank 0): \
             spacer_protrusions = {:?}",
            alice_charlie_1_edge
                .ortho_protrusion_params
                .spacer_protrusions,
        );

        // bob -> charlie_1 edge: also no spacers
        let bob_charlie_1_edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.from_node_id.as_str() == "t_bob" && e.to_node_id.as_str() == "t_charlie_1")
            .expect("Expected edge from t_bob to t_charlie_1");

        assert!(
            bob_charlie_1_edge
                .ortho_protrusion_params
                .spacer_protrusions
                .is_empty(),
            "Expected no spacer protrusions for edge t_bob -> t_charlie_1 \
             in the 0008 diagram (t_charlie_1 is at rank 0): \
             spacer_protrusions = {:?}",
            bob_charlie_1_edge
                .ortho_protrusion_params
                .spacer_protrusions,
        );
    }
}

/// The edge from `t_alice_inner` to `t_charlie_inner` in the doubly-nested
/// diagram must route orthogonally without entering `t_charlie_outer`'s
/// interior.
///
/// Two bugs could cause intermediate routing coordinates to fall below
/// `t_charlie_outer`'s top:
///
/// 1. The `connect_waypoints` collinear check using `dot_p.abs() > 0.95`
///    incorrectly treated the nearly anti-collinear displacement between the
///    two protrusion tips as "straight", drawing a diagonal line instead of an
///    orthogonal Z/S bend.
///
/// 2. The from-protrusion (73.44 px) plus the to-protrusion (110.0 px) summed
///    to 183.44 px, exceeding the node-to-node gap (153 px). The
///    from-protrusion tip was placed inside `t_charlie_outer` (at y=245.44),
///    below the to-protrusion tip (at y=215.0).
///
/// After the fix the from-protrusion is capped to 43 px (= 153 - 110), so
/// both tips meet at `t_charlie_outer`'s top boundary (y=215). The V-spike
/// guard in `connect_waypoints` (see
/// `test_nested_x2_node_edge_routing_no_upward_detour`) then replaces the Z/S
/// U-bend between the tips with a straight horizontal line, so no intermediate
/// coordinate falls below `t_charlie_outer.y`.
#[test]
fn test_nested_x2_node_edge_routing_stays_above_charlie_outer() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0002_NESTED_NODE_EDGE_PROTRUSION)
    {
        let charlie_outer = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_charlie_outer")
            .expect("Expected t_charlie_outer in svg_node_infos");

        let alice_inner_charlie_inner_edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| {
                e.from_node_id.as_str() == "t_alice_inner"
                    && e.to_node_id.as_str() == "t_charlie_inner"
            })
            .expect("Expected edge from t_alice_inner to t_charlie_inner");

        let charlie_outer_top_y = charlie_outer.y;

        // The path is built in SVG order from the from-node (t_alice_inner,
        // at the top) to the to-node (t_charlie_inner, at the bottom). The
        // first coordinate is t_alice_inner's contact point (above all
        // containers) and the last is t_charlie_inner's contact point (inside
        // t_charlie_outer).
        //
        // All *intermediate* coordinates represent the routing segment
        // connecting the two protrusion tips. None of them should fall below
        // t_charlie_outer's top, which would indicate the Z/S bend dipped
        // into the destination container.
        let all_coords = parse_path_endpoints(&alice_inner_charlie_inner_edge.path_d);

        let intermediate_coords = all_coords
            .iter()
            .skip(1)
            .take(all_coords.len().saturating_sub(2));

        for &(x, y) in intermediate_coords {
            assert!(
                y <= charlie_outer_top_y + 0.5,
                "Intermediate routing coordinate ({x:.3}, {y:.3}) is below \
                 t_charlie_outer's top boundary (y={charlie_outer_top_y:.3}). \
                 The Z/S bend dipped into the destination container. \
                 path_d = {:?}",
                alice_inner_charlie_inner_edge.path_d,
            );
        }
    }
}

/// The edge from `t_alice_inner` to `t_charlie_inner` in the doubly-nested
/// diagram must not create a V-spike at the `t_charlie_outer` boundary.
///
/// After `from_protrusion_capped` places both protrusion tips at y=215
/// (t_charlie_outer's top), the naive Z/S U-bend would route: upward from
/// the to-tip at (97,215) to y=211, across to x=88.5, then back down to
/// the from-tip at (88.5,215). The `is_same_axis` return leg then
/// immediately travels upward to (88.5,172), reversing direction and
/// creating an incoherent V-spike.
///
/// The fix in `connect_waypoints` detects vertical tips at the same Y with
/// opposite departure directions and draws a straight horizontal line instead.
/// No intermediate coordinate should appear above `t_charlie_outer`'s top
/// (y < charlie_outer.y - 0.5 would indicate an upward detour).
#[test]
fn test_nested_x2_node_edge_routing_no_upward_detour() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0002_NESTED_NODE_EDGE_PROTRUSION)
    {
        let alice_inner_charlie_inner_edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| {
                e.from_node_id.as_str() == "t_alice_inner"
                    && e.to_node_id.as_str() == "t_charlie_inner"
            })
            .expect("Expected edge from t_alice_inner to t_charlie_inner");

        // The path is built in SVG order: from-node first, to-node last.
        // The second coordinate is the from-protrusion tip -- the highest
        // (smallest y) point the routing reaches before descending toward
        // the to-node.
        //
        // After the `rank_gap_px` cap fix, the routing now correctly enters
        // the inter-rank gap between the containers. The fix ensures that
        // no later coordinate rises back above (has a smaller y than) the
        // from-protrusion tip, which would indicate a V-spike where the Z/S
        // bend looped backward in the visual arrow direction.
        let all_coords = parse_path_endpoints(&alice_inner_charlie_inner_edge.path_d);
        let from_protrusion_tip_y = all_coords.get(1).map(|&(_, y)| y).unwrap_or(0.0);

        let intermediate_coords = all_coords.iter().skip(2);

        for &(x, y) in intermediate_coords {
            assert!(
                y >= from_protrusion_tip_y - 0.5,
                "Routing coordinate ({x:.3}, {y:.3}) is above \
                 the from-protrusion tip (y={from_protrusion_tip_y:.3}). \
                 The Z/S bend was placed outside the routing gap, causing a \
                 backward loop in the visual arrow direction. \
                 path_d = {:?}",
                alice_inner_charlie_inner_edge.path_d,
            );
        }
    }
}

/// In `0017`, `edge_dep_alice_charlie__0` connects `t_alice` (nested in
/// `t_alice_outer`) to `t_charlie` (nested in `t_charlie_outer`). After the
/// `rank_gap_px` cap fix, the protrusion tips must lie within the inter-rank
/// gap -- not at the boundary of `t_charlie_outer`.
///
/// Before the fix:
/// - `from_protrusion` was computed from the full face-to-face distance (94
///   px), giving `max_protrusion = 94 * 0.48 = 45.12`.
/// - Combined with `to_protrusion = 55` (to clear `t_charlie_outer`), the sum
///   100.12 > 94 caused capping to 39, landing both tips at `t_charlie_outer.y
///   = 215`.
/// - The V-spike guard fired and drew a straight horizontal line at y = 215
///   with no arc-rounded corners.
///
/// After the fix:
/// - `rank_gap_px` is capped at `t_charlie_outer.y - t_alice.bottom_y = 39`,
///   giving `max_protrusion = 39 * 0.48 ≈ 18.72`.
/// - `from_protrusion ≈ 18.72` places the from-protrusion tip inside the
///   inter-rank gap (above `t_charlie_outer.y`), so both tips are at different
///   y-coordinates.
/// - A proper Z/S bend with arc-rounded corners is drawn in the gap.
#[test]
fn test_0017_edge_inner_to_inner_routing_in_inter_rank_gap() {
    for svg_elements in build_svg_elements_for_diagram(INPUT_DIAGRAM_0017_EDGE_INNER_TO_INNER) {
        let charlie_outer = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_charlie_outer")
            .expect("Expected t_charlie_outer in svg_node_infos");

        let alice_charlie_edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.from_node_id.as_str() == "t_alice" && e.to_node_id.as_str() == "t_charlie")
            .expect("Expected edge from t_alice to t_charlie");

        let charlie_outer_top_y = charlie_outer.y;
        let all_coords = parse_path_endpoints(&alice_charlie_edge.path_d);

        // The path runs from-node (t_alice, top) to to-node (t_charlie,
        // bottom). The second coordinate is the from-protrusion tip (top of
        // the routing gap), which must sit above t_charlie_outer's top (smaller
        // y) -- i.e. inside the inter-rank gap -- not at the boundary. Before
        // the rank_gap_px fix both tips landed at charlie_outer.y, suppressing
        // the Z/S bend.
        let from_protrusion_tip_y = all_coords.get(1).map(|&(_, y)| y).unwrap_or(0.0);

        assert!(
            from_protrusion_tip_y < charlie_outer_top_y - 0.5,
            "from_protrusion_tip_y ({from_protrusion_tip_y:.3}) should be above \
             t_charlie_outer's top boundary (y={charlie_outer_top_y:.3}), i.e. in \
             the inter-rank gap. \
             path_d = {:?}",
            alice_charlie_edge.path_d,
        );

        // All intermediate routing coordinates must stay above t_charlie_outer
        // (no dip into the container) and not rise back above the
        // from-protrusion tip (no backward loop).
        let intermediate_coords = all_coords
            .iter()
            .skip(1)
            .take(all_coords.len().saturating_sub(2));

        for &(x, y) in intermediate_coords {
            assert!(
                y <= charlie_outer_top_y + 0.5,
                "Intermediate routing coordinate ({x:.3}, {y:.3}) is below \
                 t_charlie_outer's top (y={charlie_outer_top_y:.3}): the Z/S bend \
                 dipped into the destination container. \
                 path_d = {:?}",
                alice_charlie_edge.path_d,
            );
            assert!(
                y >= from_protrusion_tip_y - 0.5,
                "Routing coordinate ({x:.3}, {y:.3}) is above the \
                 from-protrusion tip (y={from_protrusion_tip_y:.3}): the Z/S bend \
                 looped backward past the routing gap. \
                 path_d = {:?}",
                alice_charlie_edge.path_d,
            );
        }
    }
}

/// In `0008`, edges from `t_bob` to `t_charlie_1` (rank 0 inside
/// `t_charlie_outer`) should use normal Bottom -> Top face routing, not
/// cycle-edge routing, even though both nodes have local rank 0 in their
/// respective parent contexts.
#[test]
fn test_edge_from_toplevel_to_nested_rank_0_node_uses_normal_routing_complex_diagram() {
    for svg_elements in build_svg_elements_from_edge_from_node_to_nested_rank_1_node() {
        let bob = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_bob")
            .expect("Expected t_bob");
        let charlie_1 = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_charlie_1")
            .expect("Expected t_charlie_1");

        let bob_charlie_1_edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.from_node_id.as_str() == "t_bob" && e.to_node_id.as_str() == "t_charlie_1")
            .expect("Expected edge from t_bob to t_charlie_1");

        let path_tokens: Vec<&str> = bob_charlie_1_edge.path_d.split_whitespace().collect();
        assert!(
            !path_tokens.is_empty(),
            "Expected non-empty path for edge t_bob -> t_charlie_1"
        );

        let parse_suffixed = |s: &str, prefix: char| -> Option<(f32, f32)> {
            let s = s.strip_prefix(prefix)?;
            let (x_str, y_str) = s.split_once(',')?;
            Some((x_str.parse().ok()?, y_str.parse().ok()?))
        };

        let tolerance = 20.0_f32;

        // Path starts at from-node (t_bob bottom face).
        let (_, first_y) = path_tokens
            .first()
            .and_then(|t| parse_suffixed(t, 'M'))
            .expect("Path should start with M command (e.g. M80,210)");
        let expected_first_y = bob.y + bob.height_collapsed;
        assert!(
            (first_y - expected_first_y).abs() <= tolerance,
            "First path point y={first_y:.2} should be near t_bob bottom face \
             y={expected_first_y:.2} (tolerance {tolerance:.0} px). \
             Cycle-edge routing produces a different starting y. \
             path_d = {:?}",
            bob_charlie_1_edge.path_d,
        );

        // Path ends at to-node (t_charlie_1 top face).
        let (_, last_y) = path_tokens
            .last()
            .and_then(|t| parse_suffixed(t, 'L').or_else(|| parse_suffixed(t, 'M')))
            .expect("Path should end with an L or M command");
        let expected_last_y = charlie_1.y;
        assert!(
            (last_y - expected_last_y).abs() <= tolerance,
            "Last path point y={last_y:.2} should be near t_charlie_1 top face \
             y={expected_last_y:.2} (tolerance {tolerance:.0} px). \
             path_d = {:?}",
            bob_charlie_1_edge.path_d,
        );
    }
}

// === Edge description / label tests === //

/// Returns `SvgElements` for the edge-with-description fixture (0009).
///
/// The fixture has two nodes (`t_a`, `t_b`) connected by a single sequence
/// edge (`edge_ab__0`) and an entity_desc for that edge.
fn build_svg_elements_from_edge_with_description() -> impl Iterator<Item = SvgElements<'static>> {
    build_svg_elements_for_diagram(INPUT_DIAGRAM_0009_EDGE_WITH_DESCRIPTION)
}

/// An edge with a description in `edge_descs` must produce a non-empty
/// `edge_label_infos` entry in `SvgElements`.
///
/// The edge ID used as the `edge_descs` key follows the generated format
/// `"{edge_group_id}__{edge_index}"`, e.g. `edge_ab__0` for index 0 of the
/// `edge_ab` group.
#[test]
fn test_edge_description_produces_edge_label_infos() {
    for svg_elements in build_svg_elements_from_edge_with_description() {
        assert!(
            !svg_elements.edge_label_infos.is_empty(),
            "Expected edge_label_infos to be non-empty when edge_descs contains the edge ID"
        );

        let label_info = svg_elements
            .edge_label_infos
            .iter()
            .find(|info| info.edge_id.as_str() == "edge_ab__0")
            .expect("Expected an edge_label_info entry for edge_ab__0");

        // Both the from-endpoint (t_a bottom face) and to-endpoint (t_b top
        // face) should have label slots with text spans.
        let from_label = label_info
            .from_label
            .as_ref()
            .expect("Expected from_label to be present for edge_ab__0");
        assert!(
            !from_label.text_spans.is_empty(),
            "Expected from_label.text_spans to be non-empty for edge_ab__0"
        );

        let to_label = label_info
            .to_label
            .as_ref()
            .expect("Expected to_label to be present for edge_ab__0");
        assert!(
            !to_label.text_spans.is_empty(),
            "Expected to_label.text_spans to be non-empty for edge_ab__0"
        );
    }
}

fn build_svg_elements_from_self_loop_edge_with_description(
) -> impl Iterator<Item = SvgElements<'static>> {
    build_svg_elements_for_diagram(INPUT_DIAGRAM_0010_SELF_LOOP_EDGE_WITH_DESCRIPTION)
}

fn build_svg_elements_from_contained_edge_with_description(
) -> impl Iterator<Item = SvgElements<'static>> {
    build_svg_elements_for_diagram(INPUT_DIAGRAM_0011_CONTAINED_EDGE_WITH_DESCRIPTION)
}

/// The text in the edge label spans must match the description from
/// `edge_descs`.
#[test]
fn test_edge_description_text_matches_edge_descs() {
    let expected_text = "Alpha to Beta connection";

    for svg_elements in build_svg_elements_from_edge_with_description() {
        let label_info = svg_elements
            .edge_label_infos
            .iter()
            .find(|info| info.edge_id.as_str() == "edge_ab__0")
            .expect("Expected an edge_label_info entry for edge_ab__0");

        // Collect all span text from both endpoints (both show the same
        // description text).
        let from_texts: Vec<&str> = label_info
            .from_label
            .as_ref()
            .map(|l| l.text_spans.iter().map(|s| s.text.as_str()).collect())
            .unwrap_or_default();
        let combined_from = from_texts.join("");
        assert!(
            combined_from.contains(expected_text)
                || expected_text
                    .split_whitespace()
                    .all(|word| from_texts.iter().any(|t| t.contains(word))),
            "from_label spans {from_texts:?} should contain the description text '{expected_text}'"
        );

        let to_texts: Vec<&str> = label_info
            .to_label
            .as_ref()
            .map(|l| l.text_spans.iter().map(|s| s.text.as_str()).collect())
            .unwrap_or_default();
        let combined_to = to_texts.join("");
        assert!(
            combined_to.contains(expected_text)
                || expected_text
                    .split_whitespace()
                    .all(|word| to_texts.iter().any(|t| t.contains(word))),
            "to_label spans {to_texts:?} should contain the description text '{expected_text}'"
        );
    }
}

/// A self-loop edge with a description must produce a non-empty
/// `edge_label_infos` entry in `SvgElements` with a `from_label` slot.
///
/// Since `from == to`, only a single label slot is used (`from_label`).
/// `to_label` is expected to be `None`.
#[test]
fn test_self_loop_edge_description_produces_from_label() {
    for svg_elements in build_svg_elements_from_self_loop_edge_with_description() {
        assert!(
            !svg_elements.edge_label_infos.is_empty(),
            "Expected edge_label_infos to be non-empty for a self-loop with a description"
        );

        let label_info = svg_elements
            .edge_label_infos
            .iter()
            .find(|info| info.edge_id.as_str() == "edge_self__0")
            .expect("Expected an edge_label_info entry for edge_self__0");

        // Self-loop: from_label must be present with text spans.
        let from_label = label_info
            .from_label
            .as_ref()
            .expect("Expected from_label to be present for a self-loop edge");
        assert!(
            !from_label.text_spans.is_empty(),
            "Expected from_label.text_spans to be non-empty for self-loop edge_self__0"
        );

        // Self-loop: to_label is None because from == to and one slot suffices.
        assert!(
            label_info.to_label.is_none(),
            "Expected to_label to be None for a self-loop edge (from == to)"
        );
    }
}

/// A contained edge (parent -> child where child is inside parent) with a
/// description must produce a non-empty `edge_label_infos` entry in
/// `SvgElements` with both `from_label` and `to_label` slots populated.
#[test]
fn test_contained_edge_description_produces_both_labels() {
    for svg_elements in build_svg_elements_from_contained_edge_with_description() {
        assert!(
            !svg_elements.edge_label_infos.is_empty(),
            "Expected edge_label_infos to be non-empty for a contained edge with a description"
        );

        let label_info = svg_elements
            .edge_label_infos
            .iter()
            .find(|info| info.edge_id.as_str() == "edge_contained__0")
            .expect("Expected an edge_label_info entry for edge_contained__0");

        // Contained edge: from_label (on parent node) must have text spans.
        let from_label = label_info
            .from_label
            .as_ref()
            .expect("Expected from_label to be present for contained edge_contained__0");
        assert!(
            !from_label.text_spans.is_empty(),
            "Expected from_label.text_spans to be non-empty for contained edge_contained__0"
        );

        // Contained edge: to_label (on child node) must also have text spans.
        let to_label = label_info
            .to_label
            .as_ref()
            .expect("Expected to_label to be present for contained edge_contained__0");
        assert!(
            !to_label.text_spans.is_empty(),
            "Expected to_label.text_spans to be non-empty for contained edge_contained__0"
        );
    }
}

// === Edge from nested node to outer node, cyclic (0012) === //

/// Builds `SvgElements` from the 0012 fixture: symmetric edge between
/// `t_alice` (nested in `t_alice_outer`) and `t_bob` (root-level node).
fn build_svg_elements_from_edge_from_nested_node_to_outer_node_cyclic(
) -> impl Iterator<Item = SvgElements<'static>> {
    build_svg_elements_for_diagram(INPUT_DIAGRAM_0012_EDGE_FROM_NESTED_NODE_TO_OUTER_NODE_CYCLIC)
}

/// For the symmetric edge between `t_alice` (nested in `t_alice_outer`) and
/// `t_bob` (root level), the from-protrusion of the edge from `t_alice` to
/// `t_bob` must be large enough to clear `t_alice_outer`'s right boundary.
///
/// Before the fix, the divergent ancestors of `t_alice` and `t_bob` --
/// `t_alice_outer` (index 0) and `t_bob` (index 1) -- are adjacent siblings
/// at the root level. The edge was incorrectly classified as a clockwise cycle
/// edge because `t_alice` and `t_bob` are at the same LCA rank (both 0) and
/// the `nodes_adjacent_siblings_are` check (which only compares the endpoint
/// nodes' own sibling relationship) returned `false` for nodes at different
/// nesting depths.
///
/// The fix introduces `nodes_divergent_ancestors_adjacent_siblings_are` and
/// uses it in the `is_cycle_edge` check so that the edge is correctly
/// classified as a forward edge. As a result:
///
/// * `from_protrusion` is at least `t_alice_outer.right - t_alice.right`,
///   exiting `t_alice_outer` before connecting to `t_bob`.
/// * `to_protrusion` is 0 (no container to exit for `t_bob`).
/// * The path approaches `t_alice`'s right face from the right, not from the
///   left.
#[test]
fn test_edge_from_nested_to_outer_adjacent_divergent_ancestors_uses_forward_routing() {
    for svg_elements in build_svg_elements_from_edge_from_nested_node_to_outer_node_cyclic() {
        let alice_outer = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_alice_outer")
            .expect("Expected t_alice_outer in svg_node_infos");
        let alice = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_alice")
            .expect("Expected t_alice in svg_node_infos");

        let alice_right = alice.envelope_x + alice.envelope_width;
        let alice_outer_right = alice_outer.envelope_x + alice_outer.envelope_width;
        let expected_min_from_protrusion = (alice_outer_right - alice_right).max(0.0);

        // Edge from t_alice (nested) to t_bob (root): from_protrusion must
        // clear t_alice_outer's right boundary.
        let alice_bob_edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.from_node_id.as_str() == "t_alice" && e.to_node_id.as_str() == "t_bob")
            .expect("Expected edge from t_alice to t_bob");

        assert!(
            alice_bob_edge.ortho_protrusion_params.from_protrusion >= expected_min_from_protrusion,
            "edge t_alice -> t_bob from_protrusion {:.2} should be >= {:.2} \
             (t_alice_outer right {:.2} - t_alice right {:.2}) to clear t_alice_outer. \
             Path may be routing clockwise instead of forward. path_d = {:?}",
            alice_bob_edge.ortho_protrusion_params.from_protrusion,
            expected_min_from_protrusion,
            alice_outer_right,
            alice_right,
            alice_bob_edge.path_d,
        );

        // Edge from t_bob (root) to t_alice (nested): to_protrusion must
        // clear t_alice_outer's right boundary (the path arrives at t_alice
        // from the right side of t_alice_outer).
        let bob_alice_edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.from_node_id.as_str() == "t_bob" && e.to_node_id.as_str() == "t_alice")
            .expect("Expected edge from t_bob to t_alice");

        assert!(
            bob_alice_edge.ortho_protrusion_params.to_protrusion >= expected_min_from_protrusion,
            "edge t_bob -> t_alice to_protrusion {:.2} should be >= {:.2} \
                 (t_alice_outer right {:.2} - t_alice right {:.2}) to clear t_alice_outer. \
                 path_d = {:?}",
            bob_alice_edge.ortho_protrusion_params.to_protrusion,
            expected_min_from_protrusion,
            alice_outer_right,
            alice_right,
            bob_alice_edge.path_d,
        );
    }
}

// === Edge from nested node to outer node, cyclic part 2 (0013) === //

/// Builds `SvgElements` from the 0013 fixture: cyclic edges between a nested
/// node and an outer container in a diagram with three root-level siblings
/// (`t_alice_outer`, `t_bob_outer`, `t_charlie`).
fn build_svg_elements_from_edge_from_nested_node_to_outer_node_cyclic_2(
) -> impl Iterator<Item = SvgElements<'static>> {
    build_svg_elements_for_diagram(INPUT_DIAGRAM_0013_EDGE_FROM_NESTED_NODE_TO_OUTER_NODE_CYCLIC_2)
}

/// For the 0013 fixture with three root-level containers (`t_alice_outer` at
/// index 0, `t_bob_outer` at index 1, `t_charlie` at index 2), the two cyclic
/// cross-container edges must use Z/S forward routing that stays within the
/// gap between the adjacent containers -- not route all the way past
/// `t_charlie`.
///
/// Edge 1: `edge_dep_alice_bob_outer__0` (from `t_alice` nested inside
/// `t_alice_outer` to `t_bob_outer`):
///
/// * Before the fix the `from_protrusion` was computed as the distance from
///   `t_alice`'s right face all the way to `t_charlie`'s right edge (297 px),
///   because `t_charlie` was incorrectly included in the same-rank sibling
///   extreme. The `from_protrusion_capped` function then capped it to the
///   node-to-node gap (80 px), placing the protrusion tip exactly at
///   `t_bob_outer`'s left face. This made the path a degenerate L-shape with no
///   Z/S bend.
///
/// * After the fix the sibling extreme excludes `t_charlie` (which is spatially
///   beyond `t_bob_outer` in the rightward direction), so `from_protrusion`
///   equals the distance from `t_alice`'s right face to `t_alice_outer`'s right
///   edge (~56 px). The path is a proper Z/S curve that exits `t_alice_outer`
///   and enters `t_bob_outer`'s left face.
///
/// Edge 2: `edge_dep_alice_bob__0` (from `t_bob` nested inside `t_bob_outer`
/// to `t_alice_outer`):
///
/// * Before the fix the `to_protrusion` (for `t_alice_outer`'s right face) was
///   computed as the distance from `t_alice_outer`'s right edge to
///   `t_charlie`'s right edge (241 px), because `t_charlie` was included in the
///   sibling extreme. The path went all the way to x = 388 (past `t_charlie`)
///   before looping back to `t_bob`'s left face.
///
/// * After the fix `t_charlie` is excluded from the sibling extreme, so
///   `to_protrusion` is 0 (only `t_alice_outer` itself is included, and its
///   right edge equals its own right face coordinate). The path is a proper Z/S
///   curve that stays within the gap between `t_alice_outer` and `t_bob_outer`.
#[test]
fn test_adjacent_divergent_ancestor_edges_dont_route_past_charlie() {
    for svg_elements in build_svg_elements_from_edge_from_nested_node_to_outer_node_cyclic_2() {
        let alice_outer = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_alice_outer")
            .expect("Expected t_alice_outer in svg_node_infos");
        let alice = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_alice")
            .expect("Expected t_alice in svg_node_infos");
        let bob_outer = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_bob_outer")
            .expect("Expected t_bob_outer in svg_node_infos");
        let charlie = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_charlie")
            .expect("Expected t_charlie in svg_node_infos");

        let alice_right = alice.envelope_x + alice.envelope_width;
        let alice_outer_right = alice_outer.envelope_x + alice_outer.envelope_width;
        let bob_outer_right = bob_outer.envelope_x + bob_outer.envelope_width;
        let charlie_left = charlie.envelope_x;

        // === Edge 1: t_alice -> t_bob_outer === //
        //
        // `from_protrusion` must be large enough to exit `t_alice_outer` but
        // small enough that it does not reach `t_charlie`'s left boundary.
        let alice_bob_outer_edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| {
                e.from_node_id.as_str() == "t_alice" && e.to_node_id.as_str() == "t_bob_outer"
            })
            .expect("Expected edge from t_alice to t_bob_outer");

        let min_from_protrusion = (alice_outer_right - alice_right).max(0.0);
        let max_from_protrusion = charlie_left - alice_right;

        assert!(
            alice_bob_outer_edge.ortho_protrusion_params.from_protrusion >= min_from_protrusion,
            "edge t_alice -> t_bob_outer from_protrusion {:.2} should be >= {:.2} \
                 (t_alice_outer right {:.2} - t_alice right {:.2}) to exit t_alice_outer. \
                 path_d = {:?}",
            alice_bob_outer_edge.ortho_protrusion_params.from_protrusion,
            min_from_protrusion,
            alice_outer_right,
            alice_right,
            alice_bob_outer_edge.path_d,
        );

        assert!(
            alice_bob_outer_edge.ortho_protrusion_params.from_protrusion < max_from_protrusion,
            "edge t_alice -> t_bob_outer from_protrusion {:.2} should be < {:.2} \
                 (t_charlie left {:.2} - t_alice right {:.2}): path should not route \
                 past t_charlie. path_d = {:?}",
            alice_bob_outer_edge.ortho_protrusion_params.from_protrusion,
            max_from_protrusion,
            charlie_left,
            alice_right,
            alice_bob_outer_edge.path_d,
        );

        // === Edge 2: t_bob -> t_alice_outer === //
        //
        // `to_protrusion` (for `t_alice_outer`'s right face) must be small
        // enough that the path does not reach `t_charlie`'s left boundary.
        let bob_alice_outer_edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|svg_edge_info| svg_edge_info.edge_id.as_str() == "edge_dep_bob_alice_outer__0")
            .expect("Expected edge from t_bob to t_alice_outer");

        let max_to_protrusion = charlie_left - alice_outer_right;

        assert!(
            bob_alice_outer_edge.ortho_protrusion_params.to_protrusion < max_to_protrusion,
            "edge t_bob -> t_alice_outer to_protrusion {:.2} should be < {:.2} \
                 (t_charlie left {:.2} - t_alice_outer right {:.2}): path should not \
                 route past t_charlie. path_d = {:?}",
            bob_alice_outer_edge.ortho_protrusion_params.to_protrusion,
            max_to_protrusion,
            charlie_left,
            alice_outer_right,
            bob_alice_outer_edge.path_d,
        );

        // The path for edge 2 must also stay within the gap between
        // `t_alice_outer`/`t_bob_outer` and `t_charlie`. No path coordinate
        // should have an x value beyond `t_bob_outer`'s right edge.
        let path_coords = {
            let mut coords: Vec<(f32, f32)> = Vec::new();
            let path_str = bob_alice_outer_edge.path_d.as_str();
            // Parse all numeric coordinate pairs from M, L, C commands.
            let mut chars = path_str.chars().peekable();
            while let Some(ch) = chars.next() {
                if ch == 'M' || ch == 'L' {
                    let rest: String = chars
                        .by_ref()
                        .take_while(|&c| c != 'M' && c != 'L' && c != 'C')
                        .collect();
                    if let Some((x_str, y_str)) = rest.split_once(',') {
                        let x: f32 = x_str.trim().parse().unwrap_or(0.0);
                        let y: f32 = y_str
                            .trim()
                            .split_whitespace()
                            .next()
                            .unwrap_or("0")
                            .parse()
                            .unwrap_or(0.0);
                        coords.push((x, y));
                    }
                }
            }
            coords
        };
        for (x, _y) in &path_coords {
            assert!(
                *x <= bob_outer_right + 1.0,
                "edge t_bob -> t_alice_outer path has coordinate x={x:.2} which exceeds \
                     t_bob_outer's right edge {bob_outer_right:.2}. The path is routing past \
                     t_charlie. path_d = {:?}",
                bob_alice_outer_edge.path_d,
            );
        }
    }
}

// === Reversed rank direction sibling ordering (0019) === //

/// Builds `SvgElements` from the 0019 fixture, optionally substituting the
/// `rank_dir` value (the fixture declares `bottom_to_top`).
fn build_svg_elements_from_rank_dir_reversed_siblings(
    rank_dir: &str,
) -> impl Iterator<Item = SvgElements<'static>> {
    let input_diagram =
        INPUT_DIAGRAM_0019_RANK_DIR_REVERSED_SIBLINGS.replace("bottom_to_top", rank_dir);
    build_svg_elements_for_diagram(&input_diagram)
        .collect::<Vec<_>>()
        .into_iter()
}

/// Parses the last point of a kurbo-generated path `d` attribute.
///
/// Kurbo concatenates path commands with their coordinates (e.g. `L80,210`
/// rather than `L 80,210`), so the last whitespace-separated token is the
/// final command with its endpoint.
fn path_d_last_point(path_d: &str) -> Option<(f32, f32)> {
    let token = path_d.split_whitespace().last()?;
    let coords = token.trim_start_matches(|c: char| c.is_ascii_alphabetic());
    let (x_str, y_str) = coords.split_once(',')?;
    Some((x_str.parse().ok()?, y_str.parse().ok()?))
}

/// With `rank_dir: bottom_to_top`, rank-1 siblings must render left-to-right
/// in declaration order (`t_dog`, `t_cat`, `t_owner`), and the from-contact
/// points on `t_animal`'s top face must follow the same order so the edge
/// paths do not cross.
///
/// Before the fix, the `RowReverse` rank container rendered the siblings
/// right-to-left, and the negated face offsets could not fully compensate,
/// causing edge paths to cross.
#[test]
fn test_rank_dir_bottom_to_top_siblings_render_in_declaration_order() {
    for svg_elements in build_svg_elements_from_rank_dir_reversed_siblings("bottom_to_top") {
        let node_x = |node_id: &str| -> f32 {
            svg_elements
                .svg_node_infos
                .iter()
                .find(|n| n.node_id.as_str() == node_id)
                .unwrap_or_else(|| panic!("Expected {node_id} in svg_node_infos"))
                .x
        };
        let dog_x = node_x("t_dog");
        let cat_x = node_x("t_cat");
        let owner_x = node_x("t_owner");
        assert!(
            dog_x < cat_x && cat_x < owner_x,
            "Expected siblings to render left-to-right in declaration order: \
             t_dog (x={dog_x:.2}) < t_cat (x={cat_x:.2}) < t_owner (x={owner_x:.2})"
        );

        // The from-contact is the final point of each path (paths are built
        // in reverse, ending at the from-node face).
        let from_contact_x = |to_node_id: &str| -> f32 {
            let edge_info = svg_elements
                .svg_edge_infos
                .iter()
                .find(|e| {
                    e.from_node_id.as_str() == "t_animal" && e.to_node_id.as_str() == to_node_id
                })
                .unwrap_or_else(|| panic!("Expected edge from t_animal to {to_node_id}"));
            path_d_last_point(&edge_info.path_d)
                .unwrap_or_else(|| panic!("Expected parseable path_d: {:?}", edge_info.path_d))
                .0
        };
        let dog_contact_x = from_contact_x("t_dog");
        let cat_contact_x = from_contact_x("t_cat");
        let owner_contact_x = from_contact_x("t_owner");
        assert!(
            dog_contact_x < cat_contact_x && cat_contact_x < owner_contact_x,
            "Expected from-contacts on t_animal's top face to follow target order so \
             paths do not cross: t_dog ({dog_contact_x:.2}) < t_cat ({cat_contact_x:.2}) \
             < t_owner ({owner_contact_x:.2})"
        );
    }
}

/// With `rank_dir: right_to_left`, rank-1 siblings must render top-to-bottom
/// in declaration order (`t_dog`, `t_cat`, `t_owner`), and the from-contact
/// points on `t_animal`'s left face must follow the same order so the edge
/// paths do not cross.
#[test]
fn test_rank_dir_right_to_left_siblings_render_in_declaration_order() {
    for svg_elements in build_svg_elements_from_rank_dir_reversed_siblings("right_to_left") {
        let node_y = |node_id: &str| -> f32 {
            svg_elements
                .svg_node_infos
                .iter()
                .find(|n| n.node_id.as_str() == node_id)
                .unwrap_or_else(|| panic!("Expected {node_id} in svg_node_infos"))
                .y
        };
        let dog_y = node_y("t_dog");
        let cat_y = node_y("t_cat");
        let owner_y = node_y("t_owner");
        assert!(
            dog_y < cat_y && cat_y < owner_y,
            "Expected siblings to render top-to-bottom in declaration order: \
             t_dog (y={dog_y:.2}) < t_cat (y={cat_y:.2}) < t_owner (y={owner_y:.2})"
        );

        let from_contact_y = |to_node_id: &str| -> f32 {
            let edge_info = svg_elements
                .svg_edge_infos
                .iter()
                .find(|e| {
                    e.from_node_id.as_str() == "t_animal" && e.to_node_id.as_str() == to_node_id
                })
                .unwrap_or_else(|| panic!("Expected edge from t_animal to {to_node_id}"));
            path_d_last_point(&edge_info.path_d)
                .unwrap_or_else(|| panic!("Expected parseable path_d: {:?}", edge_info.path_d))
                .1
        };
        let dog_contact_y = from_contact_y("t_dog");
        let cat_contact_y = from_contact_y("t_cat");
        let owner_contact_y = from_contact_y("t_owner");
        assert!(
            dog_contact_y < cat_contact_y && cat_contact_y < owner_contact_y,
            "Expected from-contacts on t_animal's left face to follow target order so \
             paths do not cross: t_dog ({dog_contact_y:.2}) < t_cat ({cat_contact_y:.2}) \
             < t_owner ({owner_contact_y:.2})"
        );
    }
}

// === Nested-node rank ordering follows `RankDir` (0023-0026) === //

/// Returns the `x` and `y` of a node from `svg_node_infos`.
fn nested_rank_node_x_y(svg_elements: &SvgElements<'static>, node_id: &str) -> (f32, f32) {
    let svg_node_info = svg_elements
        .svg_node_infos
        .iter()
        .find(|svg_node_info| svg_node_info.node_id.as_str() == node_id)
        .unwrap_or_else(|| panic!("Expected {node_id} in svg_node_infos"));
    (svg_node_info.x, svg_node_info.y)
}

/// With `rank_dir: top_to_bottom`, a nested rank-1 node (`t_b0`) must render
/// below its rank-0 dependency (`t_a0`) -- a greater `y`.
#[test]
fn test_nested_rank_dir_top_to_bottom_higher_rank_below() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0023_NESTED_EDGES_RANK_DIR_TOP_TO_BOTTOM)
    {
        let (_t_a0_x, t_a0_y) = nested_rank_node_x_y(&svg_elements, "t_a0");
        let (_t_b0_x, t_b0_y) = nested_rank_node_x_y(&svg_elements, "t_b0");
        assert!(
            t_b0_y > t_a0_y,
            "Expected nested rank-1 t_b0 (y={t_b0_y:.2}) to be below rank-0 t_a0 \
             (y={t_a0_y:.2}) for top_to_bottom"
        );
    }
}

/// With `rank_dir: left_to_right`, a nested rank-1 node (`t_b0`) must render to
/// the right of its rank-0 dependency (`t_a0`) -- a greater `x`.
#[test]
fn test_nested_rank_dir_left_to_right_higher_rank_right() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0024_NESTED_EDGES_RANK_DIR_LEFT_TO_RIGHT)
    {
        let (t_a0_x, _t_a0_y) = nested_rank_node_x_y(&svg_elements, "t_a0");
        let (t_b0_x, _t_b0_y) = nested_rank_node_x_y(&svg_elements, "t_b0");
        assert!(
            t_b0_x > t_a0_x,
            "Expected nested rank-1 t_b0 (x={t_b0_x:.2}) to be right of rank-0 t_a0 \
             (x={t_a0_x:.2}) for left_to_right"
        );
    }
}

/// With `rank_dir: right_to_left`, a nested rank-1 node (`t_b0`) must render to
/// the left of its rank-0 dependency (`t_a0`) -- a smaller `x`.
#[test]
fn test_nested_rank_dir_right_to_left_higher_rank_left() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0025_NESTED_EDGES_RANK_DIR_RIGHT_TO_LEFT)
    {
        let (t_a0_x, _t_a0_y) = nested_rank_node_x_y(&svg_elements, "t_a0");
        let (t_b0_x, _t_b0_y) = nested_rank_node_x_y(&svg_elements, "t_b0");
        assert!(
            t_b0_x < t_a0_x,
            "Expected nested rank-1 t_b0 (x={t_b0_x:.2}) to be left of rank-0 t_a0 \
             (x={t_a0_x:.2}) for right_to_left"
        );
    }
}

/// With `rank_dir: bottom_to_top`, a nested rank-1 node (`t_b0`) must render
/// above its rank-0 dependency (`t_a0`) -- a smaller `y`.
#[test]
fn test_nested_rank_dir_bottom_to_top_higher_rank_above() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0026_NESTED_EDGES_RANK_DIR_BOTTOM_TO_TOP)
    {
        let (_t_a0_x, t_a0_y) = nested_rank_node_x_y(&svg_elements, "t_a0");
        let (_t_b0_x, t_b0_y) = nested_rank_node_x_y(&svg_elements, "t_b0");
        assert!(
            t_b0_y < t_a0_y,
            "Expected nested rank-1 t_b0 (y={t_b0_y:.2}) to be above rank-0 t_a0 \
             (y={t_a0_y:.2}) for bottom_to_top"
        );
    }
}

// === Self-loop rank-direction faces (0010) === //

/// Parses the first point of a kurbo-generated path `d` attribute (the `M`
/// command's coordinates).
fn path_d_first_point(path_d: &str) -> Option<(f32, f32)> {
    let token = path_d.split_whitespace().next()?;
    let coords = token.trim_start_matches(|c: char| c.is_ascii_alphabetic());
    let (x_str, y_str) = coords.split_once(',')?;
    Some((x_str.parse().ok()?, y_str.parse().ok()?))
}

/// Self-loop paths must exit and re-enter the rank-direction face:
/// `Bottom` for `top_to_bottom`, `Top` for `bottom_to_top`, `Right` for
/// `left_to_right`, and `Left` for `right_to_left`.
///
/// The path's first point (arrow end) and last point (exit end) must both lie
/// on the expected face line of the node.
#[test]
fn test_self_loop_path_endpoints_follow_rank_dir() {
    for rank_dir in [
        "top_to_bottom",
        "bottom_to_top",
        "left_to_right",
        "right_to_left",
    ] {
        let input_diagram = format!(
            "{INPUT_DIAGRAM_0010_SELF_LOOP_EDGE_WITH_DESCRIPTION}\nrender_options:\n  rank_dir: {rank_dir}\n"
        );
        for svg_elements in build_svg_elements_for_diagram(&input_diagram)
            .collect::<Vec<_>>()
            .into_iter()
        {
            let t_a = svg_elements
                .svg_node_infos
                .iter()
                .find(|n| n.node_id.as_str() == "t_a")
                .expect("Expected t_a in svg_node_infos");

            let edge_info = svg_elements
                .svg_edge_infos
                .iter()
                .find(|e| e.from_node_id.as_str() == "t_a" && e.to_node_id.as_str() == "t_a")
                .expect("Expected self-loop edge on t_a");

            let first_point = path_d_first_point(&edge_info.path_d)
                .unwrap_or_else(|| panic!("Expected parseable path_d: {:?}", edge_info.path_d));
            let last_point = path_d_last_point(&edge_info.path_d)
                .unwrap_or_else(|| panic!("Expected parseable path_d: {:?}", edge_info.path_d));

            // The face line coordinate both endpoints must sit on: y for
            // Top/Bottom faces, x for Left/Right faces.
            let (face_coord, endpoint_coords) = match rank_dir {
                "top_to_bottom" => (t_a.y + t_a.height_collapsed, [first_point.1, last_point.1]),
                "bottom_to_top" => (t_a.y, [first_point.1, last_point.1]),
                "left_to_right" => (t_a.x + t_a.width, [first_point.0, last_point.0]),
                "right_to_left" => (t_a.x, [first_point.0, last_point.0]),
                _ => unreachable!(),
            };

            for endpoint_coord in endpoint_coords {
                assert!(
                    (endpoint_coord - face_coord).abs() < 0.5,
                    "Self-loop endpoint coordinate {endpoint_coord:.2} should lie on the \
                     {rank_dir} face line {face_coord:.2}. path_d = {:?}",
                    edge_info.path_d,
                );
            }
        }
    }
}

// === Arrow-head clearance for orthogonal to-protrusions === //

/// Orthogonal `to_protrusion` must give the path a straight segment long
/// enough to contain the arrow head (8.0 px) plus 3.0 px of clearance before
/// the Z/S bend, where the rank gap allows.
///
/// The floor is capped at `MAX_GAP_FRACTION` (0.9) of the endpoint's rank
/// gap, so it never overshoots tight gaps.
#[test]
fn test_to_protrusion_clears_arrow_head() {
    // ARROW_HEAD_LENGTH (8.0) + ARROW_HEAD_CLEARANCE_PX (3.0).
    const TO_PROTRUSION_MIN_PX: f32 = 11.0;
    const EPSILON: f32 = 0.01;

    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0001_NESTED_NODE_EDGE_PROTRUSION)
    {
        let node_info = |node_id: &str| {
            svg_elements
                .svg_node_infos
                .iter()
                .find(|n| n.node_id.as_str() == node_id)
                .unwrap_or_else(|| panic!("Expected {node_id} in svg_node_infos"))
        };
        let charlie = node_info("t_charlie");

        // (from node, its root-level ancestor whose bottom face bounds the
        // visual rank gap above t_charlie)
        for (from_node_id, from_ancestor_id) in [("t_alice", "t_alice_outer"), ("t_bob", "t_bob")] {
            let from = node_info(from_node_id);
            let from_ancestor = node_info(from_ancestor_id);

            let edge_info = svg_elements
                .svg_edge_infos
                .iter()
                .find(|e| {
                    e.from_node_id.as_str() == from_node_id && e.to_node_id.as_str() == "t_charlie"
                })
                .unwrap_or_else(|| panic!("Expected edge {from_node_id} -> t_charlie"));
            let to_protrusion = edge_info.ortho_protrusion_params.to_protrusion;

            // Precondition: the visual rank gap above t_charlie comfortably
            // allows the floor (otherwise this test would assert the capped
            // value instead).
            let rank_gap_visual = charlie.y - (from_ancestor.y + from_ancestor.height_collapsed);
            assert!(
                rank_gap_visual * MAX_GAP_FRACTION > TO_PROTRUSION_MIN_PX,
                "Fixture rank gap {rank_gap_visual:.2} should comfortably allow the \
                 to-protrusion floor"
            );

            assert!(
                to_protrusion >= TO_PROTRUSION_MIN_PX - EPSILON,
                "edge {from_node_id} -> t_charlie to_protrusion {to_protrusion:.2} should \
                 be at least {TO_PROTRUSION_MIN_PX} so the Z/S bend clears the arrow head"
            );

            // The floor never exceeds the gap allowance: the protrusion stays
            // within MAX_GAP_FRACTION of the distance to the from-node face.
            let rank_gap_to_from_face = charlie.y - (from.y + from.height_collapsed);
            assert!(
                to_protrusion <= rank_gap_to_from_face * MAX_GAP_FRACTION + EPSILON,
                "edge {from_node_id} -> t_charlie to_protrusion {to_protrusion:.2} should \
                 not exceed {MAX_GAP_FRACTION} of the rank gap {rank_gap_to_from_face:.2}"
            );
        }
    }
}

/// Within a single rank gap, the from-side and to-side protrusion fans share
/// the `MAX_GAP_FRACTION * gap` band, split proportionally by side count, with
/// a per-side arrow-head floor. Because the two fans grow from opposite gap
/// boundaries toward each other, the deepest from-protrusion plus the deepest
/// to-protrusion must fit within `MAX_GAP_FRACTION * gap`, so their tips never
/// cross.
///
/// Verified on a fan-in: three rank-0 nodes (`t_alice`, `t_bob`, `t_charlie`)
/// all connecting to one rank-1 node (`t_delta`).
#[test]
fn test_fan_in_protrusions_do_not_cross_within_gap() {
    const EPSILON: f32 = 0.1;

    for svg_elements in build_svg_elements_for_diagram(INPUT_DIAGRAM_0022_EDGES_FAN_IN_3_TO_1) {
        let node_info = |node_id: &str| {
            svg_elements
                .svg_node_infos
                .iter()
                .find(|n| n.node_id.as_str() == node_id)
                .unwrap_or_else(|| panic!("Expected {node_id} in svg_node_infos"))
        };
        let delta = node_info("t_delta");

        // The three rank-0 from-nodes share the gap below them; the visual gap
        // spans from the lowest rank-0 bottom face down to t_delta's top face.
        let from_ids = ["t_alice", "t_bob", "t_charlie"];
        let rank0_bottom_max = from_ids
            .iter()
            .map(|id| {
                let node = node_info(id);
                node.y + node.height_collapsed
            })
            .fold(f32::MIN, f32::max);
        let visual_gap = delta.y - rank0_bottom_max;
        assert!(
            visual_gap > 0.0,
            "Expected a positive visual gap between rank 0 and t_delta, got {visual_gap:.2}"
        );

        // Deepest protrusion on each side of the gap.
        let mut deepest_from = 0.0f32;
        let mut deepest_to = 0.0f32;
        for edge_info in &svg_elements.svg_edge_infos {
            if !from_ids.contains(&edge_info.from_node_id.as_str())
                || edge_info.to_node_id.as_str() != "t_delta"
            {
                continue;
            }
            let params = &edge_info.ortho_protrusion_params;
            deepest_from = deepest_from.max(params.from_protrusion);
            deepest_to = deepest_to.max(params.to_protrusion);

            // Every to-endpoint clears the arrow head.
            assert!(
                params.to_protrusion >= TO_PROTRUSION_MIN_PX - EPSILON,
                "edge {} -> t_delta to_protrusion {:.2} should clear the arrow head \
                 (>= {TO_PROTRUSION_MIN_PX})",
                edge_info.from_node_id.as_str(),
                params.to_protrusion,
            );
        }

        // The two fans grow from opposite gap boundaries toward each other, so
        // their deepest tips together must fit within MAX_GAP_FRACTION of the
        // gap and never cross.
        assert!(
            deepest_from + deepest_to <= visual_gap * MAX_GAP_FRACTION + EPSILON,
            "deepest from-protrusion {deepest_from:.2} + deepest to-protrusion \
             {deepest_to:.2} = {:.2} should fit within {MAX_GAP_FRACTION} of the rank \
             gap {visual_gap:.2} (= {:.2}) so the protrusion tips do not cross",
            deepest_from + deepest_to,
            visual_gap * MAX_GAP_FRACTION,
        );
    }
}

/// Cycle edges and self-loops also carry an arrow head at their to-endpoint,
/// so their symmetric U-depth must satisfy the arrow-head clearance floor
/// (8.0 px arrow head + 3.0 px clearance = 11.0 px).
#[test]
fn test_cycle_and_self_loop_protrusions_clear_arrow_head() {
    // ARROW_HEAD_LENGTH (8.0) + ARROW_HEAD_CLEARANCE_PX (3.0).
    const TO_PROTRUSION_MIN_PX: f32 = 11.0;
    const EPSILON: f32 = 0.01;

    // Self-loop (0010): single node, boundary rank -- unregistered fallback.
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0010_SELF_LOOP_EDGE_WITH_DESCRIPTION)
    {
        let edge_info = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.from_node_id.as_str() == "t_a" && e.to_node_id.as_str() == "t_a")
            .expect("Expected self-loop edge on t_a");
        let to_protrusion = edge_info.ortho_protrusion_params.to_protrusion;
        assert!(
            to_protrusion >= TO_PROTRUSION_MIN_PX - EPSILON,
            "Self-loop to_protrusion {to_protrusion:.2} should be at least \
             {TO_PROTRUSION_MIN_PX} so the U-bend clears the arrow head"
        );
        assert_eq!(
            edge_info.ortho_protrusion_params.from_protrusion, to_protrusion,
            "Self-loop from/to protrusions should be equal (symmetric U)"
        );
    }

    // Cycle edges (0004): t_alice <-> t_charlie are non-adjacent siblings at
    // the same rank, routed as clockwise cycle edges.
    for svg_elements in build_svg_elements_for_diagram(
        crate::input_ir_rt::INPUT_DIAGRAM_0004_EDGES_SYMMETRIC_3_NODES,
    ) {
        for (from_node_id, to_node_id) in [("t_alice", "t_charlie"), ("t_charlie", "t_alice")] {
            let edge_info = svg_elements
                .svg_edge_infos
                .iter()
                .find(|e| {
                    e.from_node_id.as_str() == from_node_id && e.to_node_id.as_str() == to_node_id
                })
                .unwrap_or_else(|| panic!("Expected edge {from_node_id} -> {to_node_id}"));
            let to_protrusion = edge_info.ortho_protrusion_params.to_protrusion;
            assert!(
                to_protrusion >= TO_PROTRUSION_MIN_PX - EPSILON,
                "Cycle edge {from_node_id} -> {to_node_id} to_protrusion \
                 {to_protrusion:.2} should be at least {TO_PROTRUSION_MIN_PX} so the \
                 U-bend clears the arrow head"
            );
            assert_eq!(
                edge_info.ortho_protrusion_params.from_protrusion, to_protrusion,
                "Cycle edge {from_node_id} -> {to_node_id} from/to protrusions should \
                 be equal (symmetric U)"
            );
        }
    }
}

/// Self-loop from/to contact points on the same face must be separated by at
/// least the face contact gap (`CONTACT_GAP_MIN_PX` = 12.0 px), so the from
/// segment clears the arrow head (4.0 px half-width) drawn at the to contact.
///
/// The from contact is label-aligned (the edge label leaf always has a
/// non-zero padded size), while the to contact falls back to the slot-based
/// offset; without enforcement the two can land arbitrarily close together.
#[test]
fn test_self_loop_contacts_honour_face_contact_gap() {
    const CONTACT_GAP_MIN_PX: f32 = 12.0;
    const EPSILON: f32 = 0.01;

    for (fixture_name, input_diagram, node_id) in [
        (
            "0020_self_loop_cyclic_two_node_left_to_right",
            INPUT_DIAGRAM_0020_SELF_LOOP_CYCLIC_TWO_NODE_LEFT_TO_RIGHT,
            "t_alice",
        ),
        (
            "0021_self_loop_edge_left_to_right_with_edge_desc",
            INPUT_DIAGRAM_0021_SELF_LOOP_EDGE_LEFT_TO_RIGHT_WITH_EDGE_DESC,
            "t_a",
        ),
    ] {
        for svg_elements in build_svg_elements_for_diagram(input_diagram) {
            let node_info = svg_elements
                .svg_node_infos
                .iter()
                .find(|n| n.node_id.as_str() == node_id)
                .unwrap_or_else(|| panic!("Expected {node_id} in svg_node_infos"));

            let edge_info = svg_elements
                .svg_edge_infos
                .iter()
                .find(|e| e.from_node_id.as_str() == node_id && e.to_node_id.as_str() == node_id)
                .unwrap_or_else(|| panic!("Expected self-loop edge on {node_id}"));

            // Paths are built in reverse: first point = to contact (arrow
            // end), last point = from contact.
            let to_contact = path_d_first_point(&edge_info.path_d)
                .unwrap_or_else(|| panic!("Expected parseable path_d: {:?}", edge_info.path_d));
            let from_contact = path_d_last_point(&edge_info.path_d)
                .unwrap_or_else(|| panic!("Expected parseable path_d: {:?}", edge_info.path_d));

            // Both contacts sit on the right face (rank_dir: left_to_right).
            let right_face_x = node_info.x + node_info.width;
            for (contact_name, contact_x) in [("to", to_contact.0), ("from", from_contact.0)] {
                assert!(
                    (contact_x - right_face_x).abs() < 0.5,
                    "{fixture_name}: self-loop {contact_name} contact x {contact_x:.2} \
                     should lie on the right face line {right_face_x:.2}"
                );
            }

            let contact_separation = (to_contact.1 - from_contact.1).abs();
            assert!(
                contact_separation >= CONTACT_GAP_MIN_PX - EPSILON,
                "{fixture_name}: self-loop from/to contacts should be at least \
                 {CONTACT_GAP_MIN_PX} px apart so the from segment clears the arrow \
                 head, but were {contact_separation:.2} px apart \
                 (from y = {:.2}, to y = {:.2})",
                from_contact.1,
                to_contact.1,
            );
        }
    }
}

// === Edges from a nested node to a node in a sibling container (0031-0036) ===
// //

/// Flow direction of a diagram, used to pick the cross axis of an edge path.
#[derive(Clone, Copy)]
enum FlowAxis {
    /// `rank_dir: top_to_bottom` / `bottom_to_top` -- cross axis is X.
    Vertical,
    /// `rank_dir: left_to_right` / `right_to_left` -- cross axis is Y.
    Horizontal,
}

impl FlowAxis {
    /// Returns the cross-axis coordinate of a path point.
    fn cross(self, point: (f32, f32)) -> f32 {
        match self {
            FlowAxis::Vertical => point.0,
            FlowAxis::Horizontal => point.1,
        }
    }
}

/// Returns whether the orthogonal segment `a`-`b` passes through (the interior
/// of) the axis-aligned rectangle, allowing a small tolerance so a path running
/// flush along a face is not counted as intersecting.
fn segment_intersects_rect(
    a: (f32, f32),
    b: (f32, f32),
    rect_x_min: f32,
    rect_y_min: f32,
    rect_x_max: f32,
    rect_y_max: f32,
) -> bool {
    let tolerance = 0.5_f32;
    let seg_x_min = a.0.min(b.0);
    let seg_x_max = a.0.max(b.0);
    let seg_y_min = a.1.min(b.1);
    let seg_y_max = a.1.max(b.1);

    seg_x_max > rect_x_min + tolerance
        && seg_x_min < rect_x_max - tolerance
        && seg_y_max > rect_y_min + tolerance
        && seg_y_min < rect_y_max - tolerance
}

/// Asserts none of an edge path's orthogonal segments pass through a node's
/// box.
fn assert_edge_path_clears_node(
    path_d: &str,
    node: &disposition::svg_model::SvgNodeInfo<'_>,
    node_label: &str,
) {
    let coords = parse_path_endpoints(path_d);
    let rect_x_min = node.x;
    let rect_y_min = node.y;
    let rect_x_max = node.x + node.width;
    let rect_y_max = node.y + node.height_collapsed;

    for window in coords.windows(2) {
        let [a, b] = window else { continue };
        assert!(
            !segment_intersects_rect(*a, *b, rect_x_min, rect_y_min, rect_x_max, rect_y_max),
            "Edge path segment {a:?} -> {b:?} passes through {node_label} \
             (rect x: {rect_x_min:.1}..{rect_x_max:.1}, y: {rect_y_min:.1}..{rect_y_max:.1}). \
             path_d = {path_d:?}",
        );
    }
}

/// When the `from` node is the highest-ranked child of its container, an edge
/// to a node in the next sibling container exits straight out the gap-facing
/// face: there is no higher-ranked sibling to route around, so the edge has no
/// spacers and does not detour on the cross axis.
fn assert_high_rank_from_edge_routes_straight(input_diagram: &str, axis: FlowAxis) {
    for svg_elements in build_svg_elements_for_diagram(input_diagram) {
        let edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.from_node_id.as_str() == "t_a_01" && e.to_node_id.as_str() == "t_b_00")
            .expect("Expected edge from t_a_01 to t_b_00");

        assert!(
            edge.ortho_protrusion_params.spacer_protrusions.is_empty(),
            "Expected no spacer protrusions for t_a_01 -> t_b_00 -- t_a_01 is the \
             highest-ranked child of t_a_0, so the edge exits straight out the \
             gap-facing face with no sibling to route around. \
             spacer_protrusions = {:?}, path_d = {:?}",
            edge.ortho_protrusion_params.spacer_protrusions,
            edge.path_d,
        );

        // No cross-axis detour: every vertex stays within a tight band of the
        // first contact point (both endpoints are aligned on the cross axis in
        // these fixtures).
        let coords = parse_path_endpoints(&edge.path_d);
        let first_cross = axis.cross(coords[0]);
        for &point in &coords {
            let cross = axis.cross(point);
            assert!(
                (cross - first_cross).abs() <= 12.0,
                "Edge t_a_01 -> t_b_00 detours on the cross axis: vertex {point:?} \
                 is {:.1} px from the contact line ({first_cross:.1}). The edge \
                 should route straight out the gap-facing face. path_d = {:?}",
                (cross - first_cross).abs(),
                edge.path_d,
            );
        }
    }
}

/// `0031` (`top_to_bottom`): `t_a_01` is rank 1 (highest) in `t_a_0`, so the
/// edge to `t_b_00` needs no cross-container spacer.
#[test]
fn test_0031_high_rank_from_edge_top_to_bottom_routes_straight() {
    assert_high_rank_from_edge_routes_straight(
        INPUT_DIAGRAM_0031_NESTED_NODE_HIGH_RANK_EDGE_TO_NEXT_NODE_TOP_TO_BOTTOM,
        FlowAxis::Vertical,
    );
}

/// `0032` (`left_to_right`): same as `0031`, with a horizontal flow.
#[test]
fn test_0032_high_rank_from_edge_left_to_right_routes_straight() {
    assert_high_rank_from_edge_routes_straight(
        INPUT_DIAGRAM_0032_NESTED_NODE_HIGH_RANK_EDGE_TO_NEXT_NODE_LEFT_TO_RIGHT,
        FlowAxis::Horizontal,
    );
}

/// `0033` (`right_to_left`): same as `0031`, with a reversed horizontal flow.
#[test]
fn test_0033_high_rank_from_edge_right_to_left_routes_straight() {
    assert_high_rank_from_edge_routes_straight(
        INPUT_DIAGRAM_0033_NESTED_NODE_HIGH_RANK_EDGE_TO_NEXT_NODE_RIGHT_TO_LEFT,
        FlowAxis::Horizontal,
    );
}

/// `0034` (`bottom_to_top`): same as `0031`, with a reversed vertical flow.
#[test]
fn test_0034_high_rank_from_edge_bottom_to_top_routes_straight() {
    assert_high_rank_from_edge_routes_straight(
        INPUT_DIAGRAM_0034_NESTED_NODE_HIGH_RANK_EDGE_TO_NEXT_NODE_BOTTOM_TO_TOP,
        FlowAxis::Vertical,
    );
}

/// `0035` (`top_to_bottom`): `t_a_01` is rank 1 (the middle) in `t_a_0`. The
/// edge to `t_b_00` exits toward the high-rank (bottom) face, so it must route
/// around `t_a_02` (rank 2) via a cross-container spacer on the gap side -- not
/// around `t_a_00` (rank 0), which is on the far side -- and the path must
/// clear `t_a_02`.
#[test]
fn test_0035_mid_rank_from_edge_routes_around_higher_rank_sibling() {
    for svg_elements in build_svg_elements_for_diagram(
        INPUT_DIAGRAM_0035_NESTED_NODE_MID_RANK_EDGE_TO_NEXT_NODE_TOP_TO_BOTTOM,
    ) {
        let t_a_02 = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_a_02")
            .expect("Expected t_a_02 in svg_node_infos");

        let edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.from_node_id.as_str() == "t_a_01" && e.to_node_id.as_str() == "t_b_00")
            .expect("Expected edge from t_a_01 to t_b_00");

        assert_eq!(
            edge.ortho_protrusion_params.spacer_protrusions.len(),
            1,
            "Expected exactly one cross-container spacer (routing around t_a_02 on \
             the gap side) for t_a_01 -> t_b_00. spacer_protrusions = {:?}, \
             path_d = {:?}",
            edge.ortho_protrusion_params.spacer_protrusions,
            edge.path_d,
        );

        assert_edge_path_clears_node(&edge.path_d, t_a_02, "t_a_02");
    }
}

/// Asserts the path's vertices are monotonic along the main (flow) axis -- it
/// never reverses direction. A reversal indicates the spacers were visited out
/// of order (e.g. the cross-container spacer before the LCA-gap spacer), which
/// produces a backward zigzag.
fn assert_edge_path_main_axis_monotonic(path_d: &str, axis: FlowAxis) {
    let coords = parse_path_endpoints(path_d);
    let main = |point: (f32, f32)| match axis {
        FlowAxis::Vertical => point.1,
        FlowAxis::Horizontal => point.0,
    };

    // Determine overall direction from the first and last vertices, then assert
    // every step moves in that direction (allowing a small tolerance for the
    // arc-rounded corners that briefly overshoot).
    let first = main(coords[0]);
    let last = main(coords[coords.len() - 1]);
    let increasing = last >= first;
    let tolerance = 1.0_f32;

    for window in coords.windows(2) {
        let [a, b] = window else { continue };
        let step = main(*b) - main(*a);
        let backward = if increasing {
            step < -tolerance
        } else {
            step > tolerance
        };
        assert!(
            !backward,
            "Edge path reverses on the main axis at {a:?} -> {b:?} (overall \
             direction {}). The spacers were likely visited out of order, \
             producing a backward zigzag. path_d = {path_d:?}",
            if increasing {
                "increasing"
            } else {
                "decreasing"
            },
        );
    }
}

/// Asserts that a mid-rank node connects to a high-rank nested node correctly,
/// regardless of `rank_dir`: the edge keeps both its root-level LCA-gap spacer
/// and the cross-container spacer beside `t_c_00`, routes around `t_c_00`
/// (rather than through it), and never reverses along the flow axis (the spacer
/// order must follow the flow direction even for reversed `rank_dir`s).
fn assert_mid_rank_to_high_rank_routes_cleanly(input_diagram: &str, axis: FlowAxis) {
    for svg_elements in build_svg_elements_for_diagram(input_diagram) {
        let t_c_00 = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_c_00")
            .expect("Expected t_c_00 in svg_node_infos");

        let edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.from_node_id.as_str() == "t_a_01" && e.to_node_id.as_str() == "t_c_01")
            .expect("Expected edge from t_a_01 to t_c_01");

        assert!(
            edge.ortho_protrusion_params.spacer_protrusions.len() >= 2,
            "Expected at least two spacers for t_a_01 -> t_c_01: the root-level \
             LCA-gap spacer and the cross-container spacer beside t_c_00. \
             spacer_protrusions = {:?}, path_d = {:?}",
            edge.ortho_protrusion_params.spacer_protrusions,
            edge.path_d,
        );

        assert_edge_path_clears_node(&edge.path_d, t_c_00, "t_c_00");
        assert_edge_path_main_axis_monotonic(&edge.path_d, axis);
    }
}

/// `0036` (`top_to_bottom`): `t_a_01` (rank 1 in `t_a_0`) connects to `t_c_01`
/// (rank 1 in `t_c_0`, which is rank 2 at root). The edge keeps both its
/// root-level LCA-gap spacer and the cross-container spacer beside `t_c_00`.
#[test]
fn test_0036_mid_rank_to_high_rank_top_to_bottom_routes_cleanly() {
    assert_mid_rank_to_high_rank_routes_cleanly(
        INPUT_DIAGRAM_0036_NESTED_NODE_MID_RANK_EDGE_TO_NEXT_HIGH_RANK_NODE_TOP_TO_BOTTOM,
        FlowAxis::Vertical,
    );
}

/// `0036` (`top_to_bottom`): three edges exit a `Bottom` face at the same
/// horizontal midpoint -- `t_a_0 -> t_b_0`, `t_a_00 -> t_a_01`, and
/// `t_a_01 -> t_c_01`. `t_a_0` (the container) and its centered nested children
/// `t_a_00` / `t_a_01` all share the midpoint x, so without cross-node contact
/// separation their contact points (and protrusion stubs) coincide. This
/// asserts the three contacts are spread to distinct x coordinates.
#[test]
fn test_0036_coincident_face_contacts_are_separated() {
    for svg_elements in build_svg_elements_for_diagram(
        INPUT_DIAGRAM_0036_NESTED_NODE_MID_RANK_EDGE_TO_NEXT_HIGH_RANK_NODE_TOP_TO_BOTTOM,
    ) {
        let contact_x = |from: &str, to: &str| -> f32 {
            let edge = svg_elements
                .svg_edge_infos
                .iter()
                .find(|e| e.from_node_id.as_str() == from && e.to_node_id.as_str() == to)
                .unwrap_or_else(|| panic!("Expected edge from {from} to {to}"));
            parse_path_endpoints(&edge.path_d)[0].0
        };

        let contact_x_a_0_b_0 = contact_x("t_a_0", "t_b_0");
        let contact_x_a_00_a_01 = contact_x("t_a_00", "t_a_01");
        let contact_x_a_01_c_01 = contact_x("t_a_01", "t_c_01");

        // All three contacts share the same face midpoint (74.5), so before the
        // fix they all landed at the same x. They should now be distinct.
        let min_separation = 6.0_f32;
        assert!(
            (contact_x_a_0_b_0 - contact_x_a_00_a_01).abs() >= min_separation
                && (contact_x_a_0_b_0 - contact_x_a_01_c_01).abs() >= min_separation
                && (contact_x_a_00_a_01 - contact_x_a_01_c_01).abs() >= min_separation,
            "Expected the three Bottom-face contacts to be separated by at least \
             {min_separation} px, but got t_a_0->t_b_0 = {contact_x_a_0_b_0}, \
             t_a_00->t_a_01 = {contact_x_a_00_a_01}, \
             t_a_01->t_c_01 = {contact_x_a_01_c_01}",
        );
    }
}

/// `0036` (`top_to_bottom`): the local edge `t_c_00 -> t_c_01` and the
/// cross-container edge `t_a_01 -> t_c_01` both enter `t_c_01`'s `Top` face,
/// but from different rank-gap buckets (container ranks vs LCA ranks). Without
/// coordinating their approach legs, the cross-container edge's deeper leg
/// sweeps across the local edge twice. This asserts their paths no longer
/// cross. Covered for all four rank directions (`0036`-`0039`).
#[test]
fn test_0036_to_0039_shared_to_face_edges_do_not_cross() {
    let diagrams = [
        INPUT_DIAGRAM_0036_NESTED_NODE_MID_RANK_EDGE_TO_NEXT_HIGH_RANK_NODE_TOP_TO_BOTTOM,
        INPUT_DIAGRAM_0037_NESTED_NODE_MID_RANK_EDGE_TO_NEXT_HIGH_RANK_NODE_LEFT_TO_RIGHT,
        INPUT_DIAGRAM_0038_NESTED_NODE_MID_RANK_EDGE_TO_NEXT_HIGH_RANK_NODE_RIGHT_TO_LEFT,
        INPUT_DIAGRAM_0039_NESTED_NODE_MID_RANK_EDGE_TO_NEXT_HIGH_RANK_NODE_BOTTOM_TO_TOP,
    ];

    for input_diagram in diagrams {
        for svg_elements in build_svg_elements_for_diagram(input_diagram) {
            let path_for = |from: &str, to: &str| -> Vec<(f32, f32)> {
                let edge = svg_elements
                    .svg_edge_infos
                    .iter()
                    .find(|e| e.from_node_id.as_str() == from && e.to_node_id.as_str() == to)
                    .unwrap_or_else(|| panic!("Expected edge from {from} to {to}"));
                parse_path_endpoints(&edge.path_d)
            };

            let path_a_01_c_01 = path_for("t_a_01", "t_c_01");
            let path_c_00_c_01 = path_for("t_c_00", "t_c_01");

            assert!(
                !polylines_cross(&path_a_01_c_01, &path_c_00_c_01),
                "Edges t_a_01->t_c_01 and t_c_00->t_c_01 entering the shared \
                 t_c_01 face should not cross.\n  t_a_01->t_c_01: {path_a_01_c_01:?}\n  \
                 t_c_00->t_c_01: {path_c_00_c_01:?}",
            );
        }
    }
}

/// `0036`-`0039` (all rank directions): `edge_dep_b_0_c_0` enters the container
/// `t_c_0`, while `edge_dep_a_01_c_01` transits the same inter-rank gap
/// (between `t_b_0` and `t_c_0`) to reach `t_c_01` nested inside `t_c_0`. In
/// the horizontal flows their near-parallel legs touched (~1 px apart); in the
/// vertical flows `b_0_c_0`'s approach sweep crossed `a_01_c_01`'s transit leg.
/// The container contact is now kept on the from-node's side of the transit
/// (clearing it by a contact gap), so the two paths keep a clear margin in
/// every rank direction.
#[test]
fn test_0036_to_0039_container_entry_clears_nested_transit() {
    let diagrams = [
        INPUT_DIAGRAM_0036_NESTED_NODE_MID_RANK_EDGE_TO_NEXT_HIGH_RANK_NODE_TOP_TO_BOTTOM,
        INPUT_DIAGRAM_0037_NESTED_NODE_MID_RANK_EDGE_TO_NEXT_HIGH_RANK_NODE_LEFT_TO_RIGHT,
        INPUT_DIAGRAM_0038_NESTED_NODE_MID_RANK_EDGE_TO_NEXT_HIGH_RANK_NODE_RIGHT_TO_LEFT,
        INPUT_DIAGRAM_0039_NESTED_NODE_MID_RANK_EDGE_TO_NEXT_HIGH_RANK_NODE_BOTTOM_TO_TOP,
    ];

    // Arrow-head clearance: legs closer than this read as a single line.
    let min_clearance = 3.0_f32;

    for input_diagram in diagrams {
        for svg_elements in build_svg_elements_for_diagram(input_diagram) {
            let path_for = |from: &str, to: &str| -> Vec<(f32, f32)> {
                let edge = svg_elements
                    .svg_edge_infos
                    .iter()
                    .find(|e| e.from_node_id.as_str() == from && e.to_node_id.as_str() == to)
                    .unwrap_or_else(|| panic!("Expected edge from {from} to {to}"));
                parse_path_endpoints(&edge.path_d)
            };

            let path_b_0_c_0 = path_for("t_b_0", "t_c_0");
            let path_a_01_c_01 = path_for("t_a_01", "t_c_01");

            let distance = polylines_min_distance(&path_b_0_c_0, &path_a_01_c_01);
            assert!(
                distance >= min_clearance,
                "Edge t_b_0->t_c_0 (into container) and t_a_01->t_c_01 (transiting \
                 to a nested node) come within {distance} px (< {min_clearance}).\n  \
                 t_b_0->t_c_0: {path_b_0_c_0:?}\n  t_a_01->t_c_01: {path_a_01_c_01:?}",
            );
        }
    }
}

/// `0043` (`top_to_bottom`): three cross-container edges fan out from sibling
/// nodes at the same rank in `t_inputs` to nodes nested at three different
/// depths inside `t_offset_data` (`t_taffy_layout -> t_face_contacts`,
/// `t_node_ranks -> t_slot_indices`, `t_edge_labels -> t_offsets`). All three
/// are LCA-lifted to the same root rank gap, so the proportional band split
/// collapsed their `from` jogs onto (near-)identical depths and the horizontal
/// "jog" legs carrying each edge from its from-column to its spacer-column all
/// sat at the same y, reading as one line.
///
/// The interval-graph separation
/// (`OrthoProtrusionCalculator::side_jogs_separate`) only forces distinct
/// depths for legs whose cross-axis (x) spans actually overlap; legs whose
/// spans are disjoint may share a depth without coinciding. This asserts that
/// every **overlapping** pair of jog legs is separated, which
/// is the requirement that keeps them from reading as one line.
#[test]
fn test_0043_cross_container_fan_from_protrusions_separated() {
    // The first horizontal segment of `points` -- the from-protrusion jog --
    // as `(y, x_lo, x_hi)`. Skips the rounded-corner curve points (which are
    // neither horizontal nor vertical) and returns the first truly horizontal
    // segment.
    fn first_horizontal_jog(points: &[(f32, f32)]) -> (f32, f32, f32) {
        points
            .windows(2)
            .find(|seg| (seg[0].1 - seg[1].1).abs() < 1e-2 && (seg[0].0 - seg[1].0).abs() > 1e-2)
            .map(|seg| (seg[0].1, seg[0].0.min(seg[1].0), seg[0].0.max(seg[1].0)))
            .expect("Expected at least one horizontal segment in the edge path")
    }

    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0043_EDGE_OFFSETS_AND_PROTRUSION_COMPLEX_1)
    {
        let jog = |from: &str, to: &str| -> (f32, f32, f32) {
            let edge = svg_elements
                .svg_edge_infos
                .iter()
                .find(|e| e.from_node_id.as_str() == from && e.to_node_id.as_str() == to)
                .unwrap_or_else(|| panic!("Expected edge from {from} to {to}"));
            first_horizontal_jog(&parse_path_endpoints(&edge.path_d))
        };

        let jogs = [
            (
                "t_taffy_layout",
                "t_face_contacts",
                jog("t_taffy_layout", "t_face_contacts"),
            ),
            (
                "t_node_ranks",
                "t_slot_indices",
                jog("t_node_ranks", "t_slot_indices"),
            ),
            (
                "t_edge_labels",
                "t_offsets",
                jog("t_edge_labels", "t_offsets"),
            ),
        ];

        // Overlapping legs closer than this read as a single line.
        let min_separation = 6.0_f32;
        for i in 0..jogs.len() {
            for j in (i + 1)..jogs.len() {
                let (from_a, to_a, (y_a, lo_a, hi_a)) = jogs[i];
                let (from_b, to_b, (y_b, lo_b, hi_b)) = jogs[j];
                let x_spans_overlap = hi_a.min(hi_b) - lo_a.max(lo_b) > 1e-2;
                if !x_spans_overlap {
                    // Disjoint legs may share a depth; they never coincide.
                    continue;
                }
                assert!(
                    (y_a - y_b).abs() >= min_separation,
                    "Expected overlapping jog legs {from_a}->{to_a} (y={y_a}, \
                     x=[{lo_a},{hi_a}]) and {from_b}->{to_b} (y={y_b}, \
                     x=[{lo_b},{hi_b}]) to be separated by at least \
                     {min_separation} px",
                );
            }
        }
    }
}

/// `0043` (`top_to_bottom`): the same three cross-container edges fan from
/// `t_inputs` into nodes nested in `t_offset_data`. The lateral legs they run
/// in the **inter-rank gap** between the two containers -- the legs that
/// previously collapsed onto one coordinate and read as one line -- must run at
/// distinct depths.
///
/// Only the inter-rank-gap legs (above `t_offset_data`'s top) are checked. The
/// `jogs_separate` pass works per lowest-common-ancestor rank gap, so it lifts
/// and separates these legs but cannot coordinate the depths of the deeper
/// spacer-to-spacer transition legs **inside** `t_offset_data`, whose jog
/// coordinate is governed by spacer protrusions split across several rank-gap
/// buckets. Separating those tight in-container transitions is a known
/// limitation of the non-physical (LCA-bucket) approach; the from-side gap legs
/// are the reported defect and what this pass targets. Clean X-crossings (the
/// edges fan to nodes at three different ranks) are visually acceptable and
/// ignored; only coincident **parallel** overlapping legs are a defect.
#[test]
fn test_0043_cross_container_fan_legs_not_coincident() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0043_EDGE_OFFSETS_AND_PROTRUSION_COMPLEX_1)
    {
        // Top of the destination container: legs above this y are in the
        // inter-rank gap.
        let container_top = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_offset_data")
            .map(|n| n.y)
            .expect("Expected t_offset_data node");

        let path_for = |from: &str, to: &str| -> Vec<(f32, f32)> {
            let edge = svg_elements
                .svg_edge_infos
                .iter()
                .find(|e| e.from_node_id.as_str() == from && e.to_node_id.as_str() == to)
                .unwrap_or_else(|| panic!("Expected edge from {from} to {to}"));
            // Prefix of the path that stays in the inter-rank gap (above the
            // destination container), i.e. the from-side approach legs.
            let points = parse_path_endpoints(&edge.path_d);
            points
                .iter()
                .take_while(|(_, y)| *y <= container_top + 1.0)
                .copied()
                .collect()
        };

        let paths = [
            ("t_taffy_layout", "t_face_contacts"),
            ("t_node_ranks", "t_slot_indices"),
            ("t_edge_labels", "t_offsets"),
        ]
        .map(|(from, to)| (from, to, path_for(from, to)));

        // Legs closer than this read as a single line.
        let min_clearance = 2.5_f32;

        for i in 0..paths.len() {
            for j in (i + 1)..paths.len() {
                let (from_a, to_a, path_a) = &paths[i];
                let (from_b, to_b, path_b) = &paths[j];
                let gap = parallel_segment_min_gap(path_a, path_b);
                assert!(
                    gap >= min_clearance,
                    "Edge {from_a}->{to_a} and {from_b}->{to_b} have parallel \
                     inter-rank-gap legs only {gap} px apart (< {min_clearance}), \
                     reading as one line.\n  {from_a}->{to_a}: {path_a:?}\n  \
                     {from_b}->{to_b}: {path_b:?}",
                );
            }
        }
    }
}

/// `0043` (`top_to_bottom`): a dependency edge and an interaction edge run
/// between the same two nodes (`t_ir_diagram -> t_pass1_path`), both contacting
/// `t_pass1_path`'s Top face. Dependency and interaction contacts are spread in
/// separate slot pools, so the lone dependency contact stays centred on the
/// face instead of being fanned aside by the co-located interaction edge.
#[test]
fn test_0043_dependency_contact_centred_independent_of_interaction_edge() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0043_EDGE_OFFSETS_AND_PROTRUSION_COMPLEX_1)
    {
        let node_centre_x = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_pass1_path")
            .map(|n| n.x + n.width / 2.0)
            .expect("Expected t_pass1_path node");

        // To-contact (final path point) x of the edge with the given id.
        let to_contact_x = |edge_id: &str| -> f32 {
            let edge = svg_elements
                .svg_edge_infos
                .iter()
                .find(|e| e.edge_id.as_str() == edge_id)
                .unwrap_or_else(|| panic!("Expected edge {edge_id}"));
            parse_path_endpoints(&edge.path_d)
                .last()
                .expect("Expected at least one path point")
                .0
        };

        let dependency_contact_x = to_contact_x("edge_dep_ir_pass1__0");
        assert!(
            (dependency_contact_x - node_centre_x).abs() < 1.0,
            "Expected dependency edge `edge_dep_ir_pass1__0` to contact \
             t_pass1_path's Top face at its centre (x={node_centre_x}), but it \
             landed at x={dependency_contact_x} -- the interaction edge should \
             not push it aside",
        );
    }
}

/// `0043` (`top_to_bottom`): two dependency edges enter `t_face_contacts`'s Top
/// face -- `edge_dep_pass1_contacts` approaches from `t_pass1_path` on the left,
/// `edge_dep_layout_contacts` approaches via a spacer from `t_taffy_layout` on
/// the right. Contacts are ordered by the side each edge approaches from, so the
/// left-source edge takes the left contact and the two paths do not cross.
#[test]
fn test_0043_shared_target_face_contacts_ordered_by_source() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0043_EDGE_OFFSETS_AND_PROTRUSION_COMPLEX_1)
    {
        let to_contact_x = |edge_id: &str| -> f32 {
            let edge = svg_elements
                .svg_edge_infos
                .iter()
                .find(|e| e.edge_id.as_str() == edge_id)
                .unwrap_or_else(|| panic!("Expected edge {edge_id}"));
            parse_path_endpoints(&edge.path_d)
                .last()
                .expect("Expected at least one path point")
                .0
        };

        let pass1_contact_x = to_contact_x("edge_dep_pass1_contacts__0");
        let layout_contact_x = to_contact_x("edge_dep_layout_contacts__0");
        assert!(
            pass1_contact_x < layout_contact_x,
            "Expected `edge_dep_pass1_contacts__0` (approaching from the left) to \
             contact t_face_contacts left of `edge_dep_layout_contacts__0` \
             (approaching from the right), but got pass1 x={pass1_contact_x} and \
             layout x={layout_contact_x} -- the paths cross",
        );
    }
}

/// `0037` (`left_to_right`): same as `0036` with a horizontal flow.
#[test]
fn test_0037_mid_rank_to_high_rank_left_to_right_routes_cleanly() {
    assert_mid_rank_to_high_rank_routes_cleanly(
        INPUT_DIAGRAM_0037_NESTED_NODE_MID_RANK_EDGE_TO_NEXT_HIGH_RANK_NODE_LEFT_TO_RIGHT,
        FlowAxis::Horizontal,
    );
}

/// `0038` (`right_to_left`): a reversed horizontal flow. The spacers must be
/// visited in flow order (LCA-gap first, then the spacer beside `t_c_00`),
/// which requires the merged-spacer sort to be reversed for `RightToLeft` --
/// otherwise the path zigzags backward.
#[test]
fn test_0038_mid_rank_to_high_rank_right_to_left_routes_cleanly() {
    assert_mid_rank_to_high_rank_routes_cleanly(
        INPUT_DIAGRAM_0038_NESTED_NODE_MID_RANK_EDGE_TO_NEXT_HIGH_RANK_NODE_RIGHT_TO_LEFT,
        FlowAxis::Horizontal,
    );
}

/// `0039` (`bottom_to_top`): a reversed vertical flow. As with `0038`, the
/// merged-spacer sort must be reversed for `BottomToTop` so the spacers are
/// visited in flow order rather than producing a backward zigzag.
#[test]
fn test_0039_mid_rank_to_high_rank_bottom_to_top_routes_cleanly() {
    assert_mid_rank_to_high_rank_routes_cleanly(
        INPUT_DIAGRAM_0039_NESTED_NODE_MID_RANK_EDGE_TO_NEXT_HIGH_RANK_NODE_BOTTOM_TO_TOP,
        FlowAxis::Vertical,
    );
}
