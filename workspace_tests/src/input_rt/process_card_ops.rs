//! Tests for `disposition_input_rt::process_card_ops::ProcessCardOps`.

use disposition::input_model::{process::ProcessDiagram, InputDiagram};
use disposition_input_rt::{
    id_parse::{parse_process_id, parse_process_step_id},
    process_card_ops::ProcessCardOps,
};

fn empty_diagram() -> InputDiagram<'static> {
    InputDiagram::default()
}

fn diagram_with_process(process_id_str: &str) -> InputDiagram<'static> {
    let mut input_diagram = empty_diagram();
    let process_id = parse_process_id(process_id_str).unwrap();
    input_diagram
        .processes
        .insert(process_id, ProcessDiagram::default());
    input_diagram
}

fn diagram_with_process_and_steps(
    process_id_str: &str,
    steps: &[(&str, &str)],
) -> InputDiagram<'static> {
    let mut input_diagram = diagram_with_process(process_id_str);
    let process_id = parse_process_id(process_id_str).unwrap();
    let process_diagram = input_diagram.processes.get_mut(&process_id).unwrap();
    for (step_id_str, label) in steps {
        let step_id = parse_process_step_id(step_id_str).unwrap();
        process_diagram.steps.insert(step_id, label.to_string());
    }
    input_diagram
}

// === step_add === //

#[test]
fn step_add_inserts_into_empty_process() {
    let mut input_diagram = diagram_with_process("proc_0");

    ProcessCardOps::step_add(&mut input_diagram, "proc_0");

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    assert_eq!(process_diagram.steps.len(), 1);
    assert!(process_diagram.steps.contains_key(&step_id));
}

#[test]
fn step_add_generates_unique_step_ids() {
    let mut input_diagram = diagram_with_process_and_steps("proc_0", &[("proc_0_step_0", "First")]);

    ProcessCardOps::step_add(&mut input_diagram, "proc_0");

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let step_id_1 = parse_process_step_id("proc_0_step_1").unwrap();
    assert_eq!(process_diagram.steps.len(), 2);
    assert!(process_diagram.steps.contains_key(&step_id_1));
}

#[test]
fn step_add_skips_existing_step_ids() {
    let mut input_diagram =
        diagram_with_process_and_steps("proc_0", &[("proc_0_step_0", "A"), ("proc_0_step_1", "B")]);

    ProcessCardOps::step_add(&mut input_diagram, "proc_0");

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let step_id_2 = parse_process_step_id("proc_0_step_2").unwrap();
    assert_eq!(process_diagram.steps.len(), 3);
    assert!(process_diagram.steps.contains_key(&step_id_2));
}

#[test]
fn step_add_noop_for_invalid_process_id() {
    let mut input_diagram = diagram_with_process("proc_0");

    ProcessCardOps::step_add(&mut input_diagram, "");

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    assert!(process_diagram.steps.is_empty());
}

#[test]
fn step_add_noop_for_missing_process() {
    let mut input_diagram = empty_diagram();

    ProcessCardOps::step_add(&mut input_diagram, "proc_nonexistent");

    assert!(input_diagram.processes.is_empty());
}

// === step_remove === //

#[test]
fn step_remove_removes_step() {
    let mut input_diagram = diagram_with_process_and_steps(
        "proc_0",
        &[("proc_0_step_0", "Remove"), ("proc_0_step_1", "Keep")],
    );

    ProcessCardOps::step_remove(&mut input_diagram, "proc_0", "proc_0_step_0");

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let step_id_0 = parse_process_step_id("proc_0_step_0").unwrap();
    let step_id_1 = parse_process_step_id("proc_0_step_1").unwrap();
    assert_eq!(process_diagram.steps.len(), 1);
    assert!(!process_diagram.steps.contains_key(&step_id_0));
    assert!(process_diagram.steps.contains_key(&step_id_1));
}

#[test]
fn step_remove_noop_for_invalid_process_id() {
    let mut input_diagram = diagram_with_process_and_steps("proc_0", &[("proc_0_step_0", "Keep")]);

    ProcessCardOps::step_remove(&mut input_diagram, "", "proc_0_step_0");

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    assert_eq!(process_diagram.steps.len(), 1);
}

#[test]
fn step_remove_noop_for_invalid_step_id() {
    let mut input_diagram = diagram_with_process_and_steps("proc_0", &[("proc_0_step_0", "Keep")]);

    ProcessCardOps::step_remove(&mut input_diagram, "proc_0", "");

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    assert_eq!(process_diagram.steps.len(), 1);
}

