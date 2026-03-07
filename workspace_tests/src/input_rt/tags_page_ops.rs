//! Tests for `disposition_input_rt::tags_page_ops::TagsPageOps`.

use disposition::{input_model::InputDiagram, model_common::Set};
use disposition_input_rt::{
    id_parse::{parse_tag_id, parse_thing_id},
    TagsPageOps,
};

fn empty_diagram() -> InputDiagram<'static> {
    InputDiagram::default()
}

// === tag_add === //

#[test]
fn tag_add_inserts_into_empty_diagram() {
    let mut input_diagram = empty_diagram();

    TagsPageOps::tag_add(&mut input_diagram);

    let tag_id = parse_tag_id("tag_0").unwrap();
    assert_eq!(input_diagram.tags.len(), 1);
    assert!(input_diagram.tags.contains_key(&tag_id));
}

#[test]
fn tag_add_generates_unique_ids() {
    let mut input_diagram = empty_diagram();
    let tag_id_0 = parse_tag_id("tag_0").unwrap();
    input_diagram.tags.insert(tag_id_0, "First".to_owned());

    TagsPageOps::tag_add(&mut input_diagram);

    let tag_id_1 = parse_tag_id("tag_1").unwrap();
    assert_eq!(input_diagram.tags.len(), 2);
    assert!(input_diagram.tags.contains_key(&tag_id_1));
}

#[test]
fn tag_add_skips_existing_ids() {
    let mut input_diagram = empty_diagram();
    let tag_id_0 = parse_tag_id("tag_0").unwrap();
    let tag_id_1 = parse_tag_id("tag_1").unwrap();
    input_diagram.tags.insert(tag_id_0, "A".to_owned());
    input_diagram.tags.insert(tag_id_1, "B".to_owned());

    TagsPageOps::tag_add(&mut input_diagram);

    let tag_id_2 = parse_tag_id("tag_2").unwrap();
    assert_eq!(input_diagram.tags.len(), 3);
    assert!(input_diagram.tags.contains_key(&tag_id_2));
}

// === tag_remove === //

#[test]
fn tag_remove_removes_tag() {
    let mut input_diagram = empty_diagram();
    let tag_id_0 = parse_tag_id("tag_0").unwrap();
    let tag_id_1 = parse_tag_id("tag_1").unwrap();
    input_diagram
        .tags
        .insert(tag_id_0.clone(), "Gone".to_owned());
    input_diagram
        .tags
        .insert(tag_id_1.clone(), "Stays".to_owned());

    TagsPageOps::tag_remove(&mut input_diagram, "tag_0");

    assert!(!input_diagram.tags.contains_key(&tag_id_0));
    assert!(input_diagram.tags.contains_key(&tag_id_1));
}

#[test]
fn tag_remove_noop_for_invalid_id() {
    let mut input_diagram = empty_diagram();
    let tag_id = parse_tag_id("tag_0").unwrap();
    input_diagram.tags.insert(tag_id, "Keep".to_owned());

    TagsPageOps::tag_remove(&mut input_diagram, "");

    assert_eq!(input_diagram.tags.len(), 1);
}

#[test]
fn tag_remove_noop_for_missing_id() {
    let mut input_diagram = empty_diagram();
    let tag_id = parse_tag_id("tag_0").unwrap();
    input_diagram.tags.insert(tag_id, "Keep".to_owned());

    TagsPageOps::tag_remove(&mut input_diagram, "tag_nonexistent");

    assert_eq!(input_diagram.tags.len(), 1);
}

// === tag_rename === //

#[test]
fn tag_rename_renames_key_in_tags() {
    let mut input_diagram = empty_diagram();
    let tag_id_old = parse_tag_id("tag_old").unwrap();
    input_diagram
        .tags
        .insert(tag_id_old.clone(), "Name".to_owned());

    TagsPageOps::tag_rename(&mut input_diagram, "tag_old", "tag_new");

    let tag_id_new = parse_tag_id("tag_new").unwrap();
    assert!(!input_diagram.tags.contains_key(&tag_id_old));
    assert!(input_diagram.tags.contains_key(&tag_id_new));
    assert_eq!(input_diagram.tags.get(&tag_id_new).unwrap(), "Name");
}

