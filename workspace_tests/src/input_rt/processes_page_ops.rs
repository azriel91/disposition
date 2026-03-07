//! Tests for `disposition_input_rt::processes_page_ops::ProcessesPageOps`.

use disposition::input_model::{process::ProcessDiagram, InputDiagram};
use disposition_input_rt::{id_parse::parse_process_id, processes_page_ops::ProcessesPageOps};

fn empty_diagram() -> InputDiagram<'static> {
    InputDiagram::default()
}

// === process_add === //

#[test]
fn process_add_inserts_into_empty_diagram() {
    let mut input_diagram = empty_diagram();
    ProcessesPageOps::process_add(&mut input_diagram);

    let process_id = parse_process_id("proc_0").unwrap();
    assert_eq!(input_diagram.processes.len(), 1);
    assert!(input_diagram.processes.contains_key(&process_id));
}

#[test]
fn process_add_generates_unique_ids() {
    let mut input_diagram = empty_diagram();
    let process_id_0 = parse_process_id("proc_0").unwrap();
    input_diagram
        .processes
        .insert(process_id_0, ProcessDiagram::default());

    ProcessesPageOps::process_add(&mut input_diagram);

    let process_id_1 = parse_process_id("proc_1").unwrap();
    assert_eq!(input_diagram.processes.len(), 2);
    assert!(input_diagram.processes.contains_key(&process_id_1));
}

#[test]
fn process_add_skips_existing_ids() {
    let mut input_diagram = empty_diagram();
    for process_id_str in ["proc_0", "proc_1"] {
        let process_id = parse_process_id(process_id_str).unwrap();
        input_diagram
            .processes
            .insert(process_id, ProcessDiagram::default());
    }

    ProcessesPageOps::process_add(&mut input_diagram);

    let process_id_2 = parse_process_id("proc_2").unwrap();
    assert_eq!(input_diagram.processes.len(), 3);
    assert!(input_diagram.processes.contains_key(&process_id_2));
}

// === process_remove === //

#[test]
fn process_remove_removes_process() {
    let mut input_diagram = empty_diagram();
    let process_id_0 = parse_process_id("proc_0").unwrap();
    let process_id_1 = parse_process_id("proc_1").unwrap();
    input_diagram
        .processes
        .insert(process_id_0.clone(), ProcessDiagram::default());
    input_diagram
        .processes
        .insert(process_id_1.clone(), ProcessDiagram::default());

    ProcessesPageOps::process_remove(&mut input_diagram, "proc_0");

    assert!(!input_diagram.processes.contains_key(&process_id_0));
    assert!(input_diagram.processes.contains_key(&process_id_1));
}

#[test]
fn process_remove_noop_for_invalid_id() {
    let mut input_diagram = empty_diagram();
    let process_id = parse_process_id("proc_0").unwrap();
    input_diagram
        .processes
        .insert(process_id, ProcessDiagram::default());

    ProcessesPageOps::process_remove(&mut input_diagram, "");

    assert_eq!(input_diagram.processes.len(), 1);
}

#[test]
fn process_remove_noop_for_missing_id() {
    let mut input_diagram = empty_diagram();
    let process_id = parse_process_id("proc_0").unwrap();
    input_diagram
        .processes
        .insert(process_id, ProcessDiagram::default());

    ProcessesPageOps::process_remove(&mut input_diagram, "proc_nonexistent");

    assert_eq!(input_diagram.processes.len(), 1);
}

// === process_rename === //

#[test]
fn process_rename_renames_key() {
    let mut input_diagram = empty_diagram();
    let process_id_old = parse_process_id("proc_old").unwrap();
    input_diagram
        .processes
        .insert(process_id_old.clone(), ProcessDiagram::default());

    ProcessesPageOps::process_rename(&mut input_diagram, "proc_old", "proc_new");

    let process_id_new = parse_process_id("proc_new").unwrap();
    assert!(!input_diagram.processes.contains_key(&process_id_old));
    assert!(input_diagram.processes.contains_key(&process_id_new));
}

#[test]
fn process_rename_noop_when_same_id() {
    let mut input_diagram = empty_diagram();
    let process_id = parse_process_id("proc_0").unwrap();
    input_diagram
        .processes
        .insert(process_id.clone(), ProcessDiagram::default());

    ProcessesPageOps::process_rename(&mut input_diagram, "proc_0", "proc_0");

    assert_eq!(input_diagram.processes.len(), 1);
    assert!(input_diagram.processes.contains_key(&process_id));
}

