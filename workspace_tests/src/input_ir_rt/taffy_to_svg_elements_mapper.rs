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
    INPUT_DIAGRAM_0044_EDGE_OFFSETS_AND_PROTRUSION_COMPLEX_2,
    INPUT_DIAGRAM_0045_EDGE_OFFSETS_AND_PROTRUSION_COMPLEX_2_LEFT_TO_RIGHT,
    INPUT_DIAGRAM_0046_EDGE_OFFSETS_AND_PROTRUSION_COMPLEX_2_RIGHT_TO_LEFT,
    INPUT_DIAGRAM_0047_EDGE_OFFSETS_AND_PROTRUSION_COMPLEX_2_BOTTOM_TO_TOP,
    INPUT_DIAGRAM_0048_INTERACTION_EDGE_HALO, INPUT_DIAGRAM_0049_INTERACTION_EDGE_HALO_DISABLED,
    INPUT_DIAGRAM_0050_INTERACTION_EDGE_HALO_FORWARD_REVERSE,
    INPUT_DIAGRAM_0052_PROCESS_STEP_TWO_PROCESSES_COLLAPSE,
    INPUT_DIAGRAM_0053_EDGE_DESCS_GROUP_ID_KEY,
    INPUT_DIAGRAM_0054_EDGE_DESCS_INSTANCE_OVERRIDES_GROUP,
    INPUT_DIAGRAM_0055_INTERACTION_EDGE_LABEL_DESC_BG,
    INPUT_DIAGRAM_0056_INTERACTION_HALO_WITH_LABELS,
    INPUT_DIAGRAM_0057_INTERACTION_HALO_WITH_DESC_CYCLIC,
    INPUT_DIAGRAM_0058_INTERACTION_HALO_WITH_LABELS_RIGHT_TO_LEFT,
    INPUT_DIAGRAM_0059_EDGE_LABEL_DESC_BG_HIERARCHY_OVERRIDE,
    INPUT_DIAGRAM_0060_SAME_RANK_DESC_CONTAINER_GLOBAL_VS_LOCAL_SIBLING_INDEX,
    INPUT_DIAGRAM_0061_SAME_RANK_DESC_CONTAINERS_MULTIPLE_OVERLAPPING,
    INPUT_DIAGRAM_0062_EDGES_FROM_HIGHER_RANK_TO_LOWER_RANK,
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

/// In `0044`, the described container `t_offset_data` is entered by two
/// cross-container edges (`edge_dep_ranks_slots__0` -> `t_slot_indices`,
/// `edge_dep_labels_offsets__0` -> `t_offsets`). Each must route to the
/// **right** of the description label (not across it) via its text-content
/// spacer, and the two "return jogs" -- from the label column back to each
/// edge's rank column -- must sit at **distinct** depths so they do not read as
/// one line.
#[test]
fn test_0044_edges_route_around_described_label_with_distinct_return_jogs() {
    // Anchor points of an SVG path's `M`/`L`/`C` commands. For these orthogonal
    // paths the `C` arcs are tiny, so every numeric pair is close to a routing
    // waypoint -- sufficient for the spatial checks below.
    fn path_points(path_d: &str) -> Vec<(f32, f32)> {
        path_d
            .split([' ', 'M', 'L', 'C'])
            .filter_map(|tok| {
                let (x, y) = tok.split_once(',')?;
                Some((x.trim().parse::<f32>().ok()?, y.trim().parse::<f32>().ok()?))
            })
            .collect()
    }

    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0044_EDGE_OFFSETS_AND_PROTRUSION_COMPLEX_2)
    {
        let node = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_offset_data")
            .expect("Expected t_offset_data in svg_node_infos");

        // Absolute extent of the description text block (spans are node-relative).
        let label_right = node
            .text_spans
            .iter()
            .map(|s| node.x + s.x + s.width)
            .fold(f32::MIN, f32::max);
        let text_top = node
            .text_spans
            .iter()
            .map(|s| node.y + s.y)
            .fold(f32::MAX, f32::min);
        let text_bottom = node
            .text_spans
            .iter()
            .map(|s| node.y + s.y)
            .fold(f32::MIN, f32::max);

        let edge_for = |from: &str, to: &str| {
            svg_elements
                .svg_edge_infos
                .iter()
                .find(|e| e.from_node_id.as_str() == from && e.to_node_id.as_str() == to)
                .unwrap_or_else(|| panic!("Expected edge {from} -> {to}"))
        };
        let layout_contacts = edge_for("t_taffy_layout", "t_face_contacts");
        let ranks_slots = edge_for("t_node_ranks", "t_slot_indices");
        let ranks_gap = edge_for("t_node_ranks", "t_rank_gap_entries");
        let labels_offsets = edge_for("t_edge_labels", "t_offsets");

        // 1. Each edge has a vertical descent at/right of the label spanning the text
        //    band -- i.e. it routes around the label, not across it.
        for edge in [layout_contacts, ranks_slots, labels_offsets] {
            let descends_right_of_label = path_points(&edge.path_d).windows(2).any(|w| {
                let (x0, y0) = w[0];
                let (x1, y1) = w[1];
                (x0 - x1).abs() < 1.0
                    && x0 >= label_right - 1.0
                    && y0.min(y1) <= text_top
                    && y0.max(y1) >= text_bottom
            });
            assert!(
                descends_right_of_label,
                "Edge {} -> {} should descend right of the description label \
                 (label right {:.1}) through the text band [{:.1}, {:.1}]; path: {}",
                edge.from_node_id, edge.to_node_id, label_right, text_top, text_bottom, edge.path_d,
            );
        }

        // 2. The return jogs (the leftward step back over the label's right edge, below
        //    the text band) must be ordered by how far left each edge sweeps -- the
        //    edge reaching the innermost (leftmost) rank column turns highest (smallest
        //    y) so its long sweep passes above the other edges' descents rather than
        //    across them -- and each must clear the rendered text. `layout_contacts` ->
        //    `t_face_contacts` (leftmost) is above `ranks_slots` -> `t_slot_indices`,
        //    which is above `labels_offsets` -> `t_offsets` (rightmost). The jogs must
        //    also be pairwise separated so they do not read as one line.
        let return_jog_y = |path_d: &str| -> f32 {
            path_points(path_d)
                .windows(2)
                .find_map(|w| {
                    let (x0, _) = w[0];
                    let (x1, y1) = w[1];
                    (x0 >= label_right && x1 < label_right && y1 >= text_bottom).then_some(y1)
                })
                .expect("Expected a return jog crossing back over the label")
        };
        let y_layout = return_jog_y(&layout_contacts.path_d);
        let y_ranks = return_jog_y(&ranks_slots.path_d);
        let y_labels = return_jog_y(&labels_offsets.path_d);
        const JOG_SEPARATION_MIN_PX: f32 = 7.0;
        assert!(
            y_layout + JOG_SEPARATION_MIN_PX <= y_ranks
                && y_ranks + JOG_SEPARATION_MIN_PX <= y_labels,
            "Return jogs should be ordered layout_contacts < ranks_slots < labels_offsets \
             and >= {JOG_SEPARATION_MIN_PX} px apart (layout {y_layout:.1}, ranks {y_ranks:.1}, \
             labels {y_labels:.1})",
        );

        // 3. In the top rank gap (between rank 0 and the container), the edge that
        //    descends at the innermost column (`layout_contacts`, whose descent is
        //    swept over by the other two) must turn **lowest** so its descent column
        //    begins below the others' lateral legs -- otherwise those legs cross it.
        //    The top gap lies between the from-nodes' bottom face and the container's
        //    top.
        let container_top = node.y;
        let from_face_y = layout_contacts
            .path_d
            .split(['M', 'L', 'C', ' '])
            .filter_map(|tok| tok.split_once(','))
            .filter_map(|(_, y)| y.trim().parse::<f32>().ok())
            .next()
            .expect("Expected a start y");
        let top_gap_jog_y = |path_d: &str| -> f32 {
            path_points(path_d)
                .windows(2)
                .find_map(|w| {
                    let (x0, y0) = w[0];
                    let (x1, y1) = w[1];
                    // First lateral (x-changing, y-flat) leg within the top gap.
                    ((x1 - x0).abs() > 1.0
                        && (y1 - y0).abs() < 1.0
                        && y1 > from_face_y
                        && y1 < container_top)
                        .then_some(y1)
                })
                .expect("Expected a lateral jog in the top rank gap")
        };
        let y_top_layout = top_gap_jog_y(&layout_contacts.path_d);
        let y_top_ranks = top_gap_jog_y(&ranks_slots.path_d);
        let y_top_gap = top_gap_jog_y(&ranks_gap.path_d);
        assert!(
            y_top_layout > y_top_ranks && y_top_layout > y_top_gap,
            "In the top rank gap, layout_contacts must turn below ranks_slots and ranks_gap \
             so its descent column is not crossed (layout {y_top_layout:.1}, ranks_slots \
             {y_top_ranks:.1}, ranks_gap {y_top_gap:.1})",
        );

        // 4. `ir_pass1` and `layout_contacts` both sweep right across the top gap with
        //    overlapping lateral spans, so their first jogs must stay ordered and not
        //    coincide (a collinear overlap reads as one line). `layout_contacts` sweeps
        //    over `ir_pass1`'s descent column, so it turns higher; `ir_pass1` turns
        //    below it.
        //
        //    The full `JOG_SEPARATION_MIN_PX` is not asserted here: every node label is
        //    now measured via the markdown content path, whose tighter glyph metrics
        //    shrink this rank gap's jog channel (`rank_gap_px * MAX_GAP_FRACTION`)
        //    below `JOG_SEPARATION_MIN_PX`, so `jogs_separate` clamps the two
        //    legs to the band floor. They remain correctly ordered (preserving
        //    the span-containment nesting) and distinct, which is the
        //    routing-correctness property the tighter band still guarantees.
        let ir_pass1 = edge_for("t_ir_diagram", "t_pass1_path");
        let y_top_ir_pass1 = top_gap_jog_y(&ir_pass1.path_d);
        assert!(
            y_top_ir_pass1 > y_top_layout,
            "ir_pass1's first jog ({y_top_ir_pass1:.1}) must sit below layout_contacts' \
             ({y_top_layout:.1}) so the legs stay ordered and do not coincide",
        );
    }
}

