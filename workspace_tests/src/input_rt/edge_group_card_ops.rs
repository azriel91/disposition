//! Tests for `disposition_input_rt::edge_group_card_ops::EdgeGroupCardOps`.

use disposition::input_model::{
    edge::{EdgeGroup, EdgeKind},
    thing::ThingHierarchy,
    InputDiagram,
};
use disposition_input_rt::{
    edge_group_card_ops::EdgeGroupCardOps,
    id_parse::{parse_edge_group_id, parse_thing_id},
    map_target::MapTarget,
};

fn empty_diagram() -> InputDiagram<'static> {
    InputDiagram::default()
}

fn diagram_with_edge_group(
    target: MapTarget,
    edge_group_id_str: &str,
    edge_kind: EdgeKind,
    thing_id_strs: &[&str],
) -> InputDiagram<'static> {
    let mut input_diagram = empty_diagram();
    let edge_group_id = parse_edge_group_id(edge_group_id_str).unwrap();
    let thing_ids: Vec<_> = thing_id_strs
        .iter()
        .map(|s| parse_thing_id(s).unwrap())
        .collect();
    let edge_group = EdgeGroup::new(edge_kind, thing_ids);
    EdgeGroupCardOps::edge_group_set(&mut input_diagram, target, &edge_group_id, edge_group);
    input_diagram
}

fn diagram_with_things_and_edge_group(
    thing_names: &[(&str, &str)],
    target: MapTarget,
    edge_group_id_str: &str,
    edge_kind: EdgeKind,
    thing_id_strs: &[&str],
) -> InputDiagram<'static> {
    let mut input_diagram =
        diagram_with_edge_group(target, edge_group_id_str, edge_kind, thing_id_strs);
    for (thing_id_str, name) in thing_names {
        let thing_id = parse_thing_id(thing_id_str).unwrap();
        input_diagram
            .things
            .insert(thing_id.clone(), name.to_string());
        input_diagram
            .thing_hierarchy
            .insert(thing_id, ThingHierarchy::new());
    }
    input_diagram
}

// === edge_group_set === //

#[test]
fn edge_group_set_inserts_into_dependencies() {
    let mut input_diagram = empty_diagram();
    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    let edge_group = EdgeGroup::new(EdgeKind::Sequence, Vec::new());

    EdgeGroupCardOps::edge_group_set(
        &mut input_diagram,
        MapTarget::Dependencies,
        &edge_group_id,
        edge_group,
    );

    assert!(input_diagram
        .thing_dependencies
        .contains_key(&edge_group_id));
}

#[test]
fn edge_group_set_inserts_into_interactions() {
    let mut input_diagram = empty_diagram();
    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    let edge_group = EdgeGroup::new(EdgeKind::Sequence, Vec::new());

    EdgeGroupCardOps::edge_group_set(
        &mut input_diagram,
        MapTarget::Interactions,
        &edge_group_id,
        edge_group,
    );

    assert!(input_diagram
        .thing_interactions
        .contains_key(&edge_group_id));
}

// === edge_group_remove_by_id === //

#[test]
fn edge_group_remove_by_id_removes_from_dependencies() {
    let mut input_diagram =
        diagram_with_edge_group(MapTarget::Dependencies, "edge_0", EdgeKind::Sequence, &[]);
    let edge_group_id = parse_edge_group_id("edge_0").unwrap();

    EdgeGroupCardOps::edge_group_remove_by_id(
        &mut input_diagram,
        MapTarget::Dependencies,
        &edge_group_id,
    );

    assert!(!input_diagram
        .thing_dependencies
        .contains_key(&edge_group_id));
}

#[test]
fn edge_group_remove_by_id_removes_from_interactions() {
    let mut input_diagram =
        diagram_with_edge_group(MapTarget::Interactions, "edge_0", EdgeKind::Sequence, &[]);
    let edge_group_id = parse_edge_group_id("edge_0").unwrap();

    EdgeGroupCardOps::edge_group_remove_by_id(
        &mut input_diagram,
        MapTarget::Interactions,
        &edge_group_id,
    );

    assert!(!input_diagram
        .thing_interactions
        .contains_key(&edge_group_id));
}

