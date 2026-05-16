use disposition::{
    input_ir_model::IrDiagramAndIssues,
    input_model::InputDiagram,
    ir_model::IrDiagram,
    model_common::{id, Id},
    svg_model::SvgElements,
    taffy_model::{taffy::TaffyError, DimensionAndLod},
};
use disposition_input_ir_rt::{
    EdgeAnimationActive, InputDiagramMerger, InputToIrDiagramMapper, IrToTaffyBuilder,
    TaffyToSvgElementsMapper,
};

use crate::input_ir_rt::{
    EXAMPLE_IR, INPUT_DIAGRAM_0001_NESTED_NODE_EDGE_PROTRUSION,
    INPUT_DIAGRAM_0002_NESTED_NODE_EDGE_PROTRUSION, INPUT_DIAGRAM_0003_EDGES_SYMMETRIC_2_NODES,
    INPUT_DIAGRAM_0004_EDGES_SYMMETRIC_3_NODES, INPUT_DIAGRAM_0005_TAG_NODES_CYCLIC_EDGE,
    INPUT_DIAGRAM_0006_PROCESS_STEP_NODES_CYCLIC_EDGE,
    INPUT_DIAGRAM_0007_EDGE_FROM_NODE_TO_NESTED_NODE,
    INPUT_DIAGRAM_0008_EDGE_FROM_NODE_TO_NESTED_RANK_1_NODE,
    INPUT_DIAGRAM_0009_EDGE_WITH_DESCRIPTION,
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
    // Arc radius used by the orthogonal path builder for rounded corners.
    // This constant matches the `ARC_RADIUS` in
    // `edge_path_builder_pass_2_ortho.rs`.
    const ARC_RADIUS: f32 = 4.0;

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

// === Cycle edge routing tests === //

/// Parse all SVG path endpoint coordinates (from `M` and `L` commands) from a
/// path `d` attribute string.
///
/// Returns a `Vec<(f32, f32)>` of `(x, y)` pairs.
fn parse_path_endpoints(path_d: &str) -> Vec<(f32, f32)> {
    let mut result = Vec::new();
    // Iterate over whitespace-separated tokens.
    let tokens: Vec<&str> = path_d.split_whitespace().collect();
    let mut i = 0;
    while i < tokens.len() {
        let token = tokens[i];
        match token {
            "M" | "L" => {
                if let Some(coords) = tokens.get(i + 1) {
                    if let Some((x, y)) = parse_coord_pair(coords) {
                        result.push((x, y));
                    }
                    i += 2;
                    continue;
                }
            }
            "C" => {
                // Curve: ctrl1 ctrl2 endpoint -- record all three pairs.
                for offset in 1..=3 {
                    if let Some(coords) = tokens.get(i + offset) {
                        if let Some((x, y)) = parse_coord_pair(coords) {
                            result.push((x, y));
                        }
                    }
                }
                i += 4;
                continue;
            }
            _ => {
                // Single token may be a coordinate pair if it contains a comma.
                if token.contains(',') {
                    if let Some((x, y)) = parse_coord_pair(token) {
                        result.push((x, y));
                    }
                }
            }
        }
        i += 1;
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
            // Only check edges where both endpoints are thing nodes
            // (process_id is None for thing/tag nodes; Some for process nodes
            // and process step nodes).
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

        // The path is built from to-node to from-node in the SVG direction.
        // For a Bottom (t_bob) -> Top (t_charlie_1) edge the first SVG `M`
        // command should have y near t_charlie_1's top face (charlie_1.y)
        // and the final `L` command should be near t_bob's bottom face
        // (bob.y + bob.height_collapsed).
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
        let expected_first_y = charlie_1.y; // Top face of t_charlie_1
        assert!(
            (first_y - expected_first_y).abs() <= tolerance,
            "First path point y={first_y:.2} should be near t_charlie_1 top face \
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
        let expected_last_y = bob.y + bob.height_collapsed; // Bottom face of t_bob
        assert!(
            (last_y - expected_last_y).abs() <= tolerance,
            "Last path point y={last_y:.2} should be near t_bob bottom face \
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

        // The path is built in SVG order from the to-node (charlie_1, at the
        // bottom) to the from-node (alice, at the top). All coordinates
        // between the first (charlie_1 contact y) and the last (alice contact
        // y) are the routing segment.
        let all_coords = parse_path_endpoints(&alice_charlie_1_edge.path_d);

        // The from-protrusion tip is the second-to-last coordinate (just
        // before the alice contact point). Its y is the upper bound of the
        // routing gap -- no intermediate point should overshoot above it.
        let from_protrusion_tip_y = all_coords
            .get(all_coords.len().wrapping_sub(2))
            .map(|&(_, y)| y)
            .unwrap_or(0.0);

        // Skip the first (charlie_1 contact) and last (alice contact).
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

        // The path is built in SVG order from the to-node (t_charlie_inner,
        // at the bottom) to the from-node (t_alice_inner, at the top). The
        // first coordinate is t_charlie_inner's contact point (inside
        // t_charlie_outer) and the last is t_alice_inner's contact point
        // (above all containers).
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

        // The path is built in SVG order: to-node first, from-node last.
        // After the fix, all intermediate points lie at exactly y=215.
        // Before the fix they included arc control points at y≈211--213
        // (the U-bend detour) that are above t_charlie_outer's top.
        let all_coords = parse_path_endpoints(&alice_inner_charlie_inner_edge.path_d);
        let intermediate_coords = all_coords
            .iter()
            .skip(1)
            .take(all_coords.len().saturating_sub(2));

        for &(x, y) in intermediate_coords {
            assert!(
                y >= charlie_outer_top_y - 0.5,
                "Intermediate routing coordinate ({x:.3}, {y:.3}) is above \
                 t_charlie_outer's top boundary (y={charlie_outer_top_y:.3}). \
                 The path detours above the boundary, indicating a V-spike. \
                 path_d = {:?}",
                alice_inner_charlie_inner_edge.path_d,
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

        // Path starts at to-node (t_charlie_1 top face).
        let (_, first_y) = path_tokens
            .first()
            .and_then(|t| parse_suffixed(t, 'M'))
            .expect("Path should start with M command (e.g. M80,210)");
        let expected_first_y = charlie_1.y;
        assert!(
            (first_y - expected_first_y).abs() <= tolerance,
            "First path point y={first_y:.2} should be near t_charlie_1 top face \
             y={expected_first_y:.2} (tolerance {tolerance:.0} px). \
             Cycle-edge routing produces a different starting y. \
             path_d = {:?}",
            bob_charlie_1_edge.path_d,
        );

        // Path ends at from-node (t_bob bottom face).
        let (_, last_y) = path_tokens
            .last()
            .and_then(|t| parse_suffixed(t, 'L').or_else(|| parse_suffixed(t, 'M')))
            .expect("Path should end with an L or M command");
        let expected_last_y = bob.y + bob.height_collapsed;
        assert!(
            (last_y - expected_last_y).abs() <= tolerance,
            "Last path point y={last_y:.2} should be near t_bob bottom face \
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

/// An edge with a description in `entity_descs` must produce a non-empty
/// `edge_label_infos` entry in `SvgElements`.
///
/// The edge ID used as the `entity_descs` key follows the generated format
/// `"{edge_group_id}__{edge_index}"`, e.g. `edge_ab__0` for index 0 of the
/// `edge_ab` group.
#[test]
fn test_edge_description_produces_edge_label_infos() {
    for svg_elements in build_svg_elements_from_edge_with_description() {
        assert!(
            !svg_elements.edge_label_infos.is_empty(),
            "Expected edge_label_infos to be non-empty when entity_descs contains the edge ID"
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

/// The text in the edge label spans must match the description from
/// `entity_descs`.
#[test]
fn test_edge_description_text_matches_entity_descs() {
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
