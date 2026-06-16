use disposition::input_model::{DiagramFocus, InputDiagram};
use disposition_input_ir_rt::{DiagramGenerator, EdgeAnimationActive, InputDiagramMerger};

use crate::input_ir_rt::EXAMPLE_INPUT;

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