// === edge_group_count === //

#[test]
fn edge_group_count_returns_zero_for_empty() {
    let input_diagram = empty_diagram();
    assert_eq!(
        EdgeGroupCardOps::edge_group_count(&input_diagram, MapTarget::Dependencies),
        0
    );
    assert_eq!(
        EdgeGroupCardOps::edge_group_count(&input_diagram, MapTarget::Interactions),
        0
    );
}

#[test]
fn edge_group_count_returns_correct_count() {
    let mut input_diagram = empty_diagram();
    let edge_group_id_0 = parse_edge_group_id("edge_0").unwrap();
    let edge_group_id_1 = parse_edge_group_id("edge_1").unwrap();
    let edge_group = EdgeGroup::new(EdgeKind::Sequence, Vec::new());

    EdgeGroupCardOps::edge_group_set(
        &mut input_diagram,
        MapTarget::Dependencies,
        &edge_group_id_0,
        edge_group.clone(),
    );
    EdgeGroupCardOps::edge_group_set(
        &mut input_diagram,
        MapTarget::Dependencies,
        &edge_group_id_1,
        edge_group,
    );

    assert_eq!(
        EdgeGroupCardOps::edge_group_count(&input_diagram, MapTarget::Dependencies),
        2
    );
}

// === edge_group_contains === //

#[test]
fn edge_group_contains_returns_true_when_present() {
    let input_diagram =
        diagram_with_edge_group(MapTarget::Dependencies, "edge_0", EdgeKind::Sequence, &[]);
    let edge_group_id = parse_edge_group_id("edge_0").unwrap();

    assert!(EdgeGroupCardOps::edge_group_contains(
        &input_diagram,
        MapTarget::Dependencies,
        &edge_group_id,
    ));
}

#[test]
fn edge_group_contains_returns_false_when_absent() {
    let input_diagram = empty_diagram();
    let edge_group_id = parse_edge_group_id("edge_0").unwrap();

    assert!(!EdgeGroupCardOps::edge_group_contains(
        &input_diagram,
        MapTarget::Dependencies,
        &edge_group_id,
    ));
}

// === edge_group_add === //

#[test]
fn edge_group_add_inserts_into_empty_dependencies() {
    let mut input_diagram = empty_diagram();

    EdgeGroupCardOps::edge_group_add(&mut input_diagram, MapTarget::Dependencies);

    assert_eq!(input_diagram.thing_dependencies.len(), 1);
    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    assert!(input_diagram
        .thing_dependencies
        .contains_key(&edge_group_id));
}

#[test]
fn edge_group_add_inserts_into_empty_interactions() {
    let mut input_diagram = empty_diagram();

    EdgeGroupCardOps::edge_group_add(&mut input_diagram, MapTarget::Interactions);

    assert_eq!(input_diagram.thing_interactions.len(), 1);
    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    assert!(input_diagram
        .thing_interactions
        .contains_key(&edge_group_id));
}

#[test]
fn edge_group_add_generates_unique_ids() {
    let mut input_diagram =
        diagram_with_edge_group(MapTarget::Dependencies, "edge_0", EdgeKind::Sequence, &[]);

    EdgeGroupCardOps::edge_group_add(&mut input_diagram, MapTarget::Dependencies);

    assert_eq!(input_diagram.thing_dependencies.len(), 2);
    let edge_group_id_1 = parse_edge_group_id("edge_1").unwrap();
    assert!(input_diagram
        .thing_dependencies
        .contains_key(&edge_group_id_1));
}