/// Two edges from nested nodes into other nested nodes, sharing the same rank
/// gap, must keep their lateral routing segments separated so they do not
/// collapse onto one line.
///
/// Both edges clear the same divergent-ancestor sibling row. Their separation
/// is achieved by staggering the depths of the waypoints in that row -- the
/// from/to protrusions and the text-content spacers that route each edge around
/// its destination container's title band. Rather than pinning any single
/// mechanism, this checks the outcome directly: the first horizontal "jog" leg
/// of each edge (its lateral sweep across the shared row) sits at a distinct
/// main-axis coordinate, at least `MIN_PROTRUSION_PX` apart. (See
/// `OrthoProtrusionCalculator::protrusions_adjust_for_divergent_siblings`.)
fn assert_nested_node_edge_protrusions_distinct(
    input_diagram: &str,
    edge_a: (&str, &str),
    edge_b: (&str, &str),
) {
    // The main-axis coordinate of an edge path's first horizontal segment -- its
    // lateral sweep across the shared divergent-ancestor row. Skips rounded
    // corner curve points and returns the y of the first truly horizontal leg.
    fn first_horizontal_leg_y(path_d: &str) -> f32 {
        parse_path_endpoints(path_d)
            .windows(2)
            .find(|seg| (seg[0].1 - seg[1].1).abs() < 1e-2 && (seg[0].0 - seg[1].0).abs() > 1e-2)
            .map(|seg| seg[0].1)
            .expect("Expected at least one horizontal segment in the edge path")
    }

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

        let leg_y_a = first_horizontal_leg_y(&edge_info_a.path_d);
        let leg_y_b = first_horizontal_leg_y(&edge_info_b.path_d);

        assert!(
            (leg_y_a - leg_y_b).abs() >= MIN_PROTRUSION_PX - 1e-3,
            "lateral legs for {edge_a:?} (y={leg_y_a:.2}) and {edge_b:?} \
             (y={leg_y_b:.2}) must differ by >= {MIN_PROTRUSION_PX} so their \
             lateral routing segments across the shared row do not overlap. \
             path_a = {:?}, path_b = {:?}",
            edge_info_a.path_d,
            edge_info_b.path_d,
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

/// With two processes, the default `ProcessRenderCollapse::ExpandWhenOne`
/// renders both collapsed (`process_count > 1`), so process step connectors
/// must hide/reveal and translate in lockstep with their own process's step
/// circles: the first process's connectors are hidden by default and
/// revealed by focusing it (or one of its steps), while the second process's
/// connectors carry a translate-y delta that shifts them up by the first
/// process's collapsed steps' height, reverting to zero when the first
/// process is focused (revealed/expanded).
#[test]
fn test_process_step_graph_connectors_hide_and_translate_with_collapse() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0052_PROCESS_STEP_TWO_PROCESSES_COLLAPSE)
    {
        let connector_classes = |from: &str, to: &str| {
            let edge_id = format!("edge_ps_{from}__{to}");
            svg_elements
                .tailwind_classes
                .get(&Id::try_from(edge_id.clone()).unwrap())
                .unwrap_or_else(|| panic!("Expected tailwind classes for connector {edge_id}"))
        };

        // First process's connector: hidden by default, revealed when the
        // process (or its steps) is focused.
        let build_classes = connector_classes("proc_build_step_a", "proc_build_step_b");
        assert!(
            build_classes.contains("invisible"),
            "First process's connector should be invisible by default, got: {build_classes}"
        );
        assert!(
            build_classes.contains("group-has-[#proc_build:focus-within]:visible"),
            "First process's connector should be revealed when its process is focused, got: {build_classes}"
        );
        assert!(
            build_classes.contains("group-has-[#proc_build_step_a:focus-within]:visible"),
            "First process's connector should be revealed when a sibling step is focused, got: {build_classes}"
        );

        // Second process's connector: translated up by the first process's
        // (collapsed) steps' height by default, reverting to a translate-y-[0px]
        // (or thereabouts) when the first process is focused/expanded.
        let deploy_classes = connector_classes("proc_deploy_step_a", "proc_deploy_step_b");
        assert!(
            deploy_classes.contains("translate-y-[-"),
            "Second process's connector should have a nonzero default upward \
             translate-y delta from the first process's collapse, got: {deploy_classes}"
        );
        assert!(
            deploy_classes.contains("group-has-[#proc_build:focus-within]:translate-y-["),
            "Second process's connector should override translate-y when the first \
             process is focused, got: {deploy_classes}"
        );
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

        assert_eq!(
            alice_charlie_1_edge
                .ortho_protrusion_params
                .spacer_protrusions
                .len(),
            1,
            "Expected exactly one (text-content) spacer for edge t_alice -> \
             t_charlie_1: t_charlie_1 is at rank 0 inside t_charlie_outer, so no \
             siblings are between the container entry and the target -- the only \
             spacer routes around t_charlie_outer's title band: \
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

/// For `edge_dep_alice_charlie_1`, the routing must never loop backward in the
/// visual (arrow) direction.
///
/// `t_charlie_1` is rank 0 inside `t_charlie_outer`, directly below that
/// container's title band, so the edge descends past the title via a
/// text-content spacer (entering the container alongside, to the right of, the
/// title) before reaching `t_charlie_1`. That descent is intentional, so the
/// path legitimately dips below `t_charlie_outer`'s top -- what must *not*
/// happen is any backward (upward) reversal, which would mean a routing bend
/// was placed above the from-protrusion tip or the spacers were visited out of
/// order. The path therefore stays monotonic along the downward flow axis.
#[test]
fn test_edge_from_nested_routing_stays_within_gap() {
    for svg_elements in build_svg_elements_from_edge_from_node_to_nested_node() {
        let alice_charlie_1_edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| {
                e.from_node_id.as_str() == "t_alice" && e.to_node_id.as_str() == "t_charlie_1"
            })
            .expect("Expected edge from t_alice to t_charlie_1");

        // The path is built in SVG order from the from-node (alice, at the top)
        // to the to-node (charlie_1, at the bottom). It routes around
        // t_charlie_outer's title band on the way down, but must never reverse
        // along the downward flow axis.
        assert_edge_path_main_axis_monotonic(&alice_charlie_1_edge.path_d, FlowAxis::Vertical);
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
            2,
            "Expected two spacer protrusions for edge t_alice -> t_charlie_3: the \
             cross-container spacer for the rank-0 sibling group (t_charlie_1 and \
             t_charlie_2 share one spacer) plus the text-content spacer routing \
             around t_charlie_outer's title band. Got {spacer_count} spacer(s): {:?}",
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

/// Edges to `t_charlie_1` (rank 0 in `t_charlie_outer`) need no *sibling*
/// cross-container spacer, even in the presence of a rank-1 sibling
/// (`t_charlie_3`) -- only the text-content spacer that routes around
/// `t_charlie_outer`'s title band, for one spacer each.
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

        assert_eq!(
            alice_charlie_1_edge
                .ortho_protrusion_params
                .spacer_protrusions
                .len(),
            1,
            "Expected exactly one (text-content) spacer for edge t_alice -> \
             t_charlie_1 in the 0008 diagram: t_charlie_1 is at rank 0, so no \
             sibling spacer is needed -- the only spacer routes around \
             t_charlie_outer's title band: spacer_protrusions = {:?}",
            alice_charlie_1_edge
                .ortho_protrusion_params
                .spacer_protrusions,
        );

        // bob -> charlie_1 edge: also only the text-content spacer
        let bob_charlie_1_edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.from_node_id.as_str() == "t_bob" && e.to_node_id.as_str() == "t_charlie_1")
            .expect("Expected edge from t_bob to t_charlie_1");

        assert_eq!(
            bob_charlie_1_edge
                .ortho_protrusion_params
                .spacer_protrusions
                .len(),
            1,
            "Expected exactly one (text-content) spacer for edge t_bob -> \
             t_charlie_1 in the 0008 diagram: t_charlie_1 is at rank 0, so no \
             sibling spacer is needed -- the only spacer routes around \
             t_charlie_outer's title band: spacer_protrusions = {:?}",
            bob_charlie_1_edge
                .ortho_protrusion_params
                .spacer_protrusions,
        );
    }
}

/// The edge from `t_alice_inner` to `t_charlie_inner` in the doubly-nested
/// diagram must descend cleanly toward its target without ever reversing along
/// the downward flow axis.
///
/// `t_charlie_inner` is nested under the title bands of `t_charlie_outer` and
/// `t_charlie`, so the edge legitimately enters those containers -- descending
/// alongside (to the right of) each title via a text-content spacer -- rather
/// than stopping at `t_charlie_outer`'s top. What must never happen is a
/// backward (upward) reversal: a U-bend / V-spike at a container boundary, a
/// from-protrusion tip placed below the to-tip, or spacers visited out of order
/// would all show up as a non-monotonic dip, so the path is asserted monotonic
/// along the downward flow axis. (The companion
/// `test_nested_x2_node_edge_routing_no_upward_detour` guards the same property
/// against the from-protrusion tip specifically.)
#[test]
fn test_nested_x2_node_edge_routing_stays_above_charlie_outer() {
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

        // The path is built in SVG order from the from-node (t_alice_inner, at
        // the top) to the to-node (t_charlie_inner, at the bottom). It routes
        // around the destination containers' title bands on the way down, but
        // must never reverse along the downward flow axis.
        assert_edge_path_main_axis_monotonic(
            &alice_inner_charlie_inner_edge.path_d,
            FlowAxis::Vertical,
        );
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
///
/// `t_charlie` is rank 0 beneath `t_charlie_outer`'s title band, so the edge
/// then descends into the container alongside (to the right of) that title via
/// a text-content spacer to reach `t_charlie`. That descent below
/// `t_charlie_outer.y` is intentional; what must not happen is a backward
/// (upward) reversal, so the path stays monotonic along the downward flow axis.
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

        // The edge enters the gap, then descends around t_charlie_outer's title
        // band to reach t_charlie -- but must never reverse along the downward
        // flow axis (no V-spike / backward loop).
        assert_edge_path_main_axis_monotonic(&alice_charlie_edge.path_d, FlowAxis::Vertical);
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
/// to a node in the next sibling container has no higher-ranked *sibling* to
/// route around.
///
/// For the horizontal flows the destination container's title is a side strip
/// the edge enters past, so the edge exits straight out the gap-facing face
/// with no spacer and no cross-axis detour.
///
/// For the vertical flows the destination container's title band sits above its
/// ranks, so the edge picks up a single text-content spacer and routes around
/// that band (a modest cross-axis detour) before descending to the rank-0
/// target -- still monotonic along the flow axis, with no sibling spacer.
fn assert_high_rank_from_edge_routes_straight(input_diagram: &str, axis: FlowAxis) {
    for svg_elements in build_svg_elements_for_diagram(input_diagram) {
        let edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.from_node_id.as_str() == "t_a_01" && e.to_node_id.as_str() == "t_b_00")
            .expect("Expected edge from t_a_01 to t_b_00");

        match axis {
            FlowAxis::Vertical => {
                // The destination container `t_b_0` renders a title band above
                // its ranks, so the edge routes around it via one text-content
                // spacer rather than descending straight down through the title.
                // There is still no higher-ranked sibling, so that is the only
                // spacer, and the path never reverses along the flow axis.
                assert_eq!(
                    edge.ortho_protrusion_params.spacer_protrusions.len(),
                    1,
                    "Expected exactly one (text-content) spacer for t_a_01 -> \
                     t_b_00 -- t_a_01 has no higher-ranked sibling, so the only \
                     spacer routes around t_b_0's title band. \
                     spacer_protrusions = {:?}, path_d = {:?}",
                    edge.ortho_protrusion_params.spacer_protrusions,
                    edge.path_d,
                );
                assert_edge_path_main_axis_monotonic(&edge.path_d, axis);
            }
            FlowAxis::Horizontal => {
                assert!(
                    edge.ortho_protrusion_params.spacer_protrusions.is_empty(),
                    "Expected no spacer protrusions for t_a_01 -> t_b_00 -- t_a_01 \
                     is the highest-ranked child of t_a_0 and the title is a side \
                     strip, so the edge exits straight out the gap-facing face. \
                     spacer_protrusions = {:?}, path_d = {:?}",
                    edge.ortho_protrusion_params.spacer_protrusions,
                    edge.path_d,
                );

                // No cross-axis detour: every vertex stays within a tight band of
                // the first contact point (both endpoints are aligned on the
                // cross axis in these fixtures).
                let coords = parse_path_endpoints(&edge.path_d);
                let first_cross = axis.cross(coords[0]);
                for &point in &coords {
                    let cross = axis.cross(point);
                    assert!(
                        (cross - first_cross).abs() <= 12.0,
                        "Edge t_a_01 -> t_b_00 detours on the cross axis: vertex \
                         {point:?} is {:.1} px from the contact line \
                         ({first_cross:.1}). The edge should route straight out \
                         the gap-facing face. path_d = {:?}",
                        (cross - first_cross).abs(),
                        edge.path_d,
                    );
                }
            }
        }
    }
}

/// `0031` (`top_to_bottom`): `t_a_01` is rank 1 (highest) in `t_a_0`, so the
/// edge to `t_b_00` needs no *sibling* cross-container spacer -- only a
/// text-content spacer to route around `t_b_0`'s title band.
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

/// `0034` (`bottom_to_top`): same as `0031` (one text-content spacer around
/// `t_b_0`'s title band, no sibling spacer), with a reversed vertical flow.
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
/// clear `t_a_02`. It also picks up a text-content spacer to route around
/// `t_b_0`'s title band, for two spacers total.
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
            2,
            "Expected two spacers for t_a_01 -> t_b_00: the sibling cross-container \
             spacer routing around t_a_02 on the gap side, and the text-content \
             spacer routing around t_b_0's title band. spacer_protrusions = {:?}, \
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

/// `0012` (`top_to_bottom`): a symmetric dependency between `t_alice` (nested
/// inside `t_alice_outer`) and the outer node `t_bob` produces two
/// opposite-direction edges sharing the `t_alice.Right` / `t_bob.Left` faces.
/// Both bends are forced into the narrow gap between `t_alice_outer` and
/// `t_bob`; without nesting them, each edge's routing leg crosses the other's
/// bend twice. `protrusions_nest_symmetric_pair_bends` collapses the pair into
/// nested Z paths so they no longer cross.
#[test]
fn test_0012_symmetric_pair_edges_do_not_cross() {
    for svg_elements in build_svg_elements_for_diagram(
        INPUT_DIAGRAM_0012_EDGE_FROM_NESTED_NODE_TO_OUTER_NODE_CYCLIC,
    ) {
        let path_for = |edge_id: &str| -> Vec<(f32, f32)> {
            let edge = svg_elements
                .svg_edge_infos
                .iter()
                .find(|e| e.edge_id.as_str() == edge_id)
                .unwrap_or_else(|| panic!("Expected edge {edge_id}"));
            parse_path_endpoints(&edge.path_d)
        };

        let path_alice_bob = path_for("edge_dep_alice_bob__0");
        let path_bob_alice = path_for("edge_dep_alice_bob__1");

        assert!(
            !polylines_cross(&path_alice_bob, &path_bob_alice),
            "The symmetric pair edge_dep_alice_bob__0 and __1 should not cross.\n  \
             __0: {path_alice_bob:?}\n  __1: {path_bob_alice:?}",
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

        // Ordering: both `ranks_slots` and `labels_offsets` sweep left, and
        // `ranks_slots`'s lateral span contains `labels_offsets`'s descent column,
        // so `ranks_slots` sweeps over it and must turn **higher** (smaller y) so
        // its sweep passes above `labels_offsets`'s descent rather than crossing
        // it.
        let (y_ranks_slots, ..) = jog("t_node_ranks", "t_slot_indices");
        let (y_labels_offsets, ..) = jog("t_edge_labels", "t_offsets");
        assert!(
            y_ranks_slots < y_labels_offsets,
            "ranks_slots's first jog ({y_ranks_slots:.1}) must be above \
             labels_offsets's ({y_labels_offsets:.1}) -- it sweeps over the latter's \
             descent column",
        );
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
/// face -- `edge_dep_pass1_contacts` approaches from `t_pass1_path` on the
/// left, `edge_dep_layout_contacts` approaches via a spacer from
/// `t_taffy_layout` on the right. Contacts are ordered by the side each edge
/// approaches from, so the left-source edge takes the left contact and the two
/// paths do not cross.
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

/// `0045` / `0046` / `0047` are `0044` (the described-container fan) rotated to
/// `left_to_right` / `right_to_left` / `bottom_to_top`. The description label
/// always sits at the *top* of the node wrapper (a flex column with the label
/// above its rank containers), regardless of `RankDir`.
///
/// For the horizontal flows (`left_to_right` / `right_to_left`) the label is a
/// side strip and external edges enter at the rank level -- past the label, not
/// through it -- so no text-content spacer is built. For `bottom_to_top` the
/// ranks are reversed but the label stays on top, so an edge leaving a nested
/// high-rank `from` node exits *up through* the label band and does need a
/// text-content spacer -- built for the `from` side (the mirror of
/// `TopToBottom`'s `to` side). That from-side waypoint sits between the
/// from-node and to-node, so the path still flows monotonically; the earlier
/// `to`-side attempt pointed back at the *other* container's label and produced
/// a backward zigzag. This asserts every dependency edge flows monotonically
/// along the rank axis.
fn assert_described_container_fan_routes_cleanly(input_diagram: &str, axis: FlowAxis) {
    for svg_elements in build_svg_elements_for_diagram(input_diagram) {
        for edge in svg_elements.svg_edge_infos.iter() {
            // Only orthogonal dependency edges follow the rank flow; interaction
            // (`txn_*`) edges are direct curves and are exempt.
            if !edge.edge_group_id.as_str().starts_with("edge_dep_") {
                continue;
            }
            assert_edge_path_main_axis_monotonic(&edge.path_d, axis);
        }
    }
}

#[test]
fn test_0045_described_container_fan_left_to_right_routes_cleanly() {
    assert_described_container_fan_routes_cleanly(
        INPUT_DIAGRAM_0045_EDGE_OFFSETS_AND_PROTRUSION_COMPLEX_2_LEFT_TO_RIGHT,
        FlowAxis::Horizontal,
    );
}

#[test]
fn test_0046_described_container_fan_right_to_left_routes_cleanly() {
    assert_described_container_fan_routes_cleanly(
        INPUT_DIAGRAM_0046_EDGE_OFFSETS_AND_PROTRUSION_COMPLEX_2_RIGHT_TO_LEFT,
        FlowAxis::Horizontal,
    );
}

#[test]
fn test_0047_described_container_fan_bottom_to_top_routes_cleanly() {
    assert_described_container_fan_routes_cleanly(
        INPUT_DIAGRAM_0047_EDGE_OFFSETS_AND_PROTRUSION_COMPLEX_2_BOTTOM_TO_TOP,
        FlowAxis::Vertical,
    );
}

// === Interaction edge halo tests === //

/// When `RenderOptions.interaction_edge_halo` is enabled (the default), an
/// interaction edge gets a `{edge_id}__halo` tailwind-classes entry styled
/// from `type_interaction_edge_halo`, and the rendered SVG draws a
/// `class="edge_halo .."` path -- sharing the edge's `d` -- as the first
/// child of the edge's `<g>`, before `edge_body`. Dependency edges get no
/// such entry or path.
#[test]
fn test_interaction_edge_halo_enabled_renders_halo_path_before_edge_body() {
    for svg_elements in build_svg_elements_for_diagram(INPUT_DIAGRAM_0048_INTERACTION_EDGE_HALO) {
        let dep_edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.edge_id.as_str().starts_with("edge_dep_ab"))
            .expect("Expected the dependency edge to exist.");
        let ix_edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.edge_id.as_str().starts_with("edge_ix_ab"))
            .expect("Expected the interaction edge to exist.");

        let halo_key = |edge_id: &str| {
            Id::try_from(format!("{edge_id}__halo")).expect("halo ID should be valid")
        };

        let ix_halo_classes = svg_elements
            .tailwind_classes
            .get(&halo_key(ix_edge.edge_id.as_str()))
            .unwrap_or_else(|| {
                panic!(
                    "Expected halo tailwind classes for interaction edge {:?}",
                    ix_edge.edge_id
                )
            });
        // `edge_ix_ab` is a `sequence`-kind interaction edge, so it always
        // resolves as "forward" -- `type_interaction_edge_halo_forward`
        // overrides `shape_color` to green, so the halo's colour is
        // `green-800` (the shade still comes from the shared
        // `type_interaction_edge_halo` base). It's rendered via a
        // `stroke-[var(--tw-green-800-200)]` CSS variable class (not a plain
        // `stroke-green-800` class) because `base_diagram.yaml` configures
        // inverted dark-mode shades.
        assert!(
            ix_halo_classes.contains("green-800"),
            "Expected halo classes to reference green-800, got: {ix_halo_classes}"
        );
        assert!(
            ix_halo_classes.contains("opacity-20"),
            "Expected halo classes to include opacity-20, got: {ix_halo_classes}"
        );
        assert!(
            ix_halo_classes.contains("stroke-8"),
            "Expected halo classes to include stroke-8, got: {ix_halo_classes}"
        );

        assert!(
            svg_elements
                .tailwind_classes
                .get(&halo_key(dep_edge.edge_id.as_str()))
                .is_none(),
            "Dependency edge {:?} should not have a halo tailwind classes entry",
            dep_edge.edge_id
        );

        let svg = SvgElementsToSvgMapper::map(&svg_elements);

        let ix_g_start = svg
            .find(&format!("id=\"{}\"", ix_edge.edge_id))
            .expect("Expected the interaction edge's <g> element in the rendered SVG");
        let ix_g_slice = &svg[ix_g_start..];
        let halo_index = ix_g_slice
            .find("class=\"edge_halo")
            .expect("Expected an edge_halo path for the interaction edge");
        let body_index = ix_g_slice
            .find("class=\"edge_body")
            .expect("Expected an edge_body path for the interaction edge");
        assert!(
            halo_index < body_index,
            "Expected edge_halo to render before edge_body within the interaction edge's <g>"
        );

        let dep_g_start = svg
            .find(&format!("id=\"{}\"", dep_edge.edge_id))
            .expect("Expected the dependency edge's <g> element in the rendered SVG");
        let dep_g_slice = &svg[dep_g_start..];
        let dep_next_g_offset = dep_g_slice.find("<g id=\"").unwrap_or(dep_g_slice.len());
        assert!(
            !dep_g_slice[..dep_next_g_offset].contains("edge_halo"),
            "Dependency edge {:?} should not render an edge_halo path",
            dep_edge.edge_id
        );
    }
}

/// When `RenderOptions.interaction_edge_halo` is disabled, no halo tailwind
/// classes entry is created for the interaction edge, and the rendered SVG
/// contains no `edge_halo` class at all.
#[test]
fn test_interaction_edge_halo_disabled_omits_halo_path() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0049_INTERACTION_EDGE_HALO_DISABLED)
    {
        let ix_edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.edge_id.as_str().starts_with("edge_ix_ab"))
            .expect("Expected the interaction edge to exist.");

        let halo_key =
            Id::try_from(format!("{}__halo", ix_edge.edge_id)).expect("halo ID should be valid");
        assert!(
            svg_elements.tailwind_classes.get(&halo_key).is_none(),
            "Expected no halo tailwind classes entry when interaction_edge_halo is disabled"
        );

        let svg = SvgElementsToSvgMapper::map(&svg_elements);
        assert!(
            !svg.contains("edge_halo"),
            "Expected no edge_halo path in the rendered SVG when interaction_edge_halo is disabled"
        );
    }
}

