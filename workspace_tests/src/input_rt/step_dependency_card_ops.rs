//! Tests for `disposition_input_rt::step_dependency_card_ops::StepDependencyCardOps`.

use disposition::{
    input_model::{process::ProcessDiagram, InputDiagram},
    model_common::Set,
};
use disposition_input_rt::{
    id_parse::{parse_process_id, parse_process_step_id},
    ProcessCardOps, StepDependencyCardOps,
};

fn empty_diagram() -> InputDiagram<'static> {
    InputDiagram::default()
}

/// Builds a diagram with a single process whose `steps` map contains the given
/// step labels, and whose `process_step_dependencies` map contains the given
/// dependency entries.
fn diagram_with_process_and_step_dependencies(
    process_id_str: &str,
    step_id_strs: &[&str],
    step_dependencies: &[(&str, &[&str])],
) -> InputDiagram<'static> {
    let mut input_diagram = empty_diagram();
    let process_id = parse_process_id(process_id_str).unwrap();
    let mut process_diagram = ProcessDiagram::default();
    for step_id_str in step_id_strs {
        let step_id = parse_process_step_id(step_id_str).unwrap();
        process_diagram.steps.insert(step_id, String::new());
    }
    for (step_id_str, dep_id_strs) in step_dependencies {
        let step_id = parse_process_step_id(step_id_str).unwrap();
        let dep_ids: Set<_> = dep_id_strs
            .iter()
            .map(|s| parse_process_step_id(s).unwrap())
            .collect();
        process_diagram
            .process_step_dependencies
            .insert(step_id, dep_ids);
    }
    input_diagram.processes.insert(process_id, process_diagram);
    input_diagram
}

// === step_dependency_remove === //

#[test]
fn step_dependency_remove_removes_entry() {
    let mut input_diagram = diagram_with_process_and_step_dependencies(
        "proc_0",
        &["proc_0_step_0", "proc_0_step_1"],
        &[
            ("proc_0_step_0", &["proc_0_step_1"]),
            ("proc_0_step_1", &[]),
        ],
    );

    StepDependencyCardOps::step_dependency_remove(&mut input_diagram, "proc_0", "proc_0_step_0");

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let step_id_0 = parse_process_step_id("proc_0_step_0").unwrap();
    let step_id_1 = parse_process_step_id("proc_0_step_1").unwrap();
    assert!(!process_diagram
        .process_step_dependencies
        .contains_key(&step_id_0));
    assert!(process_diagram
        .process_step_dependencies
        .contains_key(&step_id_1));
}

#[test]
fn step_dependency_remove_noop_for_invalid_process_id() {
    let mut input_diagram = diagram_with_process_and_step_dependencies(
        "proc_0",
        &["proc_0_step_0"],
        &[("proc_0_step_0", &[])],
    );

    StepDependencyCardOps::step_dependency_remove(&mut input_diagram, "", "proc_0_step_0");

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    assert_eq!(process_diagram.process_step_dependencies.len(), 1);
}

#[test]
fn step_dependency_remove_noop_for_missing_process() {
    let mut input_diagram = empty_diagram();

    StepDependencyCardOps::step_dependency_remove(&mut input_diagram, "proc_nonexistent", "step_0");

    assert!(input_diagram.processes.is_empty());
}

// === step_dependency_rename === //