#[test]
fn edge_group_add_skips_existing_ids() {
    let mut input_diagram = empty_diagram();
    let edge_group_id_0 = parse_edge_group_id("edge_0").unwrap();
    let edge_group_id_1 = parse_edge_group_id("edge_1").unwrap();
    let edge_group = EdgeGroup::new(EdgeKind::Sequence, Vec::new());
    EdgeGroupCardOps::edge_group_set(
        &mut input_diagram,
        MapTarget::Dependencies,
        &edge_group_id_0,
        edge_group.clone(),
    );
    EdgeGroupCardOps::edge_group_set(
        &mut input_diagram,
        MapTarget::Dependencies,
        &edge_group_id_1,
        edge_group,
    );

    EdgeGroupCardOps::edge_group_add(&mut input_diagram, MapTarget::Dependencies);

    assert_eq!(input_diagram.thing_dependencies.len(), 3);
    let edge_group_id_2 = parse_edge_group_id("edge_2").unwrap();
    assert!(input_diagram
        .thing_dependencies
        .contains_key(&edge_group_id_2));
}

// === edge_group_move === //

#[test]
fn edge_group_move_reorders_dependencies() {
    let mut input_diagram = empty_diagram();
    for edge_group_id_str in ["edge_a", "edge_b", "edge_c"] {
        let edge_group_id = parse_edge_group_id(edge_group_id_str).unwrap();
        let edge_group = EdgeGroup::new(EdgeKind::Sequence, Vec::new());
        EdgeGroupCardOps::edge_group_set(
            &mut input_diagram,
            MapTarget::Dependencies,
            &edge_group_id,
            edge_group,
        );
    }

    EdgeGroupCardOps::edge_group_move(&mut input_diagram, MapTarget::Dependencies, 0, 2);

    let keys: Vec<&str> = input_diagram
        .thing_dependencies
        .keys()
        .map(|k| k.as_str())
        .collect();
    assert_eq!(keys, vec!["edge_b", "edge_c", "edge_a"]);
}

#[test]
fn edge_group_move_reorders_interactions() {
    let mut input_diagram = empty_diagram();
    for edge_group_id_str in ["edge_a", "edge_b", "edge_c"] {
        let edge_group_id = parse_edge_group_id(edge_group_id_str).unwrap();
        let edge_group = EdgeGroup::new(EdgeKind::Sequence, Vec::new());
        EdgeGroupCardOps::edge_group_set(
            &mut input_diagram,
            MapTarget::Interactions,
            &edge_group_id,
            edge_group,
        );
    }

    EdgeGroupCardOps::edge_group_move(&mut input_diagram, MapTarget::Interactions, 0, 2);

    let keys: Vec<&str> = input_diagram
        .thing_interactions
        .keys()
        .map(|k| k.as_str())
        .collect();
    assert_eq!(keys, vec!["edge_b", "edge_c", "edge_a"]);
}

// === edge_group_remove === //

#[test]
fn edge_group_remove_removes_by_str() {
    let mut input_diagram =
        diagram_with_edge_group(MapTarget::Dependencies, "edge_0", EdgeKind::Sequence, &[]);

    EdgeGroupCardOps::edge_group_remove(&mut input_diagram, MapTarget::Dependencies, "edge_0");

    assert!(input_diagram.thing_dependencies.is_empty());
}

#[test]
fn edge_group_remove_noop_for_invalid_id() {
    let mut input_diagram =
        diagram_with_edge_group(MapTarget::Dependencies, "edge_0", EdgeKind::Sequence, &[]);

    EdgeGroupCardOps::edge_group_remove(&mut input_diagram, MapTarget::Dependencies, "");

    assert_eq!(input_diagram.thing_dependencies.len(), 1);
}

#[test]
fn edge_group_remove_noop_for_missing_id() {
    let mut input_diagram =
        diagram_with_edge_group(MapTarget::Dependencies, "edge_0", EdgeKind::Sequence, &[]);

    EdgeGroupCardOps::edge_group_remove(
        &mut input_diagram,
        MapTarget::Dependencies,
        "edge_nonexistent",
    );

    assert_eq!(input_diagram.thing_dependencies.len(), 1);
}

// === edge_group_rename === //

