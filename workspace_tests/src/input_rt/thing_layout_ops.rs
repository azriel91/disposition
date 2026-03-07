//! Tests for `disposition_input_rt::thing_layout_ops::ThingLayoutOps`.

use disposition::input_model::InputDiagram;
use disposition_input_rt::{flat_entry::hierarchy_flatten, thing_layout_ops::ThingLayoutOps};

fn empty_diagram() -> InputDiagram<'static> {
    InputDiagram::default()
}

fn diagram_with_hierarchy(hierarchy_yaml: &str) -> InputDiagram<'static> {
    let mut input_diagram = empty_diagram();
    input_diagram.thing_hierarchy = serde_saphyr::from_str(hierarchy_yaml)
        .unwrap_or_else(|e| panic!("Expected `hierarchy_yaml` to be valid: {}", e));

    input_diagram
}

/// Returns the flat thing IDs in order, for easy assertion.
fn flat_thing_ids(input_diagram: &InputDiagram<'static>) -> Vec<(String, usize)> {
    hierarchy_flatten(&input_diagram.thing_hierarchy)
        .into_iter()
        .map(|entry| (entry.thing_id.as_str().to_owned(), entry.depth))
        .collect()
}

// === entry_move_up === //

#[test]
fn entry_move_up_swaps_with_previous_sibling() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_a: {}
t_b: {}
t_c: {}",
    );

    let new_index = ThingLayoutOps::entry_move_up(&mut input_diagram, 1);

    assert_eq!(new_index, Some(0));
    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(&ids[0].0, "t_b");
    assert_eq!(&ids[1].0, "t_a");
    assert_eq!(&ids[2].0, "t_c");
}

#[test]
fn entry_move_up_swaps_subtrees_with_previous_sibling() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_a:
  t_a_child: {}
t_b: {}",
    );

    // Move t_b (index 2) up past t_a's subtree.
    let new_index = ThingLayoutOps::entry_move_up(&mut input_diagram, 2);

    assert_eq!(new_index, Some(0));
    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(ids[0], ("t_b".to_owned(), 0));
    assert_eq!(ids[1], ("t_a".to_owned(), 0));
    assert_eq!(ids[2], ("t_a_child".to_owned(), 1));
}

#[test]
fn entry_move_up_reparents_first_child_to_parent_level() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_parent:
  t_first_child: {}
  t_second_child: {}",
    );

    // t_first_child is at index 1, depth 1. No previous sibling at depth 1
    // before it within the parent, so it reparents up.
    let new_index = ThingLayoutOps::entry_move_up(&mut input_diagram, 1);

    assert_eq!(new_index, Some(0));
    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(ids[0], ("t_first_child".to_owned(), 0));
    assert_eq!(ids[1], ("t_parent".to_owned(), 0));
    assert_eq!(ids[2], ("t_second_child".to_owned(), 1));
}

#[test]
fn entry_move_up_noop_at_index_zero() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_a: {}
t_b: {}",
    );

    let new_index = ThingLayoutOps::entry_move_up(&mut input_diagram, 0);

    assert_eq!(new_index, None);
    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(&ids[0].0, "t_a");
    assert_eq!(&ids[1].0, "t_b");
}

#[test]
fn entry_move_up_noop_for_out_of_bounds_index() {
    let mut input_diagram = diagram_with_hierarchy("t_a: {}");

    let new_index = ThingLayoutOps::entry_move_up(&mut input_diagram, 99);

    assert_eq!(new_index, None);
}

#[test]
fn entry_move_up_noop_for_first_child_at_depth_zero() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_only: {}",
    );

    let new_index = ThingLayoutOps::entry_move_up(&mut input_diagram, 0);

    assert_eq!(new_index, None);
}

// === entry_move_down === //

#[test]
fn entry_move_down_swaps_with_next_sibling() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_a: {}
t_b: {}
t_c: {}",
    );

    let new_index = ThingLayoutOps::entry_move_down(&mut input_diagram, 0);

    assert_eq!(new_index, Some(1));
    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(&ids[0].0, "t_b");
    assert_eq!(&ids[1].0, "t_a");
    assert_eq!(&ids[2].0, "t_c");
}

#[test]
fn entry_move_down_swaps_subtrees_with_next_sibling() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_a: {}
t_b:
  t_b_child: {}",
    );

    // Move t_a (index 0) down past t_b's subtree.
    let new_index = ThingLayoutOps::entry_move_down(&mut input_diagram, 0);

    assert_eq!(new_index, Some(2));
    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(ids[0], ("t_b".to_owned(), 0));
    assert_eq!(ids[1], ("t_b_child".to_owned(), 1));
    assert_eq!(ids[2], ("t_a".to_owned(), 0));
}