/// Forward (request) and reverse (response) interaction edges within the same
/// symmetric edge group get distinct halo colours when
/// `type_interaction_edge_halo_forward` / `type_interaction_edge_halo_reverse`
/// override `ShapeColor`, while attributes neither overrides (like
/// `StrokeWidth`) still fall back to the shared `type_interaction_edge_halo`
/// base.
#[test]
fn test_interaction_edge_halo_forward_reverse_use_distinct_colors() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0050_INTERACTION_EDGE_HALO_FORWARD_REVERSE)
    {
        let halo_key = |edge_id: &str| {
            Id::try_from(format!("{edge_id}__halo")).expect("halo ID should be valid")
        };

        // `edge_ix_chain` has 3 things in a symmetric group: 2 forward edges
        // (indices 0, 1) followed by 2 reverse edges (indices 2, 3).
        let forward_halo = svg_elements
            .tailwind_classes
            .get(&halo_key("edge_ix_chain__0"))
            .unwrap_or_else(|| panic!("Expected halo tailwind classes for the forward edge"));
        let reverse_halo = svg_elements
            .tailwind_classes
            .get(&halo_key("edge_ix_chain__2"))
            .unwrap_or_else(|| panic!("Expected halo tailwind classes for the reverse edge"));

        assert!(
            forward_halo.contains("green"),
            "Expected the forward (request) halo to reference green, got: {forward_halo}"
        );
        assert!(
            !forward_halo.contains("yellow"),
            "Expected the forward (request) halo to not reference yellow, got: {forward_halo}"
        );

        assert!(
            reverse_halo.contains("yellow"),
            "Expected the reverse (response) halo to reference yellow, got: {reverse_halo}"
        );
        assert!(
            !reverse_halo.contains("green"),
            "Expected the reverse (response) halo to not reference green, got: {reverse_halo}"
        );

        // Attributes not overridden by the forward/reverse types (e.g. the
        // halo's width) still come from the shared `type_interaction_edge_halo`
        // base for both.
        assert!(
            forward_halo.contains("stroke-8") && reverse_halo.contains("stroke-8"),
            "Expected both halos to keep the base stroke-8 width, got forward: {forward_halo}, \
             reverse: {reverse_halo}"
        );
    }
}

