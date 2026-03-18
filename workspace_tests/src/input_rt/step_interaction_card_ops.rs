//! Tests for `disposition_input_rt::step_interaction_card_ops::StepInteractionCardOps`.

use disposition::input_model::{
    edge::{EdgeGroup, EdgeKind},
    process::ProcessDiagram,
    InputDiagram,
};
use disposition_input_rt::{
    id_parse::{parse_edge_group_id, parse_process_id, parse_process_step_id},
    StepInteractionCardOps,
};

fn empty_diagram() -> InputDiagram<'static> {
    InputDiagram::default()
}

fn diagram_with_process_and_step_interactions(
    process_id_str: &str,
    step_interactions: &[(&str, &[&str])],
) -> InputDiagram<'static> {
    let mut input_diagram = empty_diagram();
    let process_id = parse_process_id(process_id_str).unwrap();
    let mut process_diagram = ProcessDiagram::default();
    for (step_id_str, edge_group_id_strs) in step_interactions {
        let step_id = parse_process_step_id(step_id_str).unwrap();
        let edge_group_ids: Vec<_> = edge_group_id_strs
            .iter()
            .map(|s| parse_edge_group_id(s).unwrap())
            .collect();
        process_diagram
            .step_thing_interactions
            .insert(step_id, edge_group_ids);
    }
    input_diagram.processes.insert(process_id, process_diagram);
    input_diagram
}

// === step_interaction_remove === //

#[test]
fn step_interaction_remove_removes_mapping() {
    let mut input_diagram = diagram_with_process_and_step_interactions(
        "proc_0",
        &[
            ("proc_0_step_0", &["edge_a"]),
            ("proc_0_step_1", &["edge_b"]),
        ],
    );

    StepInteractionCardOps::step_interaction_remove(&mut input_diagram, "proc_0", "proc_0_step_0");

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let step_id_0 = parse_process_step_id("proc_0_step_0").unwrap();
    let step_id_1 = parse_process_step_id("proc_0_step_1").unwrap();
    assert!(!process_diagram
        .step_thing_interactions
        .contains_key(&step_id_0));
    assert!(process_diagram
        .step_thing_interactions
        .contains_key(&step_id_1));
}

#[test]
fn step_interaction_remove_noop_for_invalid_process_id() {
    let mut input_diagram =
        diagram_with_process_and_step_interactions("proc_0", &[("proc_0_step_0", &["edge_a"])]);

    StepInteractionCardOps::step_interaction_remove(&mut input_diagram, "", "proc_0_step_0");

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    assert_eq!(process_diagram.step_thing_interactions.len(), 1);
}

#[test]
fn step_interaction_remove_noop_for_invalid_step_id() {
    let mut input_diagram =
        diagram_with_process_and_step_interactions("proc_0", &[("proc_0_step_0", &["edge_a"])]);

    StepInteractionCardOps::step_interaction_remove(&mut input_diagram, "proc_0", "");

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    assert_eq!(process_diagram.step_thing_interactions.len(), 1);
}

#[test]
fn step_interaction_remove_noop_for_missing_process() {
    let mut input_diagram = empty_diagram();

    StepInteractionCardOps::step_interaction_remove(
        &mut input_diagram,
        "proc_nonexistent",
        "step_0",
    );

    assert!(input_diagram.processes.is_empty());
}

