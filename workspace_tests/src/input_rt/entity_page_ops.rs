use disposition::{
    input_model::InputDiagram,
    input_rt::{
        id_parse::{parse_id, parse_thing_id},
        EntityPageOps, OnChangeTarget,
    },
};

fn empty_diagram() -> InputDiagram<'static> {
    InputDiagram::default()
}

// === thing_desc_add === //

#[test]
fn entity_desc_add_inserts_entry() {
    let mut input_diagram = empty_diagram();

    EntityPageOps::thing_desc_add(&mut input_diagram);

    assert_eq!(input_diagram.thing_descs.len(), 1);
}

// === entity_tooltip_add === //

#[test]
fn entity_tooltip_add_inserts_entry() {
    let mut input_diagram = empty_diagram();

    EntityPageOps::entity_tooltip_add(&mut input_diagram);

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

    EntityPageOps::kv_entry_update(
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
    let thing_id = parse_id("thing_0").unwrap();
    input_diagram
        .thing_descs
        .insert(thing_id.clone(), "old".to_owned());

    EntityPageOps::kv_entry_update(
        &mut input_diagram,
        OnChangeTarget::ThingDesc,
        "thing_0",
        "new",
    );

    assert_eq!(input_diagram.thing_descs.get(&thing_id).unwrap(), "new");
}

#[test]
fn kv_entry_update_entity_tooltip() {
    let mut input_diagram = empty_diagram();
    let entity_id = parse_id("entity_0").unwrap();
    input_diagram
        .entity_tooltips
        .insert(entity_id.clone(), "old".to_owned());

    EntityPageOps::kv_entry_update(
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

    EntityPageOps::kv_entry_remove(&mut input_diagram, OnChangeTarget::CopyText, "thing_0");

    assert!(!input_diagram.thing_copy_text.contains_key(&thing_id));
}

#[test]
fn kv_entry_remove_entity_desc() {
    let mut input_diagram = empty_diagram();
    let thing_id = parse_id("thing_0").unwrap();
    input_diagram
        .thing_descs
        .insert(thing_id.clone(), "desc".to_owned());

    EntityPageOps::kv_entry_remove(&mut input_diagram, OnChangeTarget::ThingDesc, "thing_0");

    assert!(!input_diagram.thing_descs.contains_key(&thing_id));
}

// === kv_entry_rename === //

#[test]
fn kv_entry_rename_copy_text() {
    let mut input_diagram = empty_diagram();
    let thing_id_old = parse_thing_id("thing_0").unwrap();
    input_diagram
        .thing_copy_text
        .insert(thing_id_old.clone(), "value".to_owned());

    EntityPageOps::kv_entry_rename(
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

    EntityPageOps::kv_entry_rename(
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

    EntityPageOps::kv_entry_move(&mut input_diagram, OnChangeTarget::CopyText, 0, 1);

    let keys: Vec<&str> = input_diagram
        .thing_copy_text
        .keys()
        .map(|k| k.as_str())
        .collect();
    assert_eq!(keys, vec!["thing_b", "thing_a"]);
}