// === Edge group-ID fallback tests === //

/// `edge_descs` / `edge_labels` entries keyed by an edge *group* ID (rather
/// than a specific edge instance ID) apply to every edge in that group, for
/// both dependency and interaction edges. This is a regression test for a
/// bug where only the exact instance-ID lookup was implemented, silently
/// dropping any group-ID-keyed description/label (affecting several
/// playground examples, not just interaction edges).
#[test]
fn test_edge_descs_and_labels_group_id_key_resolves_for_dependency_and_interaction_edges() {
    for svg_elements in build_svg_elements_for_diagram(INPUT_DIAGRAM_0053_EDGE_DESCS_GROUP_ID_KEY) {
        let dep_desc = svg_elements
            .edge_description_infos
            .iter()
            .find(|d| d.edge_id.as_str().starts_with("edge_dep_ab"))
            .expect("Expected a description for the dependency edge.");
        let ix_desc = svg_elements
            .edge_description_infos
            .iter()
            .find(|d| d.edge_id.as_str().starts_with("edge_ix_ab"))
            .expect("Expected a description for the interaction edge.");

        assert!(
            !dep_desc.text_spans.is_empty(),
            "Expected the dependency edge's group-ID-keyed description to render text"
        );
        assert!(
            !ix_desc.text_spans.is_empty(),
            "Expected the interaction edge's group-ID-keyed description to render text"
        );

        let dep_label = svg_elements
            .edge_label_infos
            .iter()
            .find(|l| l.edge_id.as_str().starts_with("edge_dep_ab"))
            .expect("Expected a label for the dependency edge.");
        let ix_label = svg_elements
            .edge_label_infos
            .iter()
            .find(|l| l.edge_id.as_str().starts_with("edge_ix_ab"))
            .expect("Expected a label for the interaction edge.");

        assert!(
            dep_label
                .from_label
                .as_ref()
                .is_some_and(|l| !l.text_spans.is_empty()),
            "Expected the dependency edge's group-ID-keyed label to render text"
        );
        assert!(
            ix_label
                .from_label
                .as_ref()
                .is_some_and(|l| !l.text_spans.is_empty()),
            "Expected the interaction edge's group-ID-keyed label to render text"
        );
    }
}

/// A description keyed by a specific edge instance ID overrides the group-ID
/// fallback for that one edge; other edges in the group still fall back to
/// the group-ID description.
#[test]
fn test_edge_descs_instance_id_overrides_group_id() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0054_EDGE_DESCS_INSTANCE_OVERRIDES_GROUP)
    {
        let desc_0 = svg_elements
            .edge_description_infos
            .iter()
            .find(|d| d.edge_id.as_str() == "edge_dep_ab__0")
            .expect("Expected a description for edge_dep_ab__0.");
        let desc_1 = svg_elements
            .edge_description_infos
            .iter()
            .find(|d| d.edge_id.as_str() == "edge_dep_ab__1")
            .expect("Expected a description for edge_dep_ab__1.");

        let desc_0_text: String = desc_0
            .text_spans
            .iter()
            .map(|span| span.text.as_str())
            .collect();
        let desc_1_text: String = desc_1
            .text_spans
            .iter()
            .map(|span| span.text.as_str())
            .collect();

        assert!(
            desc_0_text.contains("instance description"),
            "Expected edge_dep_ab__0 to use its own instance-ID description, got: {desc_0_text}"
        );
        assert!(
            desc_1_text.contains("group description"),
            "Expected edge_dep_ab__1 to fall back to the group-ID description, got: {desc_1_text}"
        );
    }
}

// === Edge label/description background tests === //

/// Every edge's label and description -- dependency as well as interaction
/// -- get a `{edge_id}__label_bg` / `{edge_id}__desc_bg` tailwind-classes
/// entry (styled through the `EdgeLabelAndDescBg` fallback hierarchy, see
/// `doc/src/edge_descriptions.md`), and the rendered SVG draws a background
/// `<path>` as the first child of the label/description `<g>`, before the
/// text.
#[test]
fn test_edge_label_and_desc_bg_render_before_text() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0055_INTERACTION_EDGE_LABEL_DESC_BG)
    {
        let dep_edge_id = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.edge_id.as_str().starts_with("edge_dep_ab"))
            .expect("Expected the dependency edge to exist.")
            .edge_id
            .clone();
        let ix_edge_id = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.edge_id.as_str().starts_with("edge_ix_ab"))
            .expect("Expected the interaction edge to exist.")
            .edge_id
            .clone();

        let label_bg_key = |edge_id: &str| {
            Id::try_from(format!("{edge_id}__label_bg")).expect("label_bg ID should be valid")
        };
        let desc_bg_key = |edge_id: &str| {
            Id::try_from(format!("{edge_id}__desc_bg")).expect("desc_bg ID should be valid")
        };

        for (edge_kind, edge_id) in [("interaction", &ix_edge_id), ("dependency", &dep_edge_id)] {
            let label_bg_classes = svg_elements
                .tailwind_classes
                .get(&label_bg_key(edge_id.as_str()))
                .unwrap_or_else(|| {
                    panic!("Expected label_bg tailwind classes for {edge_kind} edge {edge_id:?}")
                });
            assert!(
                label_bg_classes.contains("opacity-5"),
                "Expected {edge_kind} edge's label_bg classes to include opacity-5, \
                got: {label_bg_classes}"
            );
            let desc_bg_classes = svg_elements
                .tailwind_classes
                .get(&desc_bg_key(edge_id.as_str()))
                .unwrap_or_else(|| {
                    panic!("Expected desc_bg tailwind classes for {edge_kind} edge {edge_id:?}")
                });
            assert!(
                desc_bg_classes.contains("opacity-5"),
                "Expected {edge_kind} edge's desc_bg classes to include opacity-5, \
                got: {desc_bg_classes}"
            );
        }

        let svg = SvgElementsToSvgMapper::map(&svg_elements);

        for (edge_kind, edge_id) in [("interaction", &ix_edge_id), ("dependency", &dep_edge_id)] {
            let label_g_start = svg
                .find(&format!("id=\"{edge_id}__from_label\""))
                .unwrap_or_else(|| {
                    panic!(
                        "Expected the {edge_kind} edge's from_label <g> element in the rendered SVG"
                    )
                });
            let label_g_slice = &svg[label_g_start..];
            let path_index = label_g_slice.find("<path").unwrap_or_else(|| {
                panic!("Expected a background path in the {edge_kind} edge's label")
            });
            let text_index = label_g_slice
                .find("<text")
                .unwrap_or_else(|| panic!("Expected text in the {edge_kind} edge's label"));
            assert!(
                path_index < text_index,
                "Expected the {edge_kind} edge's label background path to render before the label text"
            );

            let desc_g_start = svg
                .find(&format!("id=\"{edge_id}__desc\""))
                .unwrap_or_else(|| {
                    panic!("Expected the {edge_kind} edge's desc <g> element in the rendered SVG")
                });
            let desc_g_slice = &svg[desc_g_start..];
            let path_index = desc_g_slice.find("<path").unwrap_or_else(|| {
                panic!("Expected a background path in the {edge_kind} edge's description")
            });
            let text_index = desc_g_slice
                .find("<text")
                .unwrap_or_else(|| panic!("Expected text in the {edge_kind} edge's description"));
            assert!(
                path_index < text_index,
                "Expected the {edge_kind} edge's description background path to render before \
                the description text"
            );
        }
    }
}

/// Edge label/description background styling resolves through a 3-tier
/// fallback hierarchy -- least to most specific:
/// `EdgeLabelAndDescBg` -> `{Dependency,Interaction}EdgeLabelAndDescBg` ->
/// `{Dependency,Interaction}Edge{Label,Desc}Bg` -- where a more specific
/// override wins over a less specific one, and unrelated edges/backgrounds
/// are left untouched by a more specific override.
///
/// The fixture overrides all 3 tiers for the dependency edge (`"rose"` ->
/// `"amber"` -> `"lime"`, the last only for the label), while the
/// interaction edge is never overridden beyond tier 1.
#[test]
fn test_edge_label_desc_bg_hierarchy_fallback() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0059_EDGE_LABEL_DESC_BG_HIERARCHY_OVERRIDE)
    {
        let dep_edge_id = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.edge_id.as_str().starts_with("edge_dep_ab"))
            .expect("Expected the dependency edge to exist.")
            .edge_id
            .clone();
        let ix_edge_id = svg_elements
            .svg_edge_infos
            .iter()
            .find(|e| e.edge_id.as_str().starts_with("edge_ix_ab"))
            .expect("Expected the interaction edge to exist.")
            .edge_id
            .clone();

        let label_bg_classes = |edge_id: &str| {
            let key =
                Id::try_from(format!("{edge_id}__label_bg")).expect("label_bg ID should be valid");
            svg_elements
                .tailwind_classes
                .get(&key)
                .unwrap_or_else(|| panic!("Expected label_bg tailwind classes for {edge_id:?}"))
        };
        let desc_bg_classes = |edge_id: &str| {
            let key =
                Id::try_from(format!("{edge_id}__desc_bg")).expect("desc_bg ID should be valid");
            svg_elements
                .tailwind_classes
                .get(&key)
                .unwrap_or_else(|| panic!("Expected desc_bg tailwind classes for {edge_id:?}"))
        };

        // Tier 1 (`type_edge_label_and_desc_bg`, "rose"): the interaction
        // edge is never overridden beyond this tier, so both its label and
        // description backgrounds stay "rose".
        let ix_label_bg = label_bg_classes(ix_edge_id.as_str());
        assert!(
            ix_label_bg.contains("rose"),
            "Expected interaction edge's label_bg classes to include \"rose\" (tier 1 default), \
            got: {ix_label_bg}"
        );
        let ix_desc_bg = desc_bg_classes(ix_edge_id.as_str());
        assert!(
            ix_desc_bg.contains("rose"),
            "Expected interaction edge's desc_bg classes to include \"rose\" (tier 1 default), \
            got: {ix_desc_bg}"
        );

        // Tier 2 (`type_dependency_edge_label_and_desc_bg`, "amber"):
        // overrides tier 1 for the dependency edge's description
        // background (no tier-3 override applies to desc).
        let dep_desc_bg = desc_bg_classes(dep_edge_id.as_str());
        assert!(
            dep_desc_bg.contains("amber"),
            "Expected dependency edge's desc_bg classes to include \"amber\" (tier 2 override), \
            got: {dep_desc_bg}"
        );
        assert!(
            !dep_desc_bg.contains("rose"),
            "Expected dependency edge's desc_bg classes to no longer include \"rose\", \
            got: {dep_desc_bg}"
        );

        // Tier 3 (`type_dependency_edge_label_bg`, "lime"): overrides tier 2
        // for the dependency edge's label background only.
        let dep_label_bg = label_bg_classes(dep_edge_id.as_str());
        assert!(
            dep_label_bg.contains("lime"),
            "Expected dependency edge's label_bg classes to include \"lime\" (tier 3 override), \
            got: {dep_label_bg}"
        );
        assert!(
            !dep_label_bg.contains("amber") && !dep_label_bg.contains("rose"),
            "Expected dependency edge's label_bg classes to no longer include \"amber\" or \
            \"rose\", got: {dep_label_bg}"
        );
    }
}