#[test]
fn entry_move_down_reparents_last_child_to_parent_level() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_parent:
  t_first_child: {}
  t_last_child: {}",
    );

    // t_last_child is at index 2, depth 1. No next sibling, so it reparents
    // out of the parent's subtree.
    let new_index = ThingLayoutOps::entry_move_down(&mut input_diagram, 2);

    assert_eq!(new_index, Some(2));
    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(ids[0], ("t_parent".to_owned(), 0));
    assert_eq!(ids[1], ("t_first_child".to_owned(), 1));
    assert_eq!(ids[2], ("t_last_child".to_owned(), 0));
}

#[test]
fn entry_move_down_noop_for_last_top_level_entry() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_a: {}
t_b: {}",
    );

    let new_index = ThingLayoutOps::entry_move_down(&mut input_diagram, 1);

    assert_eq!(new_index, None);
}

#[test]
fn entry_move_down_noop_for_out_of_bounds_index() {
    let mut input_diagram = diagram_with_hierarchy("t_a: {}");

    let new_index = ThingLayoutOps::entry_move_down(&mut input_diagram, 99);

    assert_eq!(new_index, None);
}

// === entry_indent === //

#[test]
fn entry_indent_makes_entry_child_of_previous_sibling() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_a: {}
t_b: {}",
    );

    let new_index = ThingLayoutOps::entry_indent(&mut input_diagram, 1);

    assert_eq!(new_index, Some(1));
    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(ids[0], ("t_a".to_owned(), 0));
    assert_eq!(ids[1], ("t_b".to_owned(), 1));
}

#[test]
fn entry_indent_indents_subtree() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_a: {}
t_b:
  t_b_child: {}",
    );

    let new_index = ThingLayoutOps::entry_indent(&mut input_diagram, 1);

    assert_eq!(new_index, Some(1));
    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(ids[0], ("t_a".to_owned(), 0));
    assert_eq!(ids[1], ("t_b".to_owned(), 1));
    assert_eq!(ids[2], ("t_b_child".to_owned(), 2));
}

#[test]
fn entry_indent_noop_for_first_entry() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_a: {}
t_b: {}",
    );

    let new_index = ThingLayoutOps::entry_indent(&mut input_diagram, 0);

    assert_eq!(new_index, None);
    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(ids[0], ("t_a".to_owned(), 0));
    assert_eq!(ids[1], ("t_b".to_owned(), 0));
}

#[test]
fn entry_indent_noop_for_out_of_bounds_index() {
    let mut input_diagram = diagram_with_hierarchy("t_a: {}");

    let new_index = ThingLayoutOps::entry_indent(&mut input_diagram, 99);

    assert_eq!(new_index, None);
}

#[test]
fn entry_indent_noop_for_first_child_with_no_previous_sibling() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_parent:
  t_only_child: {}",
    );

    // t_only_child at index 1, depth 1 -- no previous sibling at depth 1.
    let new_index = ThingLayoutOps::entry_indent(&mut input_diagram, 1);

    assert_eq!(new_index, None);
    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(ids[0], ("t_parent".to_owned(), 0));
    assert_eq!(ids[1], ("t_only_child".to_owned(), 1));
}

// === entry_outdent === //

#[test]
fn entry_outdent_moves_child_to_parent_level() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_parent:
  t_child: {}",
    );

    let new_index = ThingLayoutOps::entry_outdent(&mut input_diagram, 1);

    assert_eq!(new_index, Some(1));
    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(ids[0], ("t_parent".to_owned(), 0));
    assert_eq!(ids[1], ("t_child".to_owned(), 0));
}

#[test]
fn entry_outdent_places_after_parent_subtree() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_parent:
  t_first: {}
  t_second: {}",
    );

    // Outdent t_first (index 1) -- it should move after the parent subtree,
    // but t_second stays as child of parent.
    let new_index = ThingLayoutOps::entry_outdent(&mut input_diagram, 1);

    assert_eq!(new_index, Some(2));
    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(ids[0], ("t_parent".to_owned(), 0));
    assert_eq!(ids[1], ("t_second".to_owned(), 1));
    assert_eq!(ids[2], ("t_first".to_owned(), 0));
}

#[test]
fn entry_outdent_outdents_subtree() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_parent:
  t_child:
    t_grandchild: {}",
    );

    // Outdent t_child (index 1, depth 1) -- t_grandchild goes with it.
    let new_index = ThingLayoutOps::entry_outdent(&mut input_diagram, 1);

    assert_eq!(new_index, Some(1));
    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(ids[0], ("t_parent".to_owned(), 0));
    assert_eq!(ids[1], ("t_child".to_owned(), 0));
    assert_eq!(ids[2], ("t_grandchild".to_owned(), 1));
}

#[test]
fn entry_outdent_noop_at_depth_zero() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_a: {}
t_b: {}",
    );

    let new_index = ThingLayoutOps::entry_outdent(&mut input_diagram, 0);

    assert_eq!(new_index, None);
    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(ids[0], ("t_a".to_owned(), 0));
    assert_eq!(ids[1], ("t_b".to_owned(), 0));
}

