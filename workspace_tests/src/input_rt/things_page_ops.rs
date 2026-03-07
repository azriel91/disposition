//! Tests for `disposition_input_rt::things_page_ops::ThingsPageOps`.

use disposition::{
    input_model::{
        edge::{EdgeGroup, EdgeKind},
        thing::ThingHierarchy,
        InputDiagram,
    },
    model_common::Set,
};
use disposition_input_rt::{
    id_parse::{parse_edge_group_id, parse_id, parse_tag_id, parse_thing_id},
    on_change_target::OnChangeTarget,
    things_page_ops::ThingsPageOps,
};

fn empty_diagram() -> InputDiagram<'static> {
    InputDiagram::default()
}

fn diagram_with_things(names: &[(&str, &str)]) -> InputDiagram<'static> {
    let mut input_diagram = empty_diagram();
    for (thing_id_str, name) in names {
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

// === thing_add === //

#[test]
fn thing_add_inserts_into_empty_diagram() {
    let mut input_diagram = empty_diagram();

    ThingsPageOps::thing_add(&mut input_diagram);

    let thing_id = parse_thing_id("thing_0").unwrap();
    assert_eq!(input_diagram.things.len(), 1);
    assert!(input_diagram.things.contains_key(&thing_id));
    // Also inserted into hierarchy.
    assert_eq!(input_diagram.thing_hierarchy.len(), 1);
}

#[test]
fn thing_add_generates_unique_ids() {
    let mut input_diagram = diagram_with_things(&[("thing_0", "First")]);

    ThingsPageOps::thing_add(&mut input_diagram);

    let thing_id_1 = parse_thing_id("thing_1").unwrap();
    assert_eq!(input_diagram.things.len(), 2);
    assert!(input_diagram.things.contains_key(&thing_id_1));
}

#[test]
fn thing_add_skips_existing_ids() {
    let mut input_diagram = diagram_with_things(&[("thing_0", "A"), ("thing_1", "B")]);

    ThingsPageOps::thing_add(&mut input_diagram);

    let thing_id_2 = parse_thing_id("thing_2").unwrap();
    assert_eq!(input_diagram.things.len(), 3);
    assert!(input_diagram.things.contains_key(&thing_id_2));
}

// === thing_name_update === //

#[test]
fn thing_name_update_changes_name() {
    let mut input_diagram = diagram_with_things(&[("thing_0", "Old Name")]);

    ThingsPageOps::thing_name_update(&mut input_diagram, "thing_0", "New Name");

    let thing_id = parse_thing_id("thing_0").unwrap();
    assert_eq!(input_diagram.things.get(&thing_id).unwrap(), "New Name");
}

#[test]
fn thing_name_update_noop_for_missing_id() {
    let mut input_diagram = empty_diagram();

    ThingsPageOps::thing_name_update(&mut input_diagram, "nonexistent", "value");

    assert!(input_diagram.things.is_empty());
}

// === thing_rename === //

#[test]
fn thing_rename_renames_key_in_things() {
    let mut input_diagram = diagram_with_things(&[("thing_0", "Hello")]);

    ThingsPageOps::thing_rename(&mut input_diagram, "thing_0", "thing_renamed");

    let thing_id_old = parse_thing_id("thing_0").unwrap();
    let thing_id_new = parse_thing_id("thing_renamed").unwrap();
    assert!(!input_diagram.things.contains_key(&thing_id_old));
    assert!(input_diagram.things.contains_key(&thing_id_new));
    assert_eq!(input_diagram.things.get(&thing_id_new).unwrap(), "Hello");
}

#[test]
fn thing_rename_updates_hierarchy() {
    let mut input_diagram = diagram_with_things(&[("thing_0", "Hello")]);

    ThingsPageOps::thing_rename(&mut input_diagram, "thing_0", "thing_new");

    let thing_id_old = parse_thing_id("thing_0").unwrap();
    let thing_id_new = parse_thing_id("thing_new").unwrap();
    assert!(!input_diagram.thing_hierarchy.contains_key(&thing_id_old));
    assert!(input_diagram.thing_hierarchy.contains_key(&thing_id_new));
}

#[test]
fn thing_rename_updates_edge_groups() {
    let mut input_diagram = diagram_with_things(&[("thing_a", "A"), ("thing_b", "B")]);
    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    let thing_id_a = parse_thing_id("thing_a").unwrap();
    let thing_id_b = parse_thing_id("thing_b").unwrap();
    input_diagram.thing_dependencies.insert(
        edge_group_id.clone(),
        EdgeGroup::new(
            EdgeKind::Sequence,
            vec![thing_id_a.clone(), thing_id_b.clone()],
        ),
    );

    ThingsPageOps::thing_rename(&mut input_diagram, "thing_a", "thing_x");

    let edge_group = input_diagram
        .thing_dependencies
        .get(&edge_group_id)
        .unwrap();
    let thing_id_x = parse_thing_id("thing_x").unwrap();
    assert!(edge_group.things.contains(&thing_id_x));
    assert!(!edge_group.things.contains(&thing_id_a));
    // thing_b should still be there.
    assert!(edge_group.things.contains(&thing_id_b));
}

#[test]
fn thing_rename_noop_when_same_id() {
    let mut input_diagram = diagram_with_things(&[("thing_0", "Same")]);

    ThingsPageOps::thing_rename(&mut input_diagram, "thing_0", "thing_0");

    let thing_id = parse_thing_id("thing_0").unwrap();
    assert!(input_diagram.things.contains_key(&thing_id));
}

#[test]
fn thing_rename_noop_for_invalid_id() {
    let mut input_diagram = diagram_with_things(&[("thing_0", "Valid")]);

    // Empty string is not a valid ID.
    ThingsPageOps::thing_rename(&mut input_diagram, "thing_0", "");

    // Original should still be present.
    let thing_id = parse_thing_id("thing_0").unwrap();
    assert!(input_diagram.things.contains_key(&thing_id));
}

// === thing_remove === //

#[test]
fn thing_remove_removes_thing_and_hierarchy() {
    let mut input_diagram = diagram_with_things(&[("thing_0", "Gone"), ("thing_1", "Stays")]);

    ThingsPageOps::thing_remove(&mut input_diagram, "thing_0");

    let thing_id_removed = parse_thing_id("thing_0").unwrap();
    let thing_id_kept = parse_thing_id("thing_1").unwrap();
    assert!(!input_diagram.things.contains_key(&thing_id_removed));
    assert!(input_diagram.things.contains_key(&thing_id_kept));
    assert!(!input_diagram
        .thing_hierarchy
        .contains_key(&thing_id_removed));
    assert!(input_diagram.thing_hierarchy.contains_key(&thing_id_kept));
}

#[test]
fn thing_remove_cleans_up_edge_groups() {
    let mut input_diagram = diagram_with_things(&[("thing_a", "A"), ("thing_b", "B")]);
    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    let thing_id_a = parse_thing_id("thing_a").unwrap();
    let thing_id_b = parse_thing_id("thing_b").unwrap();
    input_diagram.thing_dependencies.insert(
        edge_group_id.clone(),
        EdgeGroup::new(
            EdgeKind::Sequence,
            vec![thing_id_a.clone(), thing_id_b.clone()],
        ),
    );

    ThingsPageOps::thing_remove(&mut input_diagram, "thing_a");

    let edge_group = input_diagram
        .thing_dependencies
        .get(&edge_group_id)
        .unwrap();
    assert!(!edge_group.things.contains(&thing_id_a));
    assert!(edge_group.things.contains(&thing_id_b));
}

#[test]
fn thing_remove_noop_for_invalid_id() {
    let mut input_diagram = diagram_with_things(&[("thing_0", "Keep")]);

    ThingsPageOps::thing_remove(&mut input_diagram, "");

    assert_eq!(input_diagram.things.len(), 1);
}

#[test]
fn thing_remove_cleans_up_tag_things() {
    let mut input_diagram = diagram_with_things(&[("thing_a", "A"), ("thing_b", "B")]);
    let tag_id = parse_tag_id("tag_0").unwrap();
    let thing_id_a = parse_thing_id("thing_a").unwrap();
    let thing_id_b = parse_thing_id("thing_b").unwrap();
    let mut thing_ids = Set::new();
    thing_ids.insert(thing_id_a.clone());
    thing_ids.insert(thing_id_b.clone());
    input_diagram.tag_things.insert(tag_id.clone(), thing_ids);

    ThingsPageOps::thing_remove(&mut input_diagram, "thing_a");

    let thing_ids = input_diagram.tag_things.get(&tag_id).unwrap();
    assert!(!thing_ids.contains(&thing_id_a));
    assert!(thing_ids.contains(&thing_id_b));
}

// === thing_move === //

#[test]
fn thing_move_reorders_things() {
    let mut input_diagram =
        diagram_with_things(&[("thing_a", "A"), ("thing_b", "B"), ("thing_c", "C")]);

    ThingsPageOps::thing_move(&mut input_diagram, 0, 2);

    let keys: Vec<&str> = input_diagram.things.keys().map(|k| k.as_str()).collect();
    assert_eq!(keys, vec!["thing_b", "thing_c", "thing_a"]);
}

// === thing_duplicate === //

#[test]
fn thing_duplicate_creates_copy_with_incremented_suffix() {
    let mut input_diagram = diagram_with_things(&[("thing_1", "Original")]);

    ThingsPageOps::thing_duplicate(&mut input_diagram, "thing_1");

    let thing_id_dup = parse_thing_id("thing_2").unwrap();
    assert_eq!(input_diagram.things.len(), 2);
    assert!(input_diagram.things.contains_key(&thing_id_dup));
    assert_eq!(input_diagram.things.get(&thing_id_dup).unwrap(), "Original");
}

#[test]
fn thing_duplicate_appends_copy_suffix_when_no_trailing_number() {
    let mut input_diagram = diagram_with_things(&[("thing_foo", "Foo")]);

    ThingsPageOps::thing_duplicate(&mut input_diagram, "thing_foo");

    let thing_id_dup = parse_thing_id("thing_foo_copy_1").unwrap();
    assert_eq!(input_diagram.things.len(), 2);
    assert!(input_diagram.things.contains_key(&thing_id_dup));
}

#[test]
fn thing_duplicate_skips_existing_ids() {
    let mut input_diagram = diagram_with_things(&[("thing_1", "A"), ("thing_2", "B")]);

    ThingsPageOps::thing_duplicate(&mut input_diagram, "thing_1");

    let thing_id_dup = parse_thing_id("thing_3").unwrap();
    assert_eq!(input_diagram.things.len(), 3);
    assert!(input_diagram.things.contains_key(&thing_id_dup));
}

#[test]
fn thing_duplicate_inserts_into_hierarchy() {
    let mut input_diagram = diagram_with_things(&[("thing_1", "Hello")]);

    ThingsPageOps::thing_duplicate(&mut input_diagram, "thing_1");

    let thing_id_dup = parse_thing_id("thing_2").unwrap();
    assert!(input_diagram.thing_hierarchy.contains_key(&thing_id_dup));
}

#[test]
fn thing_duplicate_copies_into_edge_groups() {
    let mut input_diagram = diagram_with_things(&[("thing_1", "A"), ("thing_other", "B")]);
    let edge_group_id = parse_edge_group_id("edge_0").unwrap();
    let thing_id_1 = parse_thing_id("thing_1").unwrap();
    let thing_id_other = parse_thing_id("thing_other").unwrap();
    input_diagram.thing_dependencies.insert(
        edge_group_id.clone(),
        EdgeGroup::new(
            EdgeKind::Sequence,
            vec![thing_id_1.clone(), thing_id_other.clone()],
        ),
    );

    ThingsPageOps::thing_duplicate(&mut input_diagram, "thing_1");

    let edge_group = input_diagram
        .thing_dependencies
        .get(&edge_group_id)
        .unwrap();
    let thing_id_2 = parse_thing_id("thing_2").unwrap();
    assert!(edge_group.things.contains(&thing_id_1));
    assert!(edge_group.things.contains(&thing_id_2));
    assert!(edge_group.things.contains(&thing_id_other));
}

#[test]
fn thing_duplicate_noop_for_invalid_id() {
    let mut input_diagram = diagram_with_things(&[("thing_0", "Only")]);

    ThingsPageOps::thing_duplicate(&mut input_diagram, "");

    assert_eq!(input_diagram.things.len(), 1);
}

#[test]
fn thing_duplicate_preserves_leading_zeroes() {
    let mut input_diagram = diagram_with_things(&[("thing_001", "Padded")]);

    ThingsPageOps::thing_duplicate(&mut input_diagram, "thing_001");

    let thing_id_dup = parse_thing_id("thing_002").unwrap();
    assert_eq!(input_diagram.things.len(), 2);
    assert!(
        input_diagram.things.contains_key(&thing_id_dup),
        "expected thing_002 to exist, got keys: {:?}",
        input_diagram
            .things
            .keys()
            .map(|k| k.as_str())
            .collect::<Vec<_>>()
    );
}

// === copy_text_add === //

#[test]
fn copy_text_add_inserts_entry() {
    let mut input_diagram = empty_diagram();

    ThingsPageOps::copy_text_add(&mut input_diagram);

    assert_eq!(input_diagram.thing_copy_text.len(), 1);
}

// === entity_desc_add === //

#[test]
fn entity_desc_add_inserts_entry() {
    let mut input_diagram = empty_diagram();

    ThingsPageOps::entity_desc_add(&mut input_diagram);

    assert_eq!(input_diagram.entity_descs.len(), 1);
}

// === entity_tooltip_add === //

#[test]
fn entity_tooltip_add_inserts_entry() {
    let mut input_diagram = empty_diagram();

    ThingsPageOps::entity_tooltip_add(&mut input_diagram);

    assert_eq!(input_diagram.entity_tooltips.len(), 1);
}

// === kv_entry_update === //

#[test]
fn kv_entry_update_copy_text() {
    let mut input_diagram = empty_diagram();
    let thing_id = parse_thing_id("thing_0").unwrap();
    input_diagram
        .thing_copy_text
        .insert(thing_id.clone(), "old".to_owned());

    ThingsPageOps::kv_entry_update(
        &mut input_diagram,
        OnChangeTarget::CopyText,
        "thing_0",
        "new",
    );

    assert_eq!(input_diagram.thing_copy_text.get(&thing_id).unwrap(), "new");
}

#[test]
fn kv_entry_update_entity_desc() {
    let mut input_diagram = empty_diagram();
    let entity_id = parse_id("entity_0").unwrap();
    input_diagram
        .entity_descs
        .insert(entity_id.clone(), "old".to_owned());

    ThingsPageOps::kv_entry_update(
        &mut input_diagram,
        OnChangeTarget::EntityDesc,
        "entity_0",
        "new",
    );

    assert_eq!(input_diagram.entity_descs.get(&entity_id).unwrap(), "new");
}

#[test]
fn kv_entry_update_entity_tooltip() {
    let mut input_diagram = empty_diagram();
    let entity_id = parse_id("entity_0").unwrap();
    input_diagram
        .entity_tooltips
        .insert(entity_id.clone(), "old".to_owned());

    ThingsPageOps::kv_entry_update(
        &mut input_diagram,
        OnChangeTarget::EntityTooltip,
        "entity_0",
        "new",
    );

    assert_eq!(
        input_diagram.entity_tooltips.get(&entity_id).unwrap(),
        "new"
    );
}

// === kv_entry_remove === //

#[test]
fn kv_entry_remove_copy_text() {
    let mut input_diagram = empty_diagram();
    let thing_id = parse_thing_id("thing_0").unwrap();
    input_diagram
        .thing_copy_text
        .insert(thing_id.clone(), "text".to_owned());

    ThingsPageOps::kv_entry_remove(&mut input_diagram, OnChangeTarget::CopyText, "thing_0");

    assert!(!input_diagram.thing_copy_text.contains_key(&thing_id));
}

#[test]
fn kv_entry_remove_entity_desc() {
    let mut input_diagram = empty_diagram();
    let entity_id = parse_id("entity_0").unwrap();
    input_diagram
        .entity_descs
        .insert(entity_id.clone(), "desc".to_owned());

    ThingsPageOps::kv_entry_remove(&mut input_diagram, OnChangeTarget::EntityDesc, "entity_0");

    assert!(!input_diagram.entity_descs.contains_key(&entity_id));
}

// === kv_entry_rename === //

#[test]
fn kv_entry_rename_copy_text() {
    let mut input_diagram = empty_diagram();
    let thing_id_old = parse_thing_id("thing_0").unwrap();
    input_diagram
        .thing_copy_text
        .insert(thing_id_old.clone(), "value".to_owned());

    ThingsPageOps::kv_entry_rename(
        &mut input_diagram,
        OnChangeTarget::CopyText,
        "thing_0",
        "thing_new",
        "value",
    );

    let thing_id_new = parse_thing_id("thing_new").unwrap();
    assert!(!input_diagram.thing_copy_text.contains_key(&thing_id_old));
    assert_eq!(
        input_diagram.thing_copy_text.get(&thing_id_new).unwrap(),
        "value"
    );
}

#[test]
fn kv_entry_rename_noop_when_same() {
    let mut input_diagram = empty_diagram();
    let thing_id = parse_thing_id("thing_0").unwrap();
    input_diagram
        .thing_copy_text
        .insert(thing_id.clone(), "val".to_owned());

    ThingsPageOps::kv_entry_rename(
        &mut input_diagram,
        OnChangeTarget::CopyText,
        "thing_0",
        "thing_0",
        "val",
    );

    assert_eq!(input_diagram.thing_copy_text.len(), 1);
    assert!(input_diagram.thing_copy_text.contains_key(&thing_id));
}

// === kv_entry_move === //

#[test]
fn kv_entry_move_reorders_copy_text() {
    let mut input_diagram = empty_diagram();
    let thing_id_a = parse_thing_id("thing_a").unwrap();
    let thing_id_b = parse_thing_id("thing_b").unwrap();
    input_diagram
        .thing_copy_text
        .insert(thing_id_a, "a".to_owned());
    input_diagram
        .thing_copy_text
        .insert(thing_id_b, "b".to_owned());

    ThingsPageOps::kv_entry_move(&mut input_diagram, OnChangeTarget::CopyText, 0, 1);

    let keys: Vec<&str> = input_diagram
        .thing_copy_text
        .keys()
        .map(|k| k.as_str())
        .collect();
    assert_eq!(keys, vec!["thing_b", "thing_a"]);
}