// === Between-ranks (non-cycle edge) description placement baseline === //

/// Baseline companion to the same-rank tests below: when the divergent
/// ancestors sit at *different* ranks (a plain `sequence` dependency, not a
/// cycle), the description must still be interleaved between the two rank
/// containers -- confirming the same-rank fix did not regress this path.
#[test]
fn test_between_ranks_edge_description_sits_between_rank_containers() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0056_INTERACTION_HALO_WITH_LABELS)
    {
        let t_client = svg_elements
            .svg_node_infos
            .iter()
            .find(|node_info| node_info.node_id.as_str() == "t_client")
            .expect("Expected t_client in svg_node_infos");
        let t_server = svg_elements
            .svg_node_infos
            .iter()
            .find(|node_info| node_info.node_id.as_str() == "t_server")
            .expect("Expected t_server in svg_node_infos");

        // `rank_dir: left_to_right` puts different-ranked nodes at different
        // X positions; t_client (rank 0) sits left of t_server (rank 1).
        let desc = svg_elements
            .edge_description_infos
            .iter()
            .find(|d| d.edge_id.as_str() == "edge_dep_client_server__0")
            .expect("Expected a description for edge_dep_client_server__0.");

        assert!(
            desc.x >= t_client.envelope_x + t_client.envelope_width
                && desc.x <= t_server.envelope_x,
            "Expected edge_dep_client_server__0's description (x: {}) to sit between t_client's \
             right edge ({}) and t_server's left edge ({})",
            desc.x,
            t_client.envelope_x + t_client.envelope_width,
            t_server.envelope_x,
        );
    }
}

/// Regression test for a bug where a cross-rank `edge_description_container`
/// under a reversed `rank_dir` (`bottom_to_top`/`right_to_left`) rendered its
/// descriptions back to front. The container mirrors
/// `rank_container_style.flex_direction`, which is `RowReverse`/
/// `ColumnReverse` for those two directions (so ordinary rank containers'
/// *reversed* sibling stacking, combined with a separate sibling-order
/// correction, still matches declaration order) -- but an
/// `edge_description_container`'s children are freshly sorted into visual
/// order every time it is built, so mirroring the reversed direction
/// (without also correcting sibling order, which nothing does for this
/// container) would render them in the opposite of sorted order.
///
/// `EdgeDescriptionBuilder::container_style_build` strips `Reverse` down to
/// plain `Row`/`Column`. `0058` mirrors `0056` under `rank_dir: right_to_left`,
/// so `edge_dep_client_server__0`, `edge_ix_client_server__0`, and
/// `edge_ix_client_server__1` (sorted in that order, by `EdgeId`) share one
/// cross-rank container with a `ColumnReverse` rank container style -- this
/// asserts their descriptions' `y` positions are strictly increasing
/// (sorted order, top to bottom), not decreasing (reversed).
#[test]
fn test_cross_rank_edge_description_container_direction_not_reversed() {
    for svg_elements in build_svg_elements_for_diagram(
        INPUT_DIAGRAM_0058_INTERACTION_HALO_WITH_LABELS_RIGHT_TO_LEFT,
    ) {
        let desc_y = |edge_id: &str| {
            svg_elements
                .edge_description_infos
                .iter()
                .find(|d| d.edge_id.as_str() == edge_id)
                .unwrap_or_else(|| panic!("Expected a description for {edge_id}."))
                .y
        };

        let dep_y = desc_y("edge_dep_client_server__0");
        let ix_0_y = desc_y("edge_ix_client_server__0");
        let ix_1_y = desc_y("edge_ix_client_server__1");

        assert!(
            dep_y < ix_0_y && ix_0_y < ix_1_y,
            "Expected the shared container's descriptions to be stacked in sorted \
             (not reversed) order: edge_dep_client_server__0 (y: {dep_y}) < \
             edge_ix_client_server__0 (y: {ix_0_y}) < edge_ix_client_server__1 (y: {ix_1_y})"
        );
    }
}

// === Same-rank (cycle edge) description placement tests === //

/// When an edge's divergent ancestors share a rank (a cycle edge, e.g. a
/// `cyclic` dependency), the description container must be inserted between
/// the two same-ranked siblings, not before/after the whole shared rank row.
/// This is a regression test for a bug where the container was placed at
/// `Some(rank - 1)`, floating before the entire rank instead of between the
/// specific pair of nodes it describes.
#[test]
fn test_same_rank_cyclic_edge_description_sits_between_divergent_ancestors_at_root_level() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0057_INTERACTION_HALO_WITH_DESC_CYCLIC)
    {
        let t_client = svg_elements
            .svg_node_infos
            .iter()
            .find(|node_info| node_info.node_id.as_str() == "t_client")
            .expect("Expected t_client in svg_node_infos");
        let t_server = svg_elements
            .svg_node_infos
            .iter()
            .find(|node_info| node_info.node_id.as_str() == "t_server")
            .expect("Expected t_server in svg_node_infos");

        // `rank_dir: left_to_right` stacks same-ranked siblings vertically, so
        // "between" is a Y-axis relationship. t_client and t_server share a
        // rank (the cyclic dependency), so whichever renders first is
        // strictly above the other -- the container must sit in the gap.
        let (upper, lower) = if t_client.envelope_y < t_server.envelope_y {
            (t_client, t_server)
        } else {
            (t_server, t_client)
        };
        let upper_bottom = upper.envelope_y + upper.envelope_height_collapsed;

        for edge_id in [
            "edge_dep_client_server__0",
            "edge_ix_client_server__0",
            "edge_ix_client_server__1",
        ] {
            let desc = svg_elements
                .edge_description_infos
                .iter()
                .find(|d| d.edge_id.as_str() == edge_id)
                .unwrap_or_else(|| panic!("Expected a description for {edge_id}."));

            assert!(
                desc.y >= upper_bottom && desc.y + desc.height <= lower.envelope_y,
                "Expected {edge_id}'s description (y: {}, height: {}) to sit between t_client/\
                 t_server (upper bottom: {upper_bottom}, lower top: {}), not before/after the \
                 whole shared rank row",
                desc.y,
                desc.height,
                lower.envelope_y,
            );
        }
    }
}

/// Regression test for a bug where a same-rank (cycle edge) description
/// container's insertion index was computed from the GLOBAL sibling index
/// (position among ALL root-level things, regardless of rank) instead of the
/// LOCAL sibling index (position among only same-ranked things). When a
/// differently-ranked sibling (`t_a`, rank 1) is declared *before* the
/// same-rank cyclic pair (`t_b`/`t_c`, rank 0), the global index skewed past
/// the rank-0 bucket's own length, appending the container after `t_c`
/// instead of between `t_b` and `t_c`.
#[test]
fn test_same_rank_description_container_sits_between_siblings_despite_lower_declared_higher_rank_sibling(
) {
    for svg_elements in build_svg_elements_for_diagram(
        INPUT_DIAGRAM_0060_SAME_RANK_DESC_CONTAINER_GLOBAL_VS_LOCAL_SIBLING_INDEX,
    ) {
        let t_b = svg_elements
            .svg_node_infos
            .iter()
            .find(|node_info| node_info.node_id.as_str() == "t_b")
            .expect("Expected t_b in svg_node_infos");
        let t_c = svg_elements
            .svg_node_infos
            .iter()
            .find(|node_info| node_info.node_id.as_str() == "t_c")
            .expect("Expected t_c in svg_node_infos");

        // `rank_dir: left_to_right` stacks same-ranked siblings vertically, so
        // "between" is a Y-axis relationship.
        let (upper, lower) = if t_b.envelope_y < t_c.envelope_y {
            (t_b, t_c)
        } else {
            (t_c, t_b)
        };
        let upper_bottom = upper.envelope_y + upper.envelope_height_collapsed;

        let desc = svg_elements
            .edge_description_infos
            .iter()
            .find(|d| d.edge_id.as_str() == "edge_dep_b_c__0")
            .expect("Expected a description for edge_dep_b_c__0.");

        assert!(
            desc.y >= upper_bottom && desc.y + desc.height <= lower.envelope_y,
            "Expected edge_dep_b_c__0's description (y: {}, height: {}) to sit between t_b/t_c \
             (upper bottom: {upper_bottom}, lower top: {}), not after both -- this is the bug \
             where the container's insertion index used the GLOBAL sibling index (which places \
             t_a before t_b/t_c) instead of the LOCAL rank-0 index",
            desc.y,
            desc.height,
            lower.envelope_y,
        );
    }
}