#[test]
fn entry_outdent_noop_for_out_of_bounds_index() {
    let mut input_diagram = diagram_with_hierarchy("t_a: {}");

    let new_index = ThingLayoutOps::entry_outdent(&mut input_diagram, 99);

    assert_eq!(new_index, None);
}

// === entry_drag_move === //

#[test]
fn entry_drag_move_moves_entry_forward() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_a: {}
t_b: {}
t_c: {}",
    );

    ThingLayoutOps::entry_drag_move(&mut input_diagram, 0, 2);

    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(&ids[0].0, "t_b");
    assert_eq!(&ids[1].0, "t_a");
    assert_eq!(&ids[2].0, "t_c");
}

#[test]
fn entry_drag_move_moves_entry_backward() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_a: {}
t_b: {}
t_c: {}",
    );

    ThingLayoutOps::entry_drag_move(&mut input_diagram, 2, 0);

    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(&ids[0].0, "t_c");
    assert_eq!(&ids[1].0, "t_a");
    assert_eq!(&ids[2].0, "t_b");
}

#[test]
fn entry_drag_move_adopts_target_depth() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_parent:
  t_child: {}
t_other: {}",
    );

    // Drag t_other (index 2, depth 0) to index 1 (t_child, depth 1).
    ThingLayoutOps::entry_drag_move(&mut input_diagram, 2, 1);

    let ids = flat_thing_ids(&input_diagram);
    // t_other should now be at depth 1 (adopted the target's depth).
    assert_eq!(ids[0], ("t_parent".to_owned(), 0));
    assert_eq!(ids[1], ("t_other".to_owned(), 1));
    assert_eq!(ids[2], ("t_child".to_owned(), 1));
}

#[test]
fn entry_drag_move_noop_when_from_equals_to() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_a: {}
t_b: {}",
    );

    ThingLayoutOps::entry_drag_move(&mut input_diagram, 0, 0);

    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(&ids[0].0, "t_a");
    assert_eq!(&ids[1].0, "t_b");
}

#[test]
fn entry_drag_move_noop_for_from_out_of_bounds() {
    let mut input_diagram = diagram_with_hierarchy("t_a: {}");

    ThingLayoutOps::entry_drag_move(&mut input_diagram, 99, 0);

    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(ids.len(), 1);
    assert_eq!(&ids[0].0, "t_a");
}

#[test]
fn entry_drag_move_noop_for_to_out_of_bounds() {
    let mut input_diagram = diagram_with_hierarchy("t_a: {}");

    ThingLayoutOps::entry_drag_move(&mut input_diagram, 0, 99);

    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(ids.len(), 1);
    assert_eq!(&ids[0].0, "t_a");
}

#[test]
fn entry_drag_move_noop_when_dropping_onto_own_subtree() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_parent:
  t_child: {}
t_other: {}",
    );

    // Try to drag t_parent (index 0) to index 1 (its own child).
    ThingLayoutOps::entry_drag_move(&mut input_diagram, 0, 1);

    // Nothing should change since index 1 is inside t_parent's subtree.
    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(ids[0], ("t_parent".to_owned(), 0));
    assert_eq!(ids[1], ("t_child".to_owned(), 1));
    assert_eq!(ids[2], ("t_other".to_owned(), 0));
}

#[test]
fn entry_drag_move_moves_subtree_together() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_a:
  t_a_child: {}
t_b: {}
t_c: {}",
    );

    // Drag t_a (index 0, with child at index 1) to index 3 (t_c).
    ThingLayoutOps::entry_drag_move(&mut input_diagram, 0, 3);

    let ids = flat_thing_ids(&input_diagram);
    // t_b should come first, then t_a and its child at t_c's depth.
    assert_eq!(&ids[0].0, "t_b");
    assert_eq!(&ids[1].0, "t_a");
    assert_eq!(&ids[2].0, "t_a_child");
}

// === Combined operations === //

#[test]
fn move_up_then_move_down_restores_original_order() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_a: {}
t_b: {}",
    );

    let new_index = ThingLayoutOps::entry_move_up(&mut input_diagram, 1);
    assert_eq!(new_index, Some(0));

    let new_index = ThingLayoutOps::entry_move_down(&mut input_diagram, 0);
    assert_eq!(new_index, Some(1));

    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(&ids[0].0, "t_a");
    assert_eq!(&ids[1].0, "t_b");
}

#[test]
fn indent_then_outdent_restores_original_depth() {
    let mut input_diagram = diagram_with_hierarchy(
        "\
t_a: {}
t_b: {}",
    );

    let new_index = ThingLayoutOps::entry_indent(&mut input_diagram, 1);
    assert_eq!(new_index, Some(1));
    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(ids[1], ("t_b".to_owned(), 1));

    let new_index = ThingLayoutOps::entry_outdent(&mut input_diagram, 1);
    assert_eq!(new_index, Some(1));
    let ids = flat_thing_ids(&input_diagram);
    assert_eq!(ids[1], ("t_b".to_owned(), 0));
}