#[test]
fn step_remove_noop_for_missing_step() {
    let mut input_diagram = diagram_with_process_and_steps("proc_0", &[("proc_0_step_0", "Keep")]);

    ProcessCardOps::step_remove(&mut input_diagram, "proc_0", "proc_0_step_99");

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    assert_eq!(process_diagram.steps.len(), 1);
}

// === step_move === //

#[test]
fn step_move_reorders_steps() {
    let mut input_diagram = diagram_with_process_and_steps(
        "proc_0",
        &[
            ("proc_0_step_a", "A"),
            ("proc_0_step_b", "B"),
            ("proc_0_step_c", "C"),
        ],
    );

    ProcessCardOps::step_move(&mut input_diagram, "proc_0", 0, 2);

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let keys: Vec<&str> = process_diagram.steps.keys().map(|k| k.as_str()).collect();
    assert_eq!(
        keys,
        vec!["proc_0_step_b", "proc_0_step_c", "proc_0_step_a"]
    );
}

#[test]
fn step_move_noop_for_invalid_process_id() {
    let mut input_diagram =
        diagram_with_process_and_steps("proc_0", &[("proc_0_step_a", "A"), ("proc_0_step_b", "B")]);

    ProcessCardOps::step_move(&mut input_diagram, "", 0, 1);

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let keys: Vec<&str> = process_diagram.steps.keys().map(|k| k.as_str()).collect();
    // Order unchanged.
    assert_eq!(keys, vec!["proc_0_step_a", "proc_0_step_b"]);
}

// === step_rename === //