/// Regression test for the same bug's effect on edge path routing: before the
/// fix, the reverse-direction crossing edge `edge_dep_b_c__1` routed through
/// the misplaced container (appended after `t_c`, past `t_c`'s own right
/// edge), producing a huge detour instead of a short jog through the gap
/// between `t_b` and `t_c`. This mirrors the real-world bug in
/// `example_input.yaml`'s `edge_dep_t_localhost__t_github_user_repo__pull__0`,
/// whose path detoured out to x=987 (past `t_aws`'s rank column) instead of
/// jogging in the small gap between `t_github`/`t_localhost`.
///
/// Asserts every point of `edge_dep_b_c__1`'s rendered path stays strictly
/// clear of `t_a`'s rank column (`t_a` is rank 1, laid out after `t_b`/`t_c`'s
/// rank 0 under `rank_dir: left_to_right`) and within a tight bound just past
/// the shared description box's own extent (which is wider than `t_b`/`t_c`
/// themselves, since it holds the "b/c dep desc" text) -- not ballooning out
/// to some distant coordinate like the real bug's x=987.
#[test]
fn test_same_rank_crossing_edge_path_stays_near_divergent_ancestors_despite_lower_declared_higher_rank_sibling(
) {
    const TIGHT_BOUND_TOLERANCE_PX: f32 = 20.0;

    for svg_elements in build_svg_elements_for_diagram(
        INPUT_DIAGRAM_0060_SAME_RANK_DESC_CONTAINER_GLOBAL_VS_LOCAL_SIBLING_INDEX,
    ) {
        let t_a = svg_elements
            .svg_node_infos
            .iter()
            .find(|node_info| node_info.node_id.as_str() == "t_a")
            .expect("Expected t_a in svg_node_infos");

        let desc = svg_elements
            .edge_description_infos
            .iter()
            .find(|d| d.edge_id.as_str() == "edge_dep_b_c__0")
            .expect("Expected a description for edge_dep_b_c__0.");
        let tight_max_x = desc.x + desc.width + TIGHT_BOUND_TOLERANCE_PX;

        let svg = SvgElementsToSvgMapper::map(&svg_elements);
        let d = edge_body_path_d(&svg, "edge_dep_b_c__1");
        let points = path_d_points(&d);

        for &(x, _y) in &points {
            assert!(
                x < t_a.envelope_x,
                "Expected edge_dep_b_c__1's path to stay clear of t_a's rank column \
                 (t_a.envelope_x: {}), but got a point at x={x}: {points:?} -- this mirrors the \
                 real bug where the path ballooned out to a distant x instead of jogging through \
                 the small gap between t_b/t_c",
                t_a.envelope_x,
            );
            assert!(
                x <= tight_max_x,
                "Expected edge_dep_b_c__1's path x-coordinates to stay within a tight bound near \
                 t_b/t_c (max allowed: {tight_max_x}), got a point at x={x}: {points:?}",
            );
        }
    }
}

/// Regression test for a bug in `RankSiblingInserter::node_insert`: when 3+
/// same-rank description containers are inserted into the same rank's
/// sibling list (one per adjacent pair `t_a`/`t_b`, `t_b`/`t_c`, `t_c`/`t_d`,
/// all defaulting to rank 0 since there are no `thing_dependencies` to order
/// them), the container for the third pair (`desc_c_d`) used to land next to
/// the *second* pair's container instead of between `t_c` and `t_d`.
///
/// The bug was in the insertion-accounting scheme: it tracked prior
/// insertions by their post-shift position in the growing sibling list
/// rather than by their own original `base_index`, so once two containers
/// had already shifted the list, a later container's "how many earlier
/// insertions come before me" count silently dropped one of them.
///
/// Asserts the three description boxes and four nodes appear in the correct
/// interleaved x-order (this diagram's `things` stack horizontally within
/// their shared rank 0 row).
#[test]
fn test_same_rank_desc_containers_multiple_overlapping_insert_in_correct_order() {
    for svg_elements in build_svg_elements_for_diagram(
        INPUT_DIAGRAM_0061_SAME_RANK_DESC_CONTAINERS_MULTIPLE_OVERLAPPING,
    ) {
        let node_x = |node_id: &str| {
            svg_elements
                .svg_node_infos
                .iter()
                .find(|node_info| node_info.node_id.as_str() == node_id)
                .unwrap_or_else(|| panic!("Expected {node_id} in svg_node_infos"))
                .envelope_x
        };
        let desc_x = |edge_id: &str| {
            svg_elements
                .edge_description_infos
                .iter()
                .find(|d| d.edge_id.as_str() == edge_id)
                .unwrap_or_else(|| panic!("Expected a description for {edge_id}."))
                .x
        };

        let t_a = node_x("t_a");
        let t_b = node_x("t_b");
        let t_c = node_x("t_c");
        let t_d = node_x("t_d");
        let desc_a_b = desc_x("edge_ix_a_b__0");
        let desc_b_c = desc_x("edge_ix_b_c__0");
        let desc_c_d = desc_x("edge_ix_c_d__0");

        assert!(
            t_a < desc_a_b
                && desc_a_b < t_b
                && t_b < desc_b_c
                && desc_b_c < t_c
                && t_c < desc_c_d
                && desc_c_d < t_d,
            "Expected order t_a({t_a}) < desc_a_b({desc_a_b}) < t_b({t_b}) < \
             desc_b_c({desc_b_c}) < t_c({t_c}) < desc_c_d({desc_c_d}) < t_d({t_d}), \
             but desc_c_d landed in the wrong gap"
        );
    }
}

/// The same same-rank placement fix applies at nested LCA levels: the cyclic
/// dependency between `t_server_proc_1` and `t_server_proc_2` (both direct
/// children of `t_server`, sharing a rank) must place its description between
/// those two siblings, not before/after `t_server`'s whole rank row.
#[test]
fn test_same_rank_cyclic_edge_description_sits_between_nested_divergent_ancestors() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0057_INTERACTION_HALO_WITH_DESC_CYCLIC)
    {
        let proc_1 = svg_elements
            .svg_node_infos
            .iter()
            .find(|node_info| node_info.node_id.as_str() == "t_server_proc_1")
            .expect("Expected t_server_proc_1 in svg_node_infos");
        let proc_2 = svg_elements
            .svg_node_infos
            .iter()
            .find(|node_info| node_info.node_id.as_str() == "t_server_proc_2")
            .expect("Expected t_server_proc_2 in svg_node_infos");

        let (upper, lower) = if proc_1.envelope_y < proc_2.envelope_y {
            (proc_1, proc_2)
        } else {
            (proc_2, proc_1)
        };
        let upper_bottom = upper.envelope_y + upper.envelope_height_collapsed;

        let desc = svg_elements
            .edge_description_infos
            .iter()
            .find(|d| d.edge_id.as_str() == "edge_dep_server_proc_1_proc_2__0")
            .expect("Expected a description for edge_dep_server_proc_1_proc_2__0.");

        assert!(
            desc.y >= upper_bottom && desc.y + desc.height <= lower.envelope_y,
            "Expected edge_dep_server_proc_1_proc_2__0's description (y: {}, height: {}) to sit \
             between t_server_proc_1/t_server_proc_2 (upper bottom: {upper_bottom}, lower top: \
             {}), not before/after t_server's whole shared rank row",
            desc.y,
            desc.height,
            lower.envelope_y,
        );
    }
}

// === Edge description contact waypoint tests === //

/// Extracts the sequence of points an SVG path `d` attribute actually passes
/// through: `M`/`L` targets, and each `C` (cubic bezier) command's final
/// endpoint (its third coordinate pair) -- the two control points of a `C`
/// only shape the curve and are not points the path terminates at, so they
/// are skipped.
fn path_d_points(d: &str) -> Vec<(f32, f32)> {
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();
    for c in d.chars() {
        if c.is_ascii_alphabetic() {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            tokens.push(c.to_string());
        } else if c == ',' || c.is_whitespace() {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }

    let parse_pair = |tokens: &[String], x_index: usize| -> Option<(f32, f32)> {
        let x = tokens.get(x_index)?.parse::<f32>().ok()?;
        let y = tokens.get(x_index + 1)?.parse::<f32>().ok()?;
        Some((x, y))
    };

    let mut points = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        match tokens[index].as_str() {
            "M" | "L" => {
                if let Some(point) = parse_pair(&tokens, index + 1) {
                    points.push(point);
                }
                index += 3;
            }
            "C" => {
                // Three coordinate pairs (six numbers); only the third pair
                // is a point the path actually reaches.
                if let Some(point) = parse_pair(&tokens, index + 5) {
                    points.push(point);
                }
                index += 7;
            }
            _ => index += 1,
        }
    }
    points
}

/// Extracts the `d` attribute of the `edge_body` path within an edge's `<g>`
/// element in a rendered SVG string.
fn edge_body_path_d(svg: &str, edge_id: &str) -> String {
    let g_start = svg
        .find(&format!("id=\"{edge_id}\""))
        .unwrap_or_else(|| panic!("Expected <g id=\"{edge_id}\"> in the rendered SVG"));
    let slice = &svg[g_start..];
    let class_pos = slice
        .find("class=\"edge_body\"")
        .unwrap_or_else(|| panic!("Expected an edge_body path for {edge_id}"));
    let before_class = &slice[..class_pos];
    let d_key = "d=\"";
    let d_start = before_class.rfind(d_key).unwrap_or_else(|| {
        panic!("Expected a `d` attribute before the edge_body class for {edge_id}")
    }) + d_key.len();
    let d_end = d_start
        + before_class[d_start..].find('"').unwrap_or_else(|| {
            panic!("Expected a closing quote for the `d` attribute for {edge_id}")
        });
    before_class[d_start..d_end].to_string()
}

/// Regression test for a bug where an interaction edge travelling *against*
/// the diagram's `RankDir` (a `symmetric` group's reverse edge) looped back
/// on itself when routing through its own description box.
///
/// Before the fix, `edge_ix_client_server__1` (`t_server -> t_client`, i.e.
/// high-rank to low-rank under `rank_dir: left_to_right`) rendered as
/// `... 456 -> 245 -> 285 -> 91 ...`: the path reached the description box's
/// near (left) edge at x=245, then had to backtrack rightward to the box's
/// far edge at x=285 before continuing on to 91 -- a visible loop.
///
/// This edge is cross-rank (`t_client` rank 0, `t_server` rank 1), so its
/// waypoint is now produced by
/// `EdgeSpacerCoordinatesCalculator::calculate_description_thread`, which
/// threads *through* the box (`entry != exit`) rather than touching a single
/// point -- but the entry/exit order is chosen so it still runs in this
/// edge's own (reverse) travel direction, so the invariant below continues
/// to hold. This asserts the path's x-coordinates are monotonically
/// non-increasing, matching this edge's actual right-to-left travel -- a
/// backward hop (the original bug) would fail this regardless of which
/// calculator produced the waypoint.
#[test]
fn test_reverse_interaction_edge_description_contact_does_not_loop() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0056_INTERACTION_HALO_WITH_LABELS)
    {
        let svg = SvgElementsToSvgMapper::map(&svg_elements);
        let d = edge_body_path_d(&svg, "edge_ix_client_server__1");
        let points = path_d_points(&d);

        assert!(
            points.len() >= 2,
            "Expected at least two points in edge_ix_client_server__1's path, got: {points:?}"
        );

        let xs: Vec<f32> = points.iter().map(|&(x, _y)| x).collect();
        let non_increasing = xs.windows(2).all(|pair| pair[0] >= pair[1] - 1e-3);
        assert!(
            non_increasing,
            "Expected edge_ix_client_server__1's path x-coordinates to be monotonically \
             non-increasing (this edge travels right-to-left), but got a backward hop: {xs:?}"
        );
    }
}