#[test]
fn tag_rename_renames_key_in_tag_things() {
    let mut input_diagram = empty_diagram();
    let tag_id_old = parse_tag_id("tag_old").unwrap();
    input_diagram
        .tags
        .insert(tag_id_old.clone(), "Name".to_owned());
    input_diagram
        .tag_things
        .insert(tag_id_old.clone(), Set::new());

    TagsPageOps::tag_rename(&mut input_diagram, "tag_old", "tag_new");

    let tag_id_new = parse_tag_id("tag_new").unwrap();
    assert!(!input_diagram.tag_things.contains_key(&tag_id_old));
    assert!(input_diagram.tag_things.contains_key(&tag_id_new));
}

#[test]
fn tag_rename_noop_when_same_id() {
    let mut input_diagram = empty_diagram();
    let tag_id = parse_tag_id("tag_0").unwrap();
    input_diagram.tags.insert(tag_id.clone(), "Same".to_owned());

    TagsPageOps::tag_rename(&mut input_diagram, "tag_0", "tag_0");

    assert_eq!(input_diagram.tags.len(), 1);
    assert!(input_diagram.tags.contains_key(&tag_id));
}

#[test]
fn tag_rename_noop_for_invalid_old_id() {
    let mut input_diagram = empty_diagram();
    let tag_id = parse_tag_id("tag_0").unwrap();
    input_diagram.tags.insert(tag_id.clone(), "Keep".to_owned());

    TagsPageOps::tag_rename(&mut input_diagram, "", "tag_new");

    assert_eq!(input_diagram.tags.len(), 1);
    assert!(input_diagram.tags.contains_key(&tag_id));
}

#[test]
fn tag_rename_noop_for_invalid_new_id() {
    let mut input_diagram = empty_diagram();
    let tag_id = parse_tag_id("tag_0").unwrap();
    input_diagram.tags.insert(tag_id.clone(), "Keep".to_owned());

    TagsPageOps::tag_rename(&mut input_diagram, "tag_0", "");

    assert_eq!(input_diagram.tags.len(), 1);
    assert!(input_diagram.tags.contains_key(&tag_id));
}

// === tag_name_update === //

#[test]
fn tag_name_update_changes_name() {
    let mut input_diagram = empty_diagram();
    let tag_id = parse_tag_id("tag_0").unwrap();
    input_diagram.tags.insert(tag_id.clone(), "Old".to_owned());

    TagsPageOps::tag_name_update(&mut input_diagram, "tag_0", "New");

    assert_eq!(input_diagram.tags.get(&tag_id).unwrap(), "New");
}

#[test]
fn tag_name_update_noop_for_missing_id() {
    let mut input_diagram = empty_diagram();

    TagsPageOps::tag_name_update(&mut input_diagram, "tag_missing", "val");

    assert!(input_diagram.tags.is_empty());
}

// === tag_move === //

#[test]
fn tag_move_reorders_tags() {
    let mut input_diagram = empty_diagram();
    let tag_id_a = parse_tag_id("tag_a").unwrap();
    let tag_id_b = parse_tag_id("tag_b").unwrap();
    let tag_id_c = parse_tag_id("tag_c").unwrap();
    input_diagram.tags.insert(tag_id_a, "A".to_owned());
    input_diagram.tags.insert(tag_id_b, "B".to_owned());
    input_diagram.tags.insert(tag_id_c, "C".to_owned());

    TagsPageOps::tag_move(&mut input_diagram, 0, 2);

    let keys: Vec<&str> = input_diagram.tags.keys().map(|k| k.as_str()).collect();
    assert_eq!(keys, vec!["tag_b", "tag_c", "tag_a"]);
}

// === tag_things_entry_add === //

#[test]
fn tag_things_entry_add_picks_unmapped_tag() {
    let mut input_diagram = empty_diagram();
    let tag_id_0 = parse_tag_id("tag_0").unwrap();
    let tag_id_1 = parse_tag_id("tag_1").unwrap();
    input_diagram
        .tags
        .insert(tag_id_0.clone(), "Zero".to_owned());
    input_diagram
        .tags
        .insert(tag_id_1.clone(), "One".to_owned());
    // Map tag_0 but not tag_1.
    input_diagram.tag_things.insert(tag_id_0, Set::new());

    TagsPageOps::tag_things_entry_add(&mut input_diagram);

    assert_eq!(input_diagram.tag_things.len(), 2);
    assert!(input_diagram.tag_things.contains_key(&tag_id_1));
}