#[test]
fn edge_group_rename_renames_key_in_dependencies() {
    let mut input_diagram =
        diagram_with_edge_group(MapTarget::Dependencies, "edge_old", EdgeKind::Sequence, &[]);

    EdgeGroupCardOps::edge_group_rename(&mut input_diagram, "edge_old", "edge_new");

    let edge_group_id_old = parse_edge_group_id("edge_old").unwrap();
    let edge_group_id_new = parse_edge_group_id("edge_new").unwrap();
    assert!(!input_diagram
        .thing_dependencies
        .contains_key(&edge_group_id_old));
    assert!(input_diagram
        .thing_dependencies
        .contains_key(&edge_group_id_new));
}

#[test]
fn edge_group_rename_renames_key_in_interactions() {
    let mut input_diagram =
        diagram_with_edge_group(MapTarget::Interactions, "edge_old", EdgeKind::Sequence, &[]);

    EdgeGroupCardOps::edge_group_rename(&mut input_diagram, "edge_old", "edge_new");

    let edge_group_id_old = parse_edge_group_id("edge_old").unwrap();
    let edge_group_id_new = parse_edge_group_id("edge_new").unwrap();
    assert!(!input_diagram
        .thing_interactions
        .contains_key(&edge_group_id_old));
    assert!(input_diagram
        .thing_interactions
        .contains_key(&edge_group_id_new));
}

#[test]
fn edge_group_rename_noop_when_same_id() {
    let mut input_diagram =
        diagram_with_edge_group(MapTarget::Dependencies, "edge_0", EdgeKind::Sequence, &[]);

    EdgeGroupCardOps::edge_group_rename(&mut input_diagram, "edge_0", "edge_0");

    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    assert!(input_diagram
        .thing_dependencies
        .contains_key(&edge_group_id));
}

#[test]
fn edge_group_rename_noop_for_invalid_old_id() {
    let mut input_diagram =
        diagram_with_edge_group(MapTarget::Dependencies, "edge_0", EdgeKind::Sequence, &[]);

    EdgeGroupCardOps::edge_group_rename(&mut input_diagram, "", "edge_new");

    assert_eq!(input_diagram.thing_dependencies.len(), 1);
    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    assert!(input_diagram
        .thing_dependencies
        .contains_key(&edge_group_id));
}

#[test]
fn edge_group_rename_noop_for_invalid_new_id() {
    let mut input_diagram =
        diagram_with_edge_group(MapTarget::Dependencies, "edge_0", EdgeKind::Sequence, &[]);

    EdgeGroupCardOps::edge_group_rename(&mut input_diagram, "edge_0", "");

    assert_eq!(input_diagram.thing_dependencies.len(), 1);
    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    assert!(input_diagram
        .thing_dependencies
        .contains_key(&edge_group_id));
}

#[test]
fn edge_group_rename_updates_step_thing_interactions() {
    let mut input_diagram = empty_diagram();
    let edge_group_id_old = parse_edge_group_id("edge_old").unwrap();
    let edge_group = EdgeGroup::new(EdgeKind::Sequence, Vec::new());
    input_diagram
        .thing_interactions
        .insert(edge_group_id_old.clone(), edge_group);

    // Add a process with a step interaction referencing the old edge group.
    let process_id = disposition_input_rt::id_parse::parse_process_id("proc_0").unwrap();
    let step_id = disposition_input_rt::id_parse::parse_process_step_id("proc_0_step_0").unwrap();
    let mut process_diagram = disposition::input_model::process::ProcessDiagram::default();
    process_diagram
        .step_thing_interactions
        .insert(step_id.clone(), vec![edge_group_id_old.clone()]);
    input_diagram
        .processes
        .insert(process_id.clone(), process_diagram);

    EdgeGroupCardOps::edge_group_rename(&mut input_diagram, "edge_old", "edge_new");

    let edge_group_id_new = parse_edge_group_id("edge_new").unwrap();
    let process_diagram = input_diagram.processes.get(&process_id).unwrap();
    let edge_group_ids = process_diagram
        .step_thing_interactions
        .get(&step_id)
        .unwrap();
    assert_eq!(edge_group_ids, &vec![edge_group_id_new]);
}