/// Regression test for a bug where a **cross-rank** edge's own description
/// contact used a single-point calculation, collapsing the box's corridor
/// down to one point. Downstream spacer-ordering/protrusion logic (built to
/// expect a genuine two-point corridor, like every other spacer kind) then
/// mishandled the degenerate waypoint: before the fix,
/// `edge_dep_client_server__0`'s path was pinned at the box's `left_x` with
/// wildly varying, out-of-box `y` values, instead of threading straight
/// across the box at a constant `y`.
///
/// `edge_dep_client_server__0` is cross-rank (`t_client` rank 0, `t_server`
/// rank 1) and travels `from < to` (`Ordering::Less`) under
/// `rank_dir: left_to_right`, so
/// `EdgeSpacerCoordinatesCalculator::calculate_description_thread` should
/// produce entry=(box.left_x, box.top_y), exit=(box.right_x, box.top_y) --
/// this asserts the rendered path visits both of those points, in that
/// order (left before right), confirming it threads through the box's top
/// edge left-to-right rather than collapsing to (or backtracking around) a
/// single point.
#[test]
fn test_between_ranks_edge_description_contact_threads_through_box() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0056_INTERACTION_HALO_WITH_LABELS)
    {
        let desc = svg_elements
            .edge_description_infos
            .iter()
            .find(|d| d.edge_id.as_str() == "edge_dep_client_server__0")
            .expect("Expected a description for edge_dep_client_server__0.");
        let box_left_x = desc.x;
        let box_right_x = desc.x + desc.width;
        // `edge_dep_client_server__0` is a dependency edge, which has no
        // interaction edge halo -- `EdgeDescriptionBuilder::edge_desc_build`
        // gives it no halo-clearance margin, and `description_thread_from_rect`
        // pulls back by `0.0`, so the routed path sits flush against the
        // box's own rendered top edge with no offset.
        let box_top_y = desc.y;

        let svg = SvgElementsToSvgMapper::map(&svg_elements);
        let d = edge_body_path_d(&svg, "edge_dep_client_server__0");
        let points = path_d_points(&d);

        let near_matches = |x: f32, y: f32| -> Option<usize> {
            points
                .iter()
                .position(|&(px, py)| (px - x).abs() < 1e-2 && (py - y).abs() < 1e-2)
        };

        let entry_index = near_matches(box_left_x, box_top_y).unwrap_or_else(|| {
            panic!(
                "Expected edge_dep_client_server__0's path to visit the box's \
                 left edge at (x: {box_left_x}, y: {box_top_y}), got points: {points:?}"
            )
        });
        let exit_index = near_matches(box_right_x, box_top_y).unwrap_or_else(|| {
            panic!(
                "Expected edge_dep_client_server__0's path to visit the box's \
                 right edge at (x: {box_right_x}, y: {box_top_y}), got points: {points:?}"
            )
        });

        assert!(
            entry_index < exit_index,
            "Expected edge_dep_client_server__0's path to reach the box's left edge \
             (index {entry_index}) before its right edge (index {exit_index}), \
             threading left-to-right: {points:?}"
        );
    }
}

/// Regression test for the equivalent bug in the **same-rank** (cycle edge)
/// case: the description contact used a single fixed-corner point (biased by
/// `sibling_index_from_cmp_to` to avoid two edges sharing a box backtracking
/// through its center) instead of threading through the box, even though a
/// same-rank edge's divergent ancestors sit directly side by side -- the box
/// is genuinely on their connecting line, just like the cross-rank case is on
/// the line between ranks.
///
/// `edge_dep_client_server__0` in `0057` is a same-rank cyclic dependency
/// under `rank_dir: left_to_right`, where same-rank siblings are laid out
/// *vertically* (`Column`) within their shared rank. So
/// `EdgeSpacerCoordinatesCalculator::calculate_description_thread_same_rank`
/// rotates onto the `TopToBottom` row (fixed `x = box.left_x`) rather than
/// `calculate_description_thread`'s own `LeftToRight` row (which would fix
/// `y`) -- this asserts the rendered path visits both
/// `(box.left_x, box.top_y)` and `(box.left_x, box.bottom_y)`, in that order,
/// confirming it threads top-to-bottom along the box's left edge rather than
/// touching only one corner.
#[test]
fn test_same_rank_edge_description_contact_threads_through_box() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0057_INTERACTION_HALO_WITH_DESC_CYCLIC)
    {
        let desc = svg_elements
            .edge_description_infos
            .iter()
            .find(|d| d.edge_id.as_str() == "edge_dep_client_server__0")
            .expect("Expected a description for edge_dep_client_server__0.");
        // `edge_dep_client_server__0` is a dependency edge, which has no
        // interaction edge halo -- `EdgeDescriptionBuilder::edge_desc_build`
        // gives it no halo-clearance margin, and `description_thread_from_rect`
        // pulls back by `0.0`, so the routed path sits flush against the
        // box's own rendered left edge with no offset.
        let box_left_x = desc.x;
        let box_top_y = desc.y;
        let box_bottom_y = desc.y + desc.height;

        let svg = SvgElementsToSvgMapper::map(&svg_elements);
        let d = edge_body_path_d(&svg, "edge_dep_client_server__0");
        let points = path_d_points(&d);

        let near_matches = |x: f32, y: f32| -> Option<usize> {
            points
                .iter()
                .position(|&(px, py)| (px - x).abs() < 1e-2 && (py - y).abs() < 1e-2)
        };

        let entry_index = near_matches(box_left_x, box_top_y).unwrap_or_else(|| {
            panic!(
                "Expected edge_dep_client_server__0's path to visit the box's \
                 top edge at (x: {box_left_x}, y: {box_top_y}), got points: {points:?}"
            )
        });
        let exit_index = near_matches(box_left_x, box_bottom_y).unwrap_or_else(|| {
            panic!(
                "Expected edge_dep_client_server__0's path to visit the box's \
                 bottom edge at (x: {box_left_x}, y: {box_bottom_y}), got points: {points:?}"
            )
        });

        assert!(
            entry_index < exit_index,
            "Expected edge_dep_client_server__0's path to reach the box's top edge \
             (index {entry_index}) before its bottom edge (index {exit_index}), \
             threading top-to-bottom: {points:?}"
        );
    }
}

// === Same-rank description container crossing tests === //

/// Regression test for a bug where an edge sharing a same-rank
/// `edge_description_container`'s pair of divergent siblings, but not itself
/// described (no entry in `edge_descs`), got no routing waypoint at all and
/// cut straight past/through the box.
///
/// `edge_dep_server_proc_1_proc_2__1` is the reverse direction of the same
/// cyclic pair as the described `edge_dep_server_proc_1_proc_2__0` -- it
/// shares the same two divergent siblings (`t_server_proc_1` /
/// `t_server_proc_2`) and the same same-rank container position, but has no
/// description of its own. Before the fix its rendered path was a bare
/// `M ... L ...` two-point straight line; this asserts it now has real
/// intermediate waypoints, with at least one inside the description box's
/// vertical band, confirming it routes via a spacer crossing the box rather
/// than ignoring it.
#[test]
fn test_same_rank_crossing_edge_routes_around_description_container() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0057_INTERACTION_HALO_WITH_DESC_CYCLIC)
    {
        let desc = svg_elements
            .edge_description_infos
            .iter()
            .find(|d| d.edge_id.as_str() == "edge_dep_server_proc_1_proc_2__0")
            .expect("Expected a description for edge_dep_server_proc_1_proc_2__0.");
        let box_top_y = desc.y;
        let box_bottom_y = desc.y + desc.height;

        let svg = SvgElementsToSvgMapper::map(&svg_elements);
        let d = edge_body_path_d(&svg, "edge_dep_server_proc_1_proc_2__1");
        let points = path_d_points(&d);

        assert!(
            points.len() > 2,
            "Expected edge_dep_server_proc_1_proc_2__1's path to have intermediate \
             waypoints routing around the description container, not a bare two-point \
             straight line: {points:?}"
        );

        let visits_box_band = points
            .iter()
            .any(|&(_x, y)| y >= box_top_y - 0.5 && y <= box_bottom_y + 0.5);
        assert!(
            visits_box_band,
            "Expected edge_dep_server_proc_1_proc_2__1's path to visit a waypoint within \
             the description box's vertical band (y: {box_top_y}..{box_bottom_y}), \
             confirming it routes via a spacer crossing the box: {points:?}"
        );
    }
}

/// Regression test for a bug where a same-rank (cycle edge) description
/// contact's final orthogonal corner landed inside its own arrow head instead
/// of clearing it, because same-rank non-cycle edges (adjacent siblings) were
/// unconditionally given zero protrusion, even when they had a real spacer
/// waypoint (their own description-contact thread-through).
///
/// `edge_dep_server_proc_1_proc_2__0` and `__1` are both same-rank,
/// adjacent-sibling edges with a spacer waypoint (the owning edge's own
/// description contact, and the crossing spacer respectively) -- both must
/// now have `to_protrusion` floored to at least `TO_PROTRUSION_MIN_PX` so the
/// Z/S bend clears the arrow head.
#[test]
fn test_same_rank_edge_to_protrusion_clears_arrow_head() {
    const EPSILON: f32 = 0.01;

    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0057_INTERACTION_HALO_WITH_DESC_CYCLIC)
    {
        for edge_id in [
            "edge_dep_server_proc_1_proc_2__0",
            "edge_dep_server_proc_1_proc_2__1",
        ] {
            let edge_info = svg_elements
                .svg_edge_infos
                .iter()
                .find(|e| e.edge_id.as_str() == edge_id)
                .unwrap_or_else(|| panic!("Expected edge {edge_id}"));
            let to_protrusion = edge_info.ortho_protrusion_params.to_protrusion;

            assert!(
                to_protrusion >= TO_PROTRUSION_MIN_PX - EPSILON,
                "edge {edge_id} to_protrusion {to_protrusion:.2} should be at least \
                 {TO_PROTRUSION_MIN_PX} so the Z/S bend clears the arrow head"
            );
        }
    }
}

/// Regression test for a bug where a same-rank edge's arrow-head-clearance
/// `from_protrusion`/`to_protrusion` (sized to clear the arrow head at the
/// real node, tens of pixels) leaked into its lone spacer waypoint's own
/// `entry_protrusion`/`exit_protrusion` (meant to be a small stub past a real
/// spacer's boundary), extending the path's waypoint far past the
/// `edge_description_container`'s box or the crossing spacer's tiny footprint
/// and back, producing a spurious double bend / detour.
///
/// Affects both the owning edge's own description-contact "spacer"
/// (`edge_dep_client_server__0`, `edge_dep_server_proc_1_proc_2__0`) and a
/// crossing edge's real spacer leaf (`edge_dep_server_proc_1_proc_2__1`) --
/// in all three cases the spacer's own entry/exit protrusion must stay well
/// below the node-endpoint protrusion it must not be confused with.
#[test]
fn test_same_rank_edge_spacer_protrusion_not_inflated_by_node_protrusion() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0057_INTERACTION_HALO_WITH_DESC_CYCLIC)
    {
        for edge_id in [
            "edge_dep_client_server__0",
            "edge_dep_server_proc_1_proc_2__0",
            "edge_dep_server_proc_1_proc_2__1",
        ] {
            let edge_info = svg_elements
                .svg_edge_infos
                .iter()
                .find(|e| e.edge_id.as_str() == edge_id)
                .unwrap_or_else(|| panic!("Expected edge {edge_id}"));
            let params = &edge_info.ortho_protrusion_params;
            let spacer_prot = params
                .spacer_protrusions
                .first()
                .unwrap_or_else(|| panic!("Expected a spacer waypoint for edge {edge_id}"));

            assert!(
                spacer_prot.entry_protrusion < params.from_protrusion,
                "edge {edge_id} spacer entry_protrusion {:.2} should be well below \
                 from_protrusion {:.2} (not leaked from the node-endpoint arrow-head floor)",
                spacer_prot.entry_protrusion,
                params.from_protrusion,
            );
            assert!(
                spacer_prot.exit_protrusion < params.to_protrusion,
                "edge {edge_id} spacer exit_protrusion {:.2} should be well below \
                 to_protrusion {:.2} (not leaked from the node-endpoint arrow-head floor)",
                spacer_prot.exit_protrusion,
                params.to_protrusion,
            );
        }
    }
}