#[test]
fn step_rename_renames_step_key() {
    let mut input_diagram =
        diagram_with_process_and_steps("proc_0", &[("proc_0_step_old", "Label")]);

    ProcessCardOps::step_rename(
        &mut input_diagram,
        "proc_0",
        "proc_0_step_old",
        "proc_0_step_new",
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let step_id_old = parse_process_step_id("proc_0_step_old").unwrap();
    let step_id_new = parse_process_step_id("proc_0_step_new").unwrap();
    assert!(!process_diagram.steps.contains_key(&step_id_old));
    assert!(process_diagram.steps.contains_key(&step_id_new));
    assert_eq!(process_diagram.steps.get(&step_id_new).unwrap(), "Label");
}

#[test]
fn step_rename_renames_in_step_thing_interactions() {
    let mut input_diagram =
        diagram_with_process_and_steps("proc_0", &[("proc_0_step_old", "Label")]);
    let process_id = parse_process_id("proc_0").unwrap();
    let step_id_old = parse_process_step_id("proc_0_step_old").unwrap();
    input_diagram
        .processes
        .get_mut(&process_id)
        .unwrap()
        .step_thing_interactions
        .insert(step_id_old.clone(), Vec::new());

    ProcessCardOps::step_rename(
        &mut input_diagram,
        "proc_0",
        "proc_0_step_old",
        "proc_0_step_new",
    );

    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let step_id_new = parse_process_step_id("proc_0_step_new").unwrap();
    assert!(!process_diagram
        .step_thing_interactions
        .contains_key(&step_id_old));
    assert!(process_diagram
        .step_thing_interactions
        .contains_key(&step_id_new));
}

#[test]
fn step_rename_noop_when_same_id() {
    let mut input_diagram = diagram_with_process_and_steps("proc_0", &[("proc_0_step_0", "Label")]);

    ProcessCardOps::step_rename(
        &mut input_diagram,
        "proc_0",
        "proc_0_step_0",
        "proc_0_step_0",
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    assert_eq!(process_diagram.steps.len(), 1);
    assert!(process_diagram.steps.contains_key(&step_id));
}

#[test]
fn step_rename_noop_for_invalid_old_step_id() {
    let mut input_diagram = diagram_with_process_and_steps("proc_0", &[("proc_0_step_0", "Label")]);

    ProcessCardOps::step_rename(&mut input_diagram, "proc_0", "", "proc_0_step_new");

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    assert_eq!(process_diagram.steps.len(), 1);
    assert!(process_diagram.steps.contains_key(&step_id));
}

#[test]
fn step_rename_noop_for_invalid_new_step_id() {
    let mut input_diagram = diagram_with_process_and_steps("proc_0", &[("proc_0_step_0", "Label")]);

    ProcessCardOps::step_rename(&mut input_diagram, "proc_0", "proc_0_step_0", "");

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    assert_eq!(process_diagram.steps.len(), 1);
    assert!(process_diagram.steps.contains_key(&step_id));
}

// === step_label_update === //

#[test]
fn step_label_update_changes_label() {
    let mut input_diagram =
        diagram_with_process_and_steps("proc_0", &[("proc_0_step_0", "Old Label")]);

    ProcessCardOps::step_label_update(&mut input_diagram, "proc_0", "proc_0_step_0", "New Label");

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    assert_eq!(process_diagram.steps.get(&step_id).unwrap(), "New Label");
}

#[test]
fn step_label_update_noop_for_missing_process() {
    let mut input_diagram = empty_diagram();

    ProcessCardOps::step_label_update(&mut input_diagram, "proc_missing", "step_0", "Label");

    assert!(input_diagram.processes.is_empty());
}

#[test]
fn step_label_update_noop_for_missing_step() {
    let mut input_diagram = diagram_with_process_and_steps("proc_0", &[("proc_0_step_0", "Old")]);

    ProcessCardOps::step_label_update(&mut input_diagram, "proc_0", "proc_0_step_missing", "New");

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    // Original step label unchanged.
    assert_eq!(process_diagram.steps.get(&step_id).unwrap(), "Old");
}

// === step_interaction_add === //

#[test]
fn step_interaction_add_picks_unmapped_step() {
    let mut input_diagram = diagram_with_process_and_steps(
        "proc_0",
        &[("proc_0_step_0", "First"), ("proc_0_step_1", "Second")],
    );

    ProcessCardOps::step_interaction_add(&mut input_diagram, "proc_0");

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let step_id_0 = parse_process_step_id("proc_0_step_0").unwrap();
    assert_eq!(process_diagram.step_thing_interactions.len(), 1);
    // Should pick the first unmapped step.
    assert!(process_diagram
        .step_thing_interactions
        .contains_key(&step_id_0));
}

#[test]
fn step_interaction_add_skips_already_mapped_steps() {
    let mut input_diagram = diagram_with_process_and_steps(
        "proc_0",
        &[("proc_0_step_0", "First"), ("proc_0_step_1", "Second")],
    );
    let process_id = parse_process_id("proc_0").unwrap();
    let step_id_0 = parse_process_step_id("proc_0_step_0").unwrap();
    input_diagram
        .processes
        .get_mut(&process_id)
        .unwrap()
        .step_thing_interactions
        .insert(step_id_0, Vec::new());

    ProcessCardOps::step_interaction_add(&mut input_diagram, "proc_0");

    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let step_id_1 = parse_process_step_id("proc_0_step_1").unwrap();
    assert_eq!(process_diagram.step_thing_interactions.len(), 2);
    // step_0 was already mapped, so step_1 should be chosen.
    assert!(process_diagram
        .step_thing_interactions
        .contains_key(&step_id_1));
}

#[test]
fn step_interaction_add_generates_placeholder_when_all_mapped() {
    let mut input_diagram = diagram_with_process_and_steps("proc_0", &[("proc_0_step_0", "Only")]);
    let process_id = parse_process_id("proc_0").unwrap();
    let step_id_0 = parse_process_step_id("proc_0_step_0").unwrap();
    input_diagram
        .processes
        .get_mut(&process_id)
        .unwrap()
        .step_thing_interactions
        .insert(step_id_0, Vec::new());

    ProcessCardOps::step_interaction_add(&mut input_diagram, "proc_0");

    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let step_id_1 = parse_process_step_id("proc_0_step_1").unwrap();
    assert_eq!(process_diagram.step_thing_interactions.len(), 2);
    // All steps mapped, so a placeholder should be generated.
    assert!(process_diagram
        .step_thing_interactions
        .contains_key(&step_id_1));
}

#[test]
fn step_interaction_add_noop_for_invalid_process_id() {
    let mut input_diagram = diagram_with_process("proc_0");

    ProcessCardOps::step_interaction_add(&mut input_diagram, "");

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    assert!(process_diagram.step_thing_interactions.is_empty());
}

#[test]
fn step_interaction_add_noop_for_missing_process() {
    let mut input_diagram = empty_diagram();

    ProcessCardOps::step_interaction_add(&mut input_diagram, "proc_missing");

    assert!(input_diagram.processes.is_empty());
}