#[test]
fn step_interaction_remove_noop_for_missing_step() {
    let mut input_diagram =
        diagram_with_process_and_step_interactions("proc_0", &[("proc_0_step_0", &["edge_a"])]);

    StepInteractionCardOps::step_interaction_remove(
        &mut input_diagram,
        "proc_0",
        "proc_0_step_nonexistent",
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    assert_eq!(process_diagram.step_thing_interactions.len(), 1);
}

// === step_interaction_rename === //

#[test]
fn step_interaction_rename_renames_step_key() {
    let mut input_diagram = diagram_with_process_and_step_interactions(
        "proc_0",
        &[("proc_0_step_old", &["edge_a", "edge_b"])],
    );

    StepInteractionCardOps::step_interaction_rename(
        &mut input_diagram,
        "proc_0",
        "proc_0_step_old",
        "proc_0_step_new",
        &["edge_a".to_owned(), "edge_b".to_owned()],
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let step_id_old = parse_process_step_id("proc_0_step_old").unwrap();
    let step_id_new = parse_process_step_id("proc_0_step_new").unwrap();
    assert!(!process_diagram
        .step_thing_interactions
        .contains_key(&step_id_old));
    assert!(process_diagram
        .step_thing_interactions
        .contains_key(&step_id_new));

    let edge_group_ids = process_diagram
        .step_thing_interactions
        .get(&step_id_new)
        .unwrap();
    let edge_group_id_a = parse_edge_group_id("edge_a").unwrap();
    let edge_group_id_b = parse_edge_group_id("edge_b").unwrap();
    assert_eq!(edge_group_ids, &vec![edge_group_id_a, edge_group_id_b]);
}

#[test]
fn step_interaction_rename_noop_when_same_id() {
    let mut input_diagram =
        diagram_with_process_and_step_interactions("proc_0", &[("proc_0_step_0", &["edge_a"])]);

    StepInteractionCardOps::step_interaction_rename(
        &mut input_diagram,
        "proc_0",
        "proc_0_step_0",
        "proc_0_step_0",
        &["edge_a".to_owned()],
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    assert_eq!(process_diagram.step_thing_interactions.len(), 1);
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    assert!(process_diagram
        .step_thing_interactions
        .contains_key(&step_id));
}

#[test]
fn step_interaction_rename_noop_for_invalid_process_id() {
    let mut input_diagram =
        diagram_with_process_and_step_interactions("proc_0", &[("proc_0_step_0", &["edge_a"])]);

    StepInteractionCardOps::step_interaction_rename(
        &mut input_diagram,
        "",
        "proc_0_step_0",
        "proc_0_step_new",
        &["edge_a".to_owned()],
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    assert!(process_diagram
        .step_thing_interactions
        .contains_key(&step_id));
}

#[test]
fn step_interaction_rename_noop_for_invalid_old_step_id() {
    let mut input_diagram =
        diagram_with_process_and_step_interactions("proc_0", &[("proc_0_step_0", &["edge_a"])]);

    StepInteractionCardOps::step_interaction_rename(
        &mut input_diagram,
        "proc_0",
        "",
        "proc_0_step_new",
        &["edge_a".to_owned()],
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    assert_eq!(process_diagram.step_thing_interactions.len(), 1);
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    assert!(process_diagram
        .step_thing_interactions
        .contains_key(&step_id));
}

#[test]
fn step_interaction_rename_noop_for_invalid_new_step_id() {
    let mut input_diagram =
        diagram_with_process_and_step_interactions("proc_0", &[("proc_0_step_0", &["edge_a"])]);

    StepInteractionCardOps::step_interaction_rename(
        &mut input_diagram,
        "proc_0",
        "proc_0_step_0",
        "",
        &["edge_a".to_owned()],
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    assert_eq!(process_diagram.step_thing_interactions.len(), 1);
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    assert!(process_diagram
        .step_thing_interactions
        .contains_key(&step_id));
}

#[test]
fn step_interaction_rename_noop_for_missing_process() {
    let mut input_diagram = empty_diagram();

    StepInteractionCardOps::step_interaction_rename(
        &mut input_diagram,
        "proc_nonexistent",
        "step_old",
        "step_new",
        &[],
    );

    assert!(input_diagram.processes.is_empty());
}

// === step_interaction_edge_update === //

#[test]
fn step_interaction_edge_update_replaces_at_index() {
    let mut input_diagram = diagram_with_process_and_step_interactions(
        "proc_0",
        &[("proc_0_step_0", &["edge_a", "edge_b"])],
    );

    StepInteractionCardOps::step_interaction_edge_update(
        &mut input_diagram,
        "proc_0",
        "proc_0_step_0",
        0,
        "edge_x",
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let edge_group_ids = process_diagram
        .step_thing_interactions
        .get(&step_id)
        .unwrap();
    let edge_group_id_x = parse_edge_group_id("edge_x").unwrap();
    let edge_group_id_b = parse_edge_group_id("edge_b").unwrap();
    assert_eq!(edge_group_ids[0], edge_group_id_x);
    assert_eq!(edge_group_ids[1], edge_group_id_b);
}

#[test]
fn step_interaction_edge_update_noop_for_invalid_process_id() {
    let mut input_diagram =
        diagram_with_process_and_step_interactions("proc_0", &[("proc_0_step_0", &["edge_a"])]);

    StepInteractionCardOps::step_interaction_edge_update(
        &mut input_diagram,
        "",
        "proc_0_step_0",
        0,
        "edge_x",
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let edge_group_ids = process_diagram
        .step_thing_interactions
        .get(&step_id)
        .unwrap();
    let edge_group_id_a = parse_edge_group_id("edge_a").unwrap();
    assert_eq!(edge_group_ids[0], edge_group_id_a);
}

#[test]
fn step_interaction_edge_update_noop_for_invalid_step_id() {
    let mut input_diagram =
        diagram_with_process_and_step_interactions("proc_0", &[("proc_0_step_0", &["edge_a"])]);

    StepInteractionCardOps::step_interaction_edge_update(
        &mut input_diagram,
        "proc_0",
        "",
        0,
        "edge_x",
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let edge_group_ids = process_diagram
        .step_thing_interactions
        .get(&step_id)
        .unwrap();
    let edge_group_id_a = parse_edge_group_id("edge_a").unwrap();
    assert_eq!(edge_group_ids[0], edge_group_id_a);
}

#[test]
fn step_interaction_edge_update_noop_for_invalid_edge_group_id() {
    let mut input_diagram =
        diagram_with_process_and_step_interactions("proc_0", &[("proc_0_step_0", &["edge_a"])]);

    StepInteractionCardOps::step_interaction_edge_update(
        &mut input_diagram,
        "proc_0",
        "proc_0_step_0",
        0,
        "",
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let edge_group_ids = process_diagram
        .step_thing_interactions
        .get(&step_id)
        .unwrap();
    let edge_group_id_a = parse_edge_group_id("edge_a").unwrap();
    assert_eq!(edge_group_ids[0], edge_group_id_a);
}

#[test]
fn step_interaction_edge_update_noop_for_out_of_bounds_index() {
    let mut input_diagram =
        diagram_with_process_and_step_interactions("proc_0", &[("proc_0_step_0", &["edge_a"])]);

    StepInteractionCardOps::step_interaction_edge_update(
        &mut input_diagram,
        "proc_0",
        "proc_0_step_0",
        99,
        "edge_x",
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let edge_group_ids = process_diagram
        .step_thing_interactions
        .get(&step_id)
        .unwrap();
    assert_eq!(edge_group_ids.len(), 1);
}

// === step_interaction_edge_remove === //

#[test]
fn step_interaction_edge_remove_removes_at_index() {
    let mut input_diagram = diagram_with_process_and_step_interactions(
        "proc_0",
        &[("proc_0_step_0", &["edge_a", "edge_b"])],
    );

    StepInteractionCardOps::step_interaction_edge_remove(
        &mut input_diagram,
        "proc_0",
        "proc_0_step_0",
        0,
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let edge_group_ids = process_diagram
        .step_thing_interactions
        .get(&step_id)
        .unwrap();
    assert_eq!(edge_group_ids.len(), 1);
    let edge_group_id_b = parse_edge_group_id("edge_b").unwrap();
    assert_eq!(edge_group_ids[0], edge_group_id_b);
}

#[test]
fn step_interaction_edge_remove_noop_for_invalid_process_id() {
    let mut input_diagram =
        diagram_with_process_and_step_interactions("proc_0", &[("proc_0_step_0", &["edge_a"])]);

    StepInteractionCardOps::step_interaction_edge_remove(
        &mut input_diagram,
        "",
        "proc_0_step_0",
        0,
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let edge_group_ids = process_diagram
        .step_thing_interactions
        .get(&step_id)
        .unwrap();
    assert_eq!(edge_group_ids.len(), 1);
}

#[test]
fn step_interaction_edge_remove_noop_for_invalid_step_id() {
    let mut input_diagram =
        diagram_with_process_and_step_interactions("proc_0", &[("proc_0_step_0", &["edge_a"])]);

    StepInteractionCardOps::step_interaction_edge_remove(&mut input_diagram, "proc_0", "", 0);

    let process_id = parse_process_id("proc_0").unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let edge_group_ids = process_diagram
        .step_thing_interactions
        .get(&step_id)
        .unwrap();
    assert_eq!(edge_group_ids.len(), 1);
}

#[test]
fn step_interaction_edge_remove_noop_for_out_of_bounds_index() {
    let mut input_diagram =
        diagram_with_process_and_step_interactions("proc_0", &[("proc_0_step_0", &["edge_a"])]);

    StepInteractionCardOps::step_interaction_edge_remove(
        &mut input_diagram,
        "proc_0",
        "proc_0_step_0",
        99,
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let edge_group_ids = process_diagram
        .step_thing_interactions
        .get(&step_id)
        .unwrap();
    assert_eq!(edge_group_ids.len(), 1);
}

// === step_interaction_edge_add === //

#[test]
fn step_interaction_edge_add_adds_placeholder_from_existing_interaction() {
    let mut input_diagram =
        diagram_with_process_and_step_interactions("proc_0", &[("proc_0_step_0", &[])]);
    // Add an interaction edge group so the placeholder can pick it up.
    let edge_group_id_first = parse_edge_group_id("edge_first").unwrap();
    let edge_group = EdgeGroup::new(EdgeKind::Sequence, Vec::new());
    input_diagram
        .thing_interactions
        .insert(edge_group_id_first.clone(), edge_group);

    StepInteractionCardOps::step_interaction_edge_add(
        &mut input_diagram,
        "proc_0",
        "proc_0_step_0",
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let edge_group_ids = process_diagram
        .step_thing_interactions
        .get(&step_id)
        .unwrap();
    assert_eq!(edge_group_ids.len(), 1);
    assert_eq!(edge_group_ids[0], edge_group_id_first);
}

#[test]
fn step_interaction_edge_add_uses_edge_0_when_no_interactions_exist() {
    let mut input_diagram =
        diagram_with_process_and_step_interactions("proc_0", &[("proc_0_step_0", &[])]);

    StepInteractionCardOps::step_interaction_edge_add(
        &mut input_diagram,
        "proc_0",
        "proc_0_step_0",
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let edge_group_ids = process_diagram
        .step_thing_interactions
        .get(&step_id)
        .unwrap();
    assert_eq!(edge_group_ids.len(), 1);
    let edge_group_id_0 = parse_edge_group_id("edge_0").unwrap();
    assert_eq!(edge_group_ids[0], edge_group_id_0);
}

#[test]
fn step_interaction_edge_add_noop_for_invalid_process_id() {
    let mut input_diagram =
        diagram_with_process_and_step_interactions("proc_0", &[("proc_0_step_0", &[])]);

    StepInteractionCardOps::step_interaction_edge_add(&mut input_diagram, "", "proc_0_step_0");

    let process_id = parse_process_id("proc_0").unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let edge_group_ids = process_diagram
        .step_thing_interactions
        .get(&step_id)
        .unwrap();
    assert!(edge_group_ids.is_empty());
}

#[test]
fn step_interaction_edge_add_noop_for_invalid_step_id() {
    let mut input_diagram =
        diagram_with_process_and_step_interactions("proc_0", &[("proc_0_step_0", &[])]);

    StepInteractionCardOps::step_interaction_edge_add(&mut input_diagram, "proc_0", "");

    let process_id = parse_process_id("proc_0").unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let edge_group_ids = process_diagram
        .step_thing_interactions
        .get(&step_id)
        .unwrap();
    assert!(edge_group_ids.is_empty());
}

#[test]
fn step_interaction_edge_add_noop_for_missing_process() {
    let mut input_diagram = empty_diagram();

    StepInteractionCardOps::step_interaction_edge_add(
        &mut input_diagram,
        "proc_nonexistent",
        "step_0",
    );

    assert!(input_diagram.processes.is_empty());
}

#[test]
fn step_interaction_edge_add_noop_for_missing_step() {
    let mut input_diagram = diagram_with_process_and_step_interactions("proc_0", &[]);

    StepInteractionCardOps::step_interaction_edge_add(
        &mut input_diagram,
        "proc_0",
        "proc_0_step_nonexistent",
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    assert!(process_diagram.step_thing_interactions.is_empty());
}