// === edge_kind_change === //

#[test]
fn edge_kind_change_updates_kind_and_preserves_things() {
    let thing_id_a = parse_thing_id("thing_a").unwrap();
    let thing_id_b = parse_thing_id("thing_b").unwrap();
    let mut input_diagram = diagram_with_edge_group(
        MapTarget::Dependencies,
        "edge_0",
        EdgeKind::Sequence,
        &["thing_a", "thing_b"],
    );

    EdgeGroupCardOps::edge_kind_change(
        &mut input_diagram,
        MapTarget::Dependencies,
        "edge_0",
        EdgeKind::Cyclic,
        &[thing_id_a.clone(), thing_id_b.clone()],
    );

    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    let edge_group = input_diagram
        .thing_dependencies
        .get(&edge_group_id)
        .unwrap();
    assert_eq!(edge_group.kind, EdgeKind::Cyclic);
    assert!(edge_group.things.contains(&thing_id_a));
    assert!(edge_group.things.contains(&thing_id_b));
}

#[test]
fn edge_kind_change_noop_for_invalid_edge_group_id() {
    let mut input_diagram =
        diagram_with_edge_group(MapTarget::Dependencies, "edge_0", EdgeKind::Sequence, &[]);

    EdgeGroupCardOps::edge_kind_change(
        &mut input_diagram,
        MapTarget::Dependencies,
        "",
        EdgeKind::Cyclic,
        &[],
    );

    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    let edge_group = input_diagram
        .thing_dependencies
        .get(&edge_group_id)
        .unwrap();
    assert_eq!(edge_group.kind, EdgeKind::Sequence);
}

// === edge_thing_update === //

#[test]
fn edge_thing_update_replaces_thing_at_index() {
    let mut input_diagram = diagram_with_edge_group(
        MapTarget::Dependencies,
        "edge_0",
        EdgeKind::Sequence,
        &["thing_a", "thing_b"],
    );

    EdgeGroupCardOps::edge_thing_update(
        &mut input_diagram,
        MapTarget::Dependencies,
        "edge_0",
        0,
        "thing_x",
    );

    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    let edge_group = input_diagram
        .thing_dependencies
        .get(&edge_group_id)
        .unwrap();
    let thing_id_x = parse_thing_id("thing_x").unwrap();
    let thing_id_a = parse_thing_id("thing_a").unwrap();
    let thing_id_b = parse_thing_id("thing_b").unwrap();
    assert_eq!(edge_group.things[0], thing_id_x);
    assert_eq!(edge_group.things[1], thing_id_b);
    assert!(!edge_group.things.contains(&thing_id_a));
}

#[test]
fn edge_thing_update_noop_for_invalid_edge_group_id() {
    let mut input_diagram = diagram_with_edge_group(
        MapTarget::Dependencies,
        "edge_0",
        EdgeKind::Sequence,
        &["thing_a"],
    );

    EdgeGroupCardOps::edge_thing_update(
        &mut input_diagram,
        MapTarget::Dependencies,
        "",
        0,
        "thing_x",
    );

    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    let edge_group = input_diagram
        .thing_dependencies
        .get(&edge_group_id)
        .unwrap();
    let thing_id_a = parse_thing_id("thing_a").unwrap();
    assert_eq!(edge_group.things[0], thing_id_a);
}

#[test]
fn edge_thing_update_noop_for_invalid_thing_id() {
    let mut input_diagram = diagram_with_edge_group(
        MapTarget::Dependencies,
        "edge_0",
        EdgeKind::Sequence,
        &["thing_a"],
    );

    EdgeGroupCardOps::edge_thing_update(
        &mut input_diagram,
        MapTarget::Dependencies,
        "edge_0",
        0,
        "",
    );

    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    let edge_group = input_diagram
        .thing_dependencies
        .get(&edge_group_id)
        .unwrap();
    let thing_id_a = parse_thing_id("thing_a").unwrap();
    assert_eq!(edge_group.things[0], thing_id_a);
}