#[test]
fn tag_things_entry_add_generates_placeholder_when_all_mapped() {
    let mut input_diagram = empty_diagram();
    let tag_id_0 = parse_tag_id("tag_0").unwrap();
    input_diagram
        .tags
        .insert(tag_id_0.clone(), "Zero".to_owned());
    input_diagram.tag_things.insert(tag_id_0, Set::new());

    TagsPageOps::tag_things_entry_add(&mut input_diagram);

    // Should have generated tag_1.
    let tag_id_1 = parse_tag_id("tag_1").unwrap();
    assert_eq!(input_diagram.tag_things.len(), 2);
    assert!(input_diagram.tag_things.contains_key(&tag_id_1));
}

#[test]
fn tag_things_entry_add_into_empty() {
    let mut input_diagram = empty_diagram();

    TagsPageOps::tag_things_entry_add(&mut input_diagram);

    assert_eq!(input_diagram.tag_things.len(), 1);
}

// === tag_things_entry_move === //

#[test]
fn tag_things_entry_move_reorders() {
    let mut input_diagram = empty_diagram();
    let tag_id_a = parse_tag_id("tag_a").unwrap();
    let tag_id_b = parse_tag_id("tag_b").unwrap();
    input_diagram.tag_things.insert(tag_id_a, Set::new());
    input_diagram.tag_things.insert(tag_id_b, Set::new());

    TagsPageOps::tag_things_entry_move(&mut input_diagram, 0, 1);

    let keys: Vec<&str> = input_diagram
        .tag_things
        .keys()
        .map(|k| k.as_str())
        .collect();
    assert_eq!(keys, vec!["tag_b", "tag_a"]);
}

// === tag_things_entry_remove === //

#[test]
fn tag_things_entry_remove_removes_entry() {
    let mut input_diagram = empty_diagram();
    let tag_id = parse_tag_id("tag_0").unwrap();
    input_diagram.tag_things.insert(tag_id, Set::new());

    TagsPageOps::tag_things_entry_remove(&mut input_diagram, "tag_0");

    assert!(input_diagram.tag_things.is_empty());
}

#[test]
fn tag_things_entry_remove_noop_for_missing() {
    let mut input_diagram = empty_diagram();
    let tag_id = parse_tag_id("tag_0").unwrap();
    input_diagram.tag_things.insert(tag_id, Set::new());

    TagsPageOps::tag_things_entry_remove(&mut input_diagram, "tag_nonexistent");

    assert_eq!(input_diagram.tag_things.len(), 1);
}

// === tag_things_entry_rename === //

#[test]
fn tag_things_entry_rename_renames_key() {
    let mut input_diagram = empty_diagram();
    let tag_id_old = parse_tag_id("tag_old").unwrap();
    let thing_id = parse_thing_id("thing_0").unwrap();
    let mut thing_ids = Set::new();
    thing_ids.insert(thing_id.clone());
    input_diagram
        .tag_things
        .insert(tag_id_old.clone(), thing_ids);

    TagsPageOps::tag_things_entry_rename(
        &mut input_diagram,
        "tag_old",
        "tag_new",
        &["thing_0".to_owned()],
    );

    let tag_id_new = parse_tag_id("tag_new").unwrap();
    assert!(!input_diagram.tag_things.contains_key(&tag_id_old));
    assert!(input_diagram.tag_things.contains_key(&tag_id_new));
    let thing_ids = input_diagram.tag_things.get(&tag_id_new).unwrap();
    assert!(thing_ids.contains(&thing_id));
}

#[test]
fn tag_things_entry_rename_noop_when_same() {
    let mut input_diagram = empty_diagram();
    let tag_id = parse_tag_id("tag_0").unwrap();
    input_diagram.tag_things.insert(tag_id.clone(), Set::new());

    TagsPageOps::tag_things_entry_rename(&mut input_diagram, "tag_0", "tag_0", &[]);

    assert_eq!(input_diagram.tag_things.len(), 1);
    assert!(input_diagram.tag_things.contains_key(&tag_id));
}

// === tag_things_thing_add === //

#[test]
fn tag_things_thing_add_adds_thing() {
    let mut input_diagram = empty_diagram();
    let tag_id = parse_tag_id("tag_0").unwrap();
    let thing_id = parse_thing_id("thing_0").unwrap();
    input_diagram
        .things
        .insert(thing_id.clone(), "First".to_owned());
    input_diagram.tag_things.insert(tag_id.clone(), Set::new());

    TagsPageOps::tag_things_thing_add(&mut input_diagram, "tag_0");

    let thing_ids = input_diagram.tag_things.get(&tag_id).unwrap();
    assert_eq!(thing_ids.len(), 1);
    assert!(thing_ids.contains(&thing_id));
}