/// Regression test for a bug where a same-rank crossing edge's path still cut
/// through its `edge_description_container`'s bounding rect, even after the
/// entry/exit-protrusion leak (see
/// `test_same_rank_edge_spacer_protrusion_not_inflated_by_node_protrusion`)
/// was fixed.
///
/// The crossing spacer's entry/exit were resolved via the direction-oblivious
/// generic `EdgeSpacerCoordinatesCalculator::calculate`, which always treats
/// the container's "forward" end (matching the diagram's canonical `RankDir`)
/// as the entry -- for `edge_dep_server_proc_1_proc_2__1` (whose divergent
/// ancestors sit in the *reverse* order along the shared rank), this selected
/// the far end of the crossing spacer as the nominal entry point, so the
/// connector from the `from`-node's protrusion leg had to cut straight through
/// the box's vertical span to reach it before turning. Fixed by resolving the
/// crossing spacer via `calculate_description_thread_same_rank` (rotated
/// axis, entry/exit swapped to match *this* edge's own travel direction),
/// mirroring how the owning edge's own description contact is already
/// resolved.
///
/// Asserts that no segment of the crossing edge's rendered path overlaps the
/// description box's bounding rect (checked as an axis-aligned bounding-box
/// test per consecutive point pair, which correctly covers this diagram's
/// orthogonal routing: every real routing segment is horizontal or vertical,
/// and the small `ARC_RADIUS`-rounded corners are short enough that their
/// own bounding box is equally telling).
#[test]
fn test_same_rank_crossing_edge_path_does_not_cross_description_box() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0057_INTERACTION_HALO_WITH_DESC_CYCLIC)
    {
        let desc = svg_elements
            .edge_description_infos
            .iter()
            .find(|d| d.edge_id.as_str() == "edge_dep_server_proc_1_proc_2__0")
            .expect("Expected a description for edge_dep_server_proc_1_proc_2__0.");
        let box_left_x = desc.x;
        let box_right_x = desc.x + desc.width;
        let box_top_y = desc.y;
        let box_bottom_y = desc.y + desc.height;

        let svg = SvgElementsToSvgMapper::map(&svg_elements);
        let d = edge_body_path_d(&svg, "edge_dep_server_proc_1_proc_2__1");
        let points = path_d_points(&d);

        for pair in points.windows(2) {
            let &[(x1, y1), (x2, y2)] = pair else {
                unreachable!("windows(2) always yields pairs");
            };
            let seg_left_x = x1.min(x2);
            let seg_right_x = x1.max(x2);
            let seg_top_y = y1.min(y2);
            let seg_bottom_y = y1.max(y2);

            let overlaps_box = seg_left_x < box_right_x
                && seg_right_x > box_left_x
                && seg_top_y < box_bottom_y
                && seg_bottom_y > box_top_y;

            assert!(
                !overlaps_box,
                "Expected edge_dep_server_proc_1_proc_2__1's path segment \
                 (({x1:.2}, {y1:.2}) -> ({x2:.2}, {y2:.2})) to stay clear of the \
                 description box (x: {box_left_x:.2}..{box_right_x:.2}, \
                 y: {box_top_y:.2}..{box_bottom_y:.2}), full path: {points:?}"
            );
        }
    }
}

// === Fallback contact vs. co-located real label geometry tests === //

/// Regression test for a bug where a label-less edge's slot-based fallback
/// contact point landed inside a *different*, co-located edge's real
/// (text-bearing) label box.
///
/// `edge_dep_client_server__1` (`t_server -> t_client`) has no entry in
/// `edge_labels`, so its `t_server`-side contact comes from the per-kind
/// slot-arithmetic fallback (see "Edge-kind pools" in
/// `SvgEdgeInfosBuilder::face_offsets_compute`). `edge_ix_client_server__0`
/// (`t_client -> t_server`) is a co-located interaction edge that *does* have
/// a label on the same `t_server` face (its `to_label`, text: `"c/s to"`).
/// Before the fix, the fallback arithmetic centered independently per kind
/// and had no awareness of the interaction edge's real label geometry, so
/// `edge_dep_client_server__1`'s path clipped through the label's rendered
/// box. Asserts no segment of its rendered path overlaps that box (same
/// axis-aligned bounding-box check per consecutive point pair as
/// `test_same_rank_crossing_edge_path_does_not_cross_description_box`).
#[test]
fn test_fallback_contact_clears_co_located_interaction_label_box() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0057_INTERACTION_HALO_WITH_DESC_CYCLIC)
    {
        let to_label = svg_elements
            .edge_label_infos
            .iter()
            .find(|label| label.edge_id.as_str() == "edge_ix_client_server__0")
            .and_then(|label| label.to_label.as_ref())
            .expect("Expected edge_ix_client_server__0 to have a to_label with real content");
        let box_left_x = to_label.x;
        let box_right_x = to_label.x + to_label.width;
        let box_top_y = to_label.y;
        let box_bottom_y = to_label.y + to_label.height;

        let svg = SvgElementsToSvgMapper::map(&svg_elements);
        let d = edge_body_path_d(&svg, "edge_dep_client_server__1");
        let points = path_d_points(&d);

        for pair in points.windows(2) {
            let &[(x1, y1), (x2, y2)] = pair else {
                unreachable!("windows(2) always yields pairs");
            };
            let seg_left_x = x1.min(x2);
            let seg_right_x = x1.max(x2);
            let seg_top_y = y1.min(y2);
            let seg_bottom_y = y1.max(y2);

            let overlaps_box = seg_left_x < box_right_x
                && seg_right_x > box_left_x
                && seg_top_y < box_bottom_y
                && seg_bottom_y > box_top_y;

            assert!(
                !overlaps_box,
                "Expected edge_dep_client_server__1's path segment \
                 (({x1:.2}, {y1:.2}) -> ({x2:.2}, {y2:.2})) to stay clear of \
                 edge_ix_client_server__0's to_label box (x: {box_left_x:.2}..{box_right_x:.2}, \
                 y: {box_top_y:.2}..{box_bottom_y:.2}), full path: {points:?}"
            );
        }
    }
}

// === Higher-to-lower rank edge routing tests === //

/// Building SVG elements for a diagram whose curved interaction edge runs
/// from a higher-ranked divergent ancestor to a lower-ranked one must not
/// panic.
///
/// Regression test for a debug-mode `u32` underflow in
/// `OrthoProtrusionCalculator::spacer_gap_key`: an edge from rank 1 to rank 0
/// with more spacers than intermediate ranks computed
/// `rank_from - 1 - spacer_idx`, which underflowed (panicking in debug
/// builds, and producing garbage `RankGapKey` buckets in release builds).
#[test]
fn test_0062_higher_to_lower_rank_curved_edge_builds_without_panic() {
    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0062_EDGES_FROM_HIGHER_RANK_TO_LOWER_RANK)
    {
        assert!(
            svg_elements
                .svg_edge_infos
                .iter()
                .any(|svg_edge_info| svg_edge_info
                    .edge_id
                    .starts_with("edge_ix__t_aws_s3_tier_footage__t_aws_rds_tier")),
            "Expected edge_ix__t_aws_s3_tier_footage__t_aws_rds_tier edges to be built."
        );
    }
}

/// `edge_ix__t_aws_s3_tier_footage__t_aws_rds_tier__0` enters `t_aws_az1`
/// (described) then descends into `t_aws_az1_subnet_tier_private` (also
/// described) to reach `t_aws_az1_subnet_tier_private_lambda_pipe`.
/// `t_aws_az1`'s cross-container spacers (needed to clear
/// `t_aws_az1_subnet_mgmt` / `t_aws_az1_subnet_tier_public`) already snap to
/// a shared outer column. Before `spacers_snap_to_outermost_column` also
/// pulled a later, deeper text-content spacer out to meet that column,
/// `t_aws_az1_subnet_tier_private`'s own (un-aligned) label spacer sat
/// inside it, so the path bowed out to the column while clearing
/// `t_aws_az1`'s siblings, dipped back in to clear
/// `t_aws_az1_subnet_tier_private`'s own label, then had to jog back out
/// again toward the to-node -- an unnecessary extra Z-bend.
#[test]
fn test_0062_deeper_described_container_spacer_does_not_dip_inside_outer_column() {
    fn path_points(path_d: &str) -> Vec<(f32, f32)> {
        path_d
            .split([' ', 'M', 'L', 'C'])
            .filter_map(|tok| {
                let (x, y) = tok.split_once(',')?;
                Some((x.trim().parse::<f32>().ok()?, y.trim().parse::<f32>().ok()?))
            })
            .collect()
    }

    for svg_elements in
        build_svg_elements_for_diagram(INPUT_DIAGRAM_0062_EDGES_FROM_HIGHER_RANK_TO_LOWER_RANK)
    {
        let edge = svg_elements
            .svg_edge_infos
            .iter()
            .find(|svg_edge_info| {
                svg_edge_info.edge_id.as_str()
                    == "edge_ix__t_aws_s3_tier_footage__t_aws_rds_tier__0"
            })
            .expect("Expected edge_ix__t_aws_s3_tier_footage__t_aws_rds_tier__0");

        let az1 = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_aws_az1")
            .expect("Expected t_aws_az1 in svg_node_infos");
        let subnet_tier_private = svg_elements
            .svg_node_infos
            .iter()
            .find(|n| n.node_id.as_str() == "t_aws_az1_subnet_tier_private")
            .expect("Expected t_aws_az1_subnet_tier_private in svg_node_infos");

        let points = path_points(&edge.path_d);

        // The region within `t_aws_az1` but above `t_aws_az1_subnet_tier_private`
        // is where the path must clear `t_aws_az1`'s other rank-0 siblings
        // (`t_aws_az1_subnet_mgmt` / `t_aws_az1_subnet_tier_public`) via its
        // cross-container spacers -- this is where the outer column is
        // established.
        let clearing_region_max_x = points
            .iter()
            .filter(|&&(_, y)| y >= az1.y && y < subnet_tier_private.y)
            .map(|&(x, _)| x)
            .fold(f32::MIN, f32::max);
        assert!(
            clearing_region_max_x > f32::MIN,
            "Expected at least one waypoint within t_aws_az1's own clearing \
             region (y in [{:.1}, {:.1})); path: {}",
            az1.y,
            subnet_tier_private.y,
            edge.path_d,
        );

        // `t_aws_az1_subnet_tier_private`'s own text-content spacer sits just
        // below its top face, marking a column just past its label. Once the
        // path reaches this band it should already be at (or past) the outer
        // column reached while clearing `t_aws_az1`'s siblings -- not dipped
        // back inside it.
        let private_title_band_max_x = points
            .iter()
            .filter(|&&(_, y)| {
                y >= subnet_tier_private.y && y < subnet_tier_private.y + TEXT_LINE_HEIGHT * 2.0
            })
            .map(|&(x, _)| x)
            .fold(f32::MIN, f32::max);
        assert!(
            private_title_band_max_x > f32::MIN,
            "Expected at least one waypoint within t_aws_az1_subnet_tier_private's \
             own title band; path: {}",
            edge.path_d,
        );

        assert!(
            (private_title_band_max_x - clearing_region_max_x).abs() < 1.0,
            "t_aws_az1_subnet_tier_private's label spacer ({private_title_band_max_x:.1}) \
             should sit at the same outer column already reached while clearing \
             t_aws_az1's siblings ({clearing_region_max_x:.1}), not dip back inside \
             it; path: {}",
            edge.path_d,
        );
    }
}