#[test]
fn edge_thing_update_noop_for_out_of_bounds_index() {
    let mut input_diagram = diagram_with_edge_group(
        MapTarget::Dependencies,
        "edge_0",
        EdgeKind::Sequence,
        &["thing_a"],
    );

    EdgeGroupCardOps::edge_thing_update(
        &mut input_diagram,
        MapTarget::Dependencies,
        "edge_0",
        99,
        "thing_x",
    );

    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    let edge_group = input_diagram
        .thing_dependencies
        .get(&edge_group_id)
        .unwrap();
    assert_eq!(edge_group.things.len(), 1);
}

// === edge_thing_remove === //

#[test]
fn edge_thing_remove_removes_thing_at_index() {
    let mut input_diagram = diagram_with_edge_group(
        MapTarget::Dependencies,
        "edge_0",
        EdgeKind::Sequence,
        &["thing_a", "thing_b"],
    );

    EdgeGroupCardOps::edge_thing_remove(&mut input_diagram, MapTarget::Dependencies, "edge_0", 0);

    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    let edge_group = input_diagram
        .thing_dependencies
        .get(&edge_group_id)
        .unwrap();
    assert_eq!(edge_group.things.len(), 1);
    let thing_id_b = parse_thing_id("thing_b").unwrap();
    assert_eq!(edge_group.things[0], thing_id_b);
}

#[test]
fn edge_thing_remove_noop_for_invalid_edge_group_id() {
    let mut input_diagram = diagram_with_edge_group(
        MapTarget::Dependencies,
        "edge_0",
        EdgeKind::Sequence,
        &["thing_a"],
    );

    EdgeGroupCardOps::edge_thing_remove(&mut input_diagram, MapTarget::Dependencies, "", 0);

    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    let edge_group = input_diagram
        .thing_dependencies
        .get(&edge_group_id)
        .unwrap();
    assert_eq!(edge_group.things.len(), 1);
}

#[test]
fn edge_thing_remove_noop_for_out_of_bounds_index() {
    let mut input_diagram = diagram_with_edge_group(
        MapTarget::Dependencies,
        "edge_0",
        EdgeKind::Sequence,
        &["thing_a"],
    );

    EdgeGroupCardOps::edge_thing_remove(&mut input_diagram, MapTarget::Dependencies, "edge_0", 99);

    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    let edge_group = input_diagram
        .thing_dependencies
        .get(&edge_group_id)
        .unwrap();
    assert_eq!(edge_group.things.len(), 1);
}

// === edge_thing_move === //

#[test]
fn edge_thing_move_reorders_things() {
    let mut input_diagram = diagram_with_edge_group(
        MapTarget::Dependencies,
        "edge_0",
        EdgeKind::Sequence,
        &["thing_a", "thing_b", "thing_c"],
    );

    EdgeGroupCardOps::edge_thing_move(&mut input_diagram, MapTarget::Dependencies, "edge_0", 0, 2);

    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    let edge_group = input_diagram
        .thing_dependencies
        .get(&edge_group_id)
        .unwrap();
    let thing_id_a = parse_thing_id("thing_a").unwrap();
    let thing_id_b = parse_thing_id("thing_b").unwrap();
    let thing_id_c = parse_thing_id("thing_c").unwrap();
    assert_eq!(edge_group.things, vec![thing_id_b, thing_id_c, thing_id_a]);
}

#[test]
fn edge_thing_move_noop_when_from_equals_to() {
    let mut input_diagram = diagram_with_edge_group(
        MapTarget::Dependencies,
        "edge_0",
        EdgeKind::Sequence,
        &["thing_a", "thing_b"],
    );

    EdgeGroupCardOps::edge_thing_move(&mut input_diagram, MapTarget::Dependencies, "edge_0", 0, 0);

    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    let edge_group = input_diagram
        .thing_dependencies
        .get(&edge_group_id)
        .unwrap();
    let thing_id_a = parse_thing_id("thing_a").unwrap();
    let thing_id_b = parse_thing_id("thing_b").unwrap();
    assert_eq!(edge_group.things, vec![thing_id_a, thing_id_b]);
}