#[test]
fn step_dependency_rename_renames_step_key_preserving_deps() {
    let mut input_diagram = diagram_with_process_and_step_dependencies(
        "proc_0",
        &["proc_0_step_old", "proc_0_step_a", "proc_0_step_b"],
        &[("proc_0_step_old", &["proc_0_step_a", "proc_0_step_b"])],
    );

    StepDependencyCardOps::step_dependency_rename(
        &mut input_diagram,
        "proc_0",
        "proc_0_step_old",
        "proc_0_step_new",
        &["proc_0_step_a".to_owned(), "proc_0_step_b".to_owned()],
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let step_id_old = parse_process_step_id("proc_0_step_old").unwrap();
    let step_id_new = parse_process_step_id("proc_0_step_new").unwrap();
    assert!(!process_diagram
        .process_step_dependencies
        .contains_key(&step_id_old));

    let dep_ids = process_diagram
        .process_step_dependencies
        .get(&step_id_new)
        .unwrap();
    let dep_id_a = parse_process_step_id("proc_0_step_a").unwrap();
    let dep_id_b = parse_process_step_id("proc_0_step_b").unwrap();
    assert!(dep_ids.contains(&dep_id_a));
    assert!(dep_ids.contains(&dep_id_b));
    assert_eq!(dep_ids.len(), 2);
}

#[test]
fn step_dependency_rename_noop_when_same_id() {
    let mut input_diagram = diagram_with_process_and_step_dependencies(
        "proc_0",
        &["proc_0_step_0", "proc_0_step_1"],
        &[("proc_0_step_0", &["proc_0_step_1"])],
    );

    StepDependencyCardOps::step_dependency_rename(
        &mut input_diagram,
        "proc_0",
        "proc_0_step_0",
        "proc_0_step_0",
        &["proc_0_step_1".to_owned()],
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    assert_eq!(process_diagram.process_step_dependencies.len(), 1);
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    assert!(process_diagram
        .process_step_dependencies
        .contains_key(&step_id));
}

#[test]
fn step_dependency_rename_noop_for_invalid_new_step_id() {
    let mut input_diagram = diagram_with_process_and_step_dependencies(
        "proc_0",
        &["proc_0_step_0"],
        &[("proc_0_step_0", &[])],
    );

    StepDependencyCardOps::step_dependency_rename(
        &mut input_diagram,
        "proc_0",
        "proc_0_step_0",
        "",
        &[],
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    assert!(process_diagram
        .process_step_dependencies
        .contains_key(&step_id));
}

// === step_dependency_dep_update === //

#[test]
fn step_dependency_dep_update_replaces_at_index() {
    let mut input_diagram = diagram_with_process_and_step_dependencies(
        "proc_0",
        &[
            "proc_0_step_0",
            "proc_0_step_a",
            "proc_0_step_b",
            "proc_0_step_x",
        ],
        &[("proc_0_step_0", &["proc_0_step_a", "proc_0_step_b"])],
    );

    StepDependencyCardOps::step_dependency_dep_update(
        &mut input_diagram,
        "proc_0",
        "proc_0_step_0",
        0,
        "proc_0_step_x",
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let dep_ids = process_diagram
        .process_step_dependencies
        .get(&step_id)
        .unwrap();
    let dep_id_x = parse_process_step_id("proc_0_step_x").unwrap();
    let dep_id_b = parse_process_step_id("proc_0_step_b").unwrap();
    assert_eq!(dep_ids.get_index(0), Some(&dep_id_x));
    assert_eq!(dep_ids.get_index(1), Some(&dep_id_b));
}

#[test]
fn step_dependency_dep_update_noop_for_out_of_bounds_index() {
    let mut input_diagram = diagram_with_process_and_step_dependencies(
        "proc_0",
        &["proc_0_step_0", "proc_0_step_a"],
        &[("proc_0_step_0", &["proc_0_step_a"])],
    );

    StepDependencyCardOps::step_dependency_dep_update(
        &mut input_diagram,
        "proc_0",
        "proc_0_step_0",
        99,
        "proc_0_step_x",
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let dep_ids = process_diagram
        .process_step_dependencies
        .get(&step_id)
        .unwrap();
    assert_eq!(dep_ids.len(), 1);
}

// === step_dependency_dep_remove === //

#[test]
fn step_dependency_dep_remove_removes_at_index() {
    let mut input_diagram = diagram_with_process_and_step_dependencies(
        "proc_0",
        &["proc_0_step_0", "proc_0_step_a", "proc_0_step_b"],
        &[("proc_0_step_0", &["proc_0_step_a", "proc_0_step_b"])],
    );

    StepDependencyCardOps::step_dependency_dep_remove(
        &mut input_diagram,
        "proc_0",
        "proc_0_step_0",
        0,
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let dep_ids = process_diagram
        .process_step_dependencies
        .get(&step_id)
        .unwrap();
    assert_eq!(dep_ids.len(), 1);
    let dep_id_b = parse_process_step_id("proc_0_step_b").unwrap();
    assert_eq!(dep_ids.get_index(0), Some(&dep_id_b));
}

// === step_dependency_dep_move === //

#[test]
fn step_dependency_dep_move_repositions_entry() {
    let mut input_diagram = diagram_with_process_and_step_dependencies(
        "proc_0",
        &[
            "proc_0_step_0",
            "proc_0_step_a",
            "proc_0_step_b",
            "proc_0_step_c",
        ],
        &[(
            "proc_0_step_0",
            &["proc_0_step_a", "proc_0_step_b", "proc_0_step_c"],
        )],
    );

    StepDependencyCardOps::step_dependency_dep_move(
        &mut input_diagram,
        "proc_0",
        "proc_0_step_0",
        0,
        2,
    );

    let process_id = parse_process_id("proc_0").unwrap();
    let step_id = parse_process_step_id("proc_0_step_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let dep_ids = process_diagram
        .process_step_dependencies
        .get(&step_id)
        .unwrap();
    let dep_id_a = parse_process_step_id("proc_0_step_a").unwrap();
    assert_eq!(dep_ids.get_index(2), Some(&dep_id_a));
}

// === step_dependency_dep_add === //

#[test]
fn step_dependency_dep_add_uses_first_other_step() {
    let mut input_diagram = diagram_with_process_and_step_dependencies(
        "proc_0",
        &["proc_0_step_0", "proc_0_step_1"],
        &[("proc_0_step_1", &[])],
    );

    StepDependencyCardOps::step_dependency_dep_add(&mut input_diagram, "proc_0", "proc_0_step_1");

    let process_id = parse_process_id("proc_0").unwrap();
    let step_id = parse_process_step_id("proc_0_step_1").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let dep_ids = process_diagram
        .process_step_dependencies
        .get(&step_id)
        .unwrap();
    let dep_id_0 = parse_process_step_id("proc_0_step_0").unwrap();
    assert_eq!(dep_ids.len(), 1);
    assert!(dep_ids.contains(&dep_id_0));
}

#[test]
fn step_dependency_dep_add_skips_self_and_existing() {
    let mut input_diagram = diagram_with_process_and_step_dependencies(
        "proc_0",
        &["proc_0_step_0", "proc_0_step_1", "proc_0_step_2"],
        &[("proc_0_step_2", &["proc_0_step_0"])],
    );

    StepDependencyCardOps::step_dependency_dep_add(&mut input_diagram, "proc_0", "proc_0_step_2");

    let process_id = parse_process_id("proc_0").unwrap();
    let step_id = parse_process_step_id("proc_0_step_2").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let dep_ids = process_diagram
        .process_step_dependencies
        .get(&step_id)
        .unwrap();
    let dep_id_1 = parse_process_step_id("proc_0_step_1").unwrap();
    assert_eq!(dep_ids.len(), 2);
    assert!(dep_ids.contains(&dep_id_1));
}

// === ProcessCardOps::step_dependency_add === //

#[test]
fn process_step_dependency_add_uses_first_unmapped_step() {
    let mut input_diagram = diagram_with_process_and_step_dependencies(
        "proc_0",
        &["proc_0_step_0", "proc_0_step_1"],
        &[("proc_0_step_0", &[])],
    );

    ProcessCardOps::step_dependency_add(&mut input_diagram, "proc_0");

    let process_id = parse_process_id("proc_0").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let step_id_1 = parse_process_step_id("proc_0_step_1").unwrap();
    assert!(process_diagram
        .process_step_dependencies
        .contains_key(&step_id_1));
    assert_eq!(process_diagram.process_step_dependencies.len(), 2);
}

#[test]
fn process_step_dependency_add_noop_for_missing_process() {
    let mut input_diagram = empty_diagram();

    ProcessCardOps::step_dependency_add(&mut input_diagram, "proc_nonexistent");

    assert!(input_diagram.processes.is_empty());
}