#[test]
fn process_rename_noop_for_invalid_old_id() {
    let mut input_diagram = empty_diagram();
    let process_id = parse_process_id("proc_0").unwrap();
    input_diagram
        .processes
        .insert(process_id.clone(), ProcessDiagram::default());

    ProcessesPageOps::process_rename(&mut input_diagram, "", "proc_new");

    assert_eq!(input_diagram.processes.len(), 1);
    assert!(input_diagram.processes.contains_key(&process_id));
}

#[test]
fn process_rename_noop_for_invalid_new_id() {
    let mut input_diagram = empty_diagram();
    let process_id = parse_process_id("proc_0").unwrap();
    input_diagram
        .processes
        .insert(process_id.clone(), ProcessDiagram::default());

    ProcessesPageOps::process_rename(&mut input_diagram, "proc_0", "");

    assert_eq!(input_diagram.processes.len(), 1);
    assert!(input_diagram.processes.contains_key(&process_id));
}

// === process_move === //

#[test]
fn process_move_reorders_processes() {
    let mut input_diagram = empty_diagram();
    for process_id_str in ["proc_a", "proc_b", "proc_c"] {
        let process_id = parse_process_id(process_id_str).unwrap();
        input_diagram
            .processes
            .insert(process_id, ProcessDiagram::default());
    }

    ProcessesPageOps::process_move(&mut input_diagram, 0, 2);

    let keys: Vec<&str> = input_diagram.processes.keys().map(|k| k.as_str()).collect();
    assert_eq!(keys, vec!["proc_b", "proc_c", "proc_a"]);
}

// === process_name_update === //

#[test]
fn process_name_update_sets_name() {
    let mut input_diagram = empty_diagram();
    let process_id = parse_process_id("proc_0").unwrap();
    input_diagram
        .processes
        .insert(process_id.clone(), ProcessDiagram::default());

    ProcessesPageOps::process_name_update(&mut input_diagram, "proc_0", "My Process");

    assert_eq!(
        input_diagram
            .processes
            .get(&process_id)
            .unwrap()
            .name
            .as_deref(),
        Some("My Process")
    );
}

#[test]
fn process_name_update_clears_name_when_empty() {
    let mut input_diagram = empty_diagram();
    let process_id = parse_process_id("proc_0").unwrap();
    let mut process_diagram = ProcessDiagram::default();
    process_diagram.name = Some("Old Name".to_owned());
    input_diagram
        .processes
        .insert(process_id.clone(), process_diagram);

    ProcessesPageOps::process_name_update(&mut input_diagram, "proc_0", "");

    assert_eq!(input_diagram.processes.get(&process_id).unwrap().name, None);
}

#[test]
fn process_name_update_noop_for_missing_id() {
    let mut input_diagram = empty_diagram();

    ProcessesPageOps::process_name_update(&mut input_diagram, "proc_missing", "Name");

    assert!(input_diagram.processes.is_empty());
}

// === process_desc_update === //

#[test]
fn process_desc_update_sets_desc() {
    let mut input_diagram = empty_diagram();
    let process_id = parse_process_id("proc_0").unwrap();
    input_diagram
        .processes
        .insert(process_id.clone(), ProcessDiagram::default());

    ProcessesPageOps::process_desc_update(&mut input_diagram, "proc_0", "A description");

    assert_eq!(
        input_diagram
            .processes
            .get(&process_id)
            .unwrap()
            .desc
            .as_deref(),
        Some("A description")
    );
}

#[test]
fn process_desc_update_clears_desc_when_empty() {
    let mut input_diagram = empty_diagram();
    let process_id = parse_process_id("proc_0").unwrap();
    let mut process_diagram = ProcessDiagram::default();
    process_diagram.desc = Some("Old Desc".to_owned());
    input_diagram
        .processes
        .insert(process_id.clone(), process_diagram);

    ProcessesPageOps::process_desc_update(&mut input_diagram, "proc_0", "");

    assert_eq!(input_diagram.processes.get(&process_id).unwrap().desc, None);
}

#[test]
fn process_desc_update_noop_for_missing_id() {
    let mut input_diagram = empty_diagram();

    ProcessesPageOps::process_desc_update(&mut input_diagram, "proc_missing", "Desc");

    assert!(input_diagram.processes.is_empty());
}