#[test]
fn tag_things_thing_add_uses_placeholder_when_no_things_exist() {
    let mut input_diagram = empty_diagram();
    let tag_id = parse_tag_id("tag_0").unwrap();
    input_diagram.tag_things.insert(tag_id.clone(), Set::new());

    TagsPageOps::tag_things_thing_add(&mut input_diagram, "tag_0");

    let thing_ids = input_diagram.tag_things.get(&tag_id).unwrap();
    assert_eq!(thing_ids.len(), 1);
    // Placeholder is "thing_0".
    let thing_id_placeholder = parse_thing_id("thing_0").unwrap();
    assert!(thing_ids.contains(&thing_id_placeholder));
}

#[test]
fn tag_things_thing_add_noop_for_missing_tag() {
    let mut input_diagram = empty_diagram();

    TagsPageOps::tag_things_thing_add(&mut input_diagram, "tag_nonexistent");

    assert!(input_diagram.tag_things.is_empty());
}

// === tag_things_thing_update === //

#[test]
fn tag_things_thing_update_replaces_at_index() {
    let mut input_diagram = empty_diagram();
    let tag_id = parse_tag_id("tag_0").unwrap();
    let thing_id_a = parse_thing_id("thing_a").unwrap();
    let thing_id_b = parse_thing_id("thing_b").unwrap();
    let mut thing_ids = Set::new();
    thing_ids.insert(thing_id_a.clone());
    thing_ids.insert(thing_id_b.clone());
    input_diagram.tag_things.insert(tag_id.clone(), thing_ids);

    TagsPageOps::tag_things_thing_update(&mut input_diagram, "tag_0", 0, "thing_x");

    let thing_ids = input_diagram.tag_things.get(&tag_id).unwrap();
    let thing_id_x = parse_thing_id("thing_x").unwrap();
    assert!(thing_ids.contains(&thing_id_x));
    assert!(!thing_ids.contains(&thing_id_a));
    assert!(thing_ids.contains(&thing_id_b));
}

#[test]
fn tag_things_thing_update_noop_for_invalid_new_id() {
    let mut input_diagram = empty_diagram();
    let tag_id = parse_tag_id("tag_0").unwrap();
    let thing_id_a = parse_thing_id("thing_a").unwrap();
    let mut thing_ids = Set::new();
    thing_ids.insert(thing_id_a.clone());
    input_diagram.tag_things.insert(tag_id.clone(), thing_ids);

    TagsPageOps::tag_things_thing_update(&mut input_diagram, "tag_0", 0, "");

    let thing_ids = input_diagram.tag_things.get(&tag_id).unwrap();
    // Original still present because "" is an invalid ID.
    assert!(thing_ids.contains(&thing_id_a));
}

// === tag_things_thing_remove === //

#[test]
fn tag_things_thing_remove_removes_by_index() {
    let mut input_diagram = empty_diagram();
    let tag_id = parse_tag_id("tag_0").unwrap();
    let thing_id_a = parse_thing_id("thing_a").unwrap();
    let thing_id_b = parse_thing_id("thing_b").unwrap();
    let mut thing_ids = Set::new();
    thing_ids.insert(thing_id_a.clone());
    thing_ids.insert(thing_id_b.clone());
    input_diagram.tag_things.insert(tag_id.clone(), thing_ids);

    TagsPageOps::tag_things_thing_remove(&mut input_diagram, "tag_0", 0);

    let thing_ids = input_diagram.tag_things.get(&tag_id).unwrap();
    assert_eq!(thing_ids.len(), 1);
    assert!(!thing_ids.contains(&thing_id_a));
    assert!(thing_ids.contains(&thing_id_b));
}

#[test]
fn tag_things_thing_remove_noop_for_out_of_bounds_index() {
    let mut input_diagram = empty_diagram();
    let tag_id = parse_tag_id("tag_0").unwrap();
    let thing_id_a = parse_thing_id("thing_a").unwrap();
    let mut thing_ids = Set::new();
    thing_ids.insert(thing_id_a);
    input_diagram.tag_things.insert(tag_id.clone(), thing_ids);

    TagsPageOps::tag_things_thing_remove(&mut input_diagram, "tag_0", 99);

    let thing_ids = input_diagram.tag_things.get(&tag_id).unwrap();
    assert_eq!(thing_ids.len(), 1);
}

#[test]
fn tag_things_thing_remove_noop_for_missing_tag() {
    let mut input_diagram = empty_diagram();

    TagsPageOps::tag_things_thing_remove(&mut input_diagram, "tag_nonexistent", 0);

    assert!(input_diagram.tag_things.is_empty());
}
