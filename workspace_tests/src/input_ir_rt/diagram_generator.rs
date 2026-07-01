use disposition::input_model::{DiagramFocus, InputDiagram};
use disposition_input_ir_rt::{DiagramGenerator, EdgeAnimationActive, InputDiagramMerger};

use crate::input_ir_rt::{
    EXAMPLE_INPUT, INPUT_DIAGRAM_0012_EDGE_FROM_NESTED_NODE_TO_OUTER_NODE_CYCLIC,
    INPUT_DIAGRAM_0044_EDGE_OFFSETS_AND_PROTRUSION_COMPLEX_2,
};

/// `PerProcessStepOrTag` generation produces one diagram per focus state, in
/// the documented order, with no interactive focus CSS baked into the SVGs.
#[test]
fn generate_per_process_step_or_tag_produces_one_diagram_per_focus_in_order() {
    let input_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();

    // Mirror the merge that `generate_per_process_step_or_tag` performs, then
    // build the expected ordered focus list: nothing focused, then each process
    // followed by its steps, then each tag -- all in declaration order.
    let merged = InputDiagramMerger::merge(InputDiagram::base(), &input_diagram);
    let mut expected_focuses = vec![DiagramFocus::None];
    merged
        .processes
        .iter()
        .for_each(|(process_id, process_diagram)| {
            expected_focuses.push(DiagramFocus::Process(process_id.clone()));
            process_diagram.steps.keys().for_each(|process_step_id| {
                expected_focuses.push(DiagramFocus::ProcessStep {
                    process_id: process_id.clone(),
                    process_step_id: process_step_id.clone(),
                });
            });
        });
    merged.tags.keys().for_each(|tag_id| {
        expected_focuses.push(DiagramFocus::Tag(tag_id.clone()));
    });

    let diagrams = DiagramGenerator::generate_per_process_step_or_tag(
        &input_diagram,
        EdgeAnimationActive::OnProcessStepFocus,
    )
    .expect("Expected diagrams to be generated.");

    let actual_focuses = diagrams
        .iter()
        .map(|diagram_focus_generated| diagram_focus_generated.focus.clone())
        .collect::<Vec<_>>();
    assert_eq!(expected_focuses, actual_focuses);

    // Baked diagrams must not contain interactive focus CSS selectors.
    diagrams.iter().for_each(|diagram_focus_generated| {
        let focus = &diagram_focus_generated.focus;
        let svg = &diagram_focus_generated.diagram_generated.svg;
        assert!(
            !svg.contains("peer/"),
            "found `peer/` in baked svg for {focus:?}"
        );
        assert!(
            !svg.contains("peer-["),
            "found `peer-[` in baked svg for {focus:?}"
        );
        assert!(
            !svg.contains("group-has-["),
            "found `group-has-[` in baked svg for {focus:?}"
        );
    });
}

/// The interactive (single diagram) path keeps emitting `group-has-[...]` focus
/// CSS, so the absence of those selectors in the baked path is meaningful.
#[test]
fn generate_single_diagram_retains_interactive_focus_css() {
    let input_diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();

    let diagram_generated =
        DiagramGenerator::generate(&input_diagram, EdgeAnimationActive::OnProcessStepFocus)
            .expect("Expected diagram to be generated.");

    assert!(
        diagram_generated.svg.contains("group-has-["),
        "expected interactive `group-has-[` focus CSS in the single diagram"
    );
}

/// The edge-routing diagnostics are populated alongside the SVG elements, and
/// stay consistent with them: one `edge_entries` record per rendered orthogonal
/// edge, with matching protrusion params, and every `rank_gap_entries` record
/// references an edge present in `edge_entries`.
#[test]
fn generate_populates_consistent_edge_routing_diagnostics() {
    for input_diagram_yaml in [
        INPUT_DIAGRAM_0012_EDGE_FROM_NESTED_NODE_TO_OUTER_NODE_CYCLIC,
        INPUT_DIAGRAM_0044_EDGE_OFFSETS_AND_PROTRUSION_COMPLEX_2,
    ] {
        let input_diagram = serde_saphyr::from_str::<InputDiagram>(input_diagram_yaml).unwrap();
        let diagram_generated =
            DiagramGenerator::generate(&input_diagram, EdgeAnimationActive::OnProcessStepFocus)
                .expect("Expected diagram to be generated.");

        let edge_routing_diagnostics = &diagram_generated.edge_routing_diagnostics;

        assert!(
            !edge_routing_diagnostics.edge_entries.is_empty(),
            "Expected edge_entries to be populated for a diagram with edges."
        );

        // Each diagnostic entry's protrusion params match the rendered edge's.
        for edge_entry in &edge_routing_diagnostics.edge_entries {
            let svg_edge_info = diagram_generated
                .svg_elements
                .svg_edge_infos
                .iter()
                .find(|svg_edge_info| svg_edge_info.edge_id == edge_entry.edge_id)
                .unwrap_or_else(|| {
                    panic!(
                        "Expected svg_edge_info for diagnostic edge {:?}",
                        edge_entry.edge_id
                    )
                });

            assert_eq!(
                svg_edge_info.ortho_protrusion_params, edge_entry.ortho_protrusion_params,
                "Protrusion params for edge {:?} should match between the diagnostics and \
                 the rendered edge.",
                edge_entry.edge_id,
            );
        }

        // Every rank-gap entry references an edge present in `edge_entries`.
        for rank_gap_entry in &edge_routing_diagnostics.rank_gap_entries {
            assert!(
                edge_routing_diagnostics
                    .edge_entries
                    .iter()
                    .any(|edge_entry| edge_entry.edge_id == rank_gap_entry.edge_id),
                "rank_gap_entry edge {:?} should resolve to a known edge_entry.",
                rank_gap_entry.edge_id,
            );
            assert!(
                rank_gap_entry.rank_low <= rank_gap_entry.rank_high,
                "rank_gap_entry should have rank_low <= rank_high, got {:?} > {:?}",
                rank_gap_entry.rank_low,
                rank_gap_entry.rank_high,
            );
        }
    }
}