#[test]
fn edge_thing_move_noop_for_invalid_edge_group_id() {
    let mut input_diagram = diagram_with_edge_group(
        MapTarget::Dependencies,
        "edge_0",
        EdgeKind::Sequence,
        &["thing_a", "thing_b"],
    );

    EdgeGroupCardOps::edge_thing_move(&mut input_diagram, MapTarget::Dependencies, "", 0, 1);

    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    let edge_group = input_diagram
        .thing_dependencies
        .get(&edge_group_id)
        .unwrap();
    let thing_id_a = parse_thing_id("thing_a").unwrap();
    let thing_id_b = parse_thing_id("thing_b").unwrap();
    assert_eq!(edge_group.things, vec![thing_id_a, thing_id_b]);
}

#[test]
fn edge_thing_move_noop_for_out_of_bounds_from() {
    let mut input_diagram = diagram_with_edge_group(
        MapTarget::Dependencies,
        "edge_0",
        EdgeKind::Sequence,
        &["thing_a", "thing_b"],
    );

    EdgeGroupCardOps::edge_thing_move(&mut input_diagram, MapTarget::Dependencies, "edge_0", 99, 0);

    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    let edge_group = input_diagram
        .thing_dependencies
        .get(&edge_group_id)
        .unwrap();
    let thing_id_a = parse_thing_id("thing_a").unwrap();
    let thing_id_b = parse_thing_id("thing_b").unwrap();
    assert_eq!(edge_group.things, vec![thing_id_a, thing_id_b]);
}

// === edge_thing_add === //

#[test]
fn edge_thing_add_adds_placeholder_from_existing_thing() {
    let mut input_diagram = diagram_with_things_and_edge_group(
        &[("thing_first", "First")],
        MapTarget::Dependencies,
        "edge_0",
        EdgeKind::Sequence,
        &[],
    );

    EdgeGroupCardOps::edge_thing_add(&mut input_diagram, MapTarget::Dependencies, "edge_0");

    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    let edge_group = input_diagram
        .thing_dependencies
        .get(&edge_group_id)
        .unwrap();
    assert_eq!(edge_group.things.len(), 1);
    let thing_id_first = parse_thing_id("thing_first").unwrap();
    assert_eq!(edge_group.things[0], thing_id_first);
}

#[test]
fn edge_thing_add_uses_thing_0_when_no_things_exist() {
    let mut input_diagram =
        diagram_with_edge_group(MapTarget::Dependencies, "edge_0", EdgeKind::Sequence, &[]);

    EdgeGroupCardOps::edge_thing_add(&mut input_diagram, MapTarget::Dependencies, "edge_0");

    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    let edge_group = input_diagram
        .thing_dependencies
        .get(&edge_group_id)
        .unwrap();
    assert_eq!(edge_group.things.len(), 1);
    let thing_id_0 = parse_thing_id("thing_0").unwrap();
    assert_eq!(edge_group.things[0], thing_id_0);
}

#[test]
fn edge_thing_add_noop_for_invalid_edge_group_id() {
    let mut input_diagram =
        diagram_with_edge_group(MapTarget::Dependencies, "edge_0", EdgeKind::Sequence, &[]);

    EdgeGroupCardOps::edge_thing_add(&mut input_diagram, MapTarget::Dependencies, "");

    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    let edge_group = input_diagram
        .thing_dependencies
        .get(&edge_group_id)
        .unwrap();
    assert!(edge_group.things.is_empty());
}

#[test]
fn edge_thing_add_noop_for_missing_edge_group() {
    let mut input_diagram = empty_diagram();

    EdgeGroupCardOps::edge_thing_add(
        &mut input_diagram,
        MapTarget::Dependencies,
        "edge_nonexistent",
    );

    assert!(input_diagram.thing_dependencies.is_empty());
}
