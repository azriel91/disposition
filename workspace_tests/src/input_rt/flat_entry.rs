//! Tests for `disposition_input_rt::flat_entry`.

use disposition::input_model::thing::ThingHierarchy;
use disposition_input_rt::flat_entry::{hierarchy_flatten, hierarchy_rebuild};

fn hierarchy_deserialize(hierarchy_yaml: &str) -> ThingHierarchy<'static> {
    serde_saphyr::from_str(hierarchy_yaml)
        .unwrap_or_else(|e| panic!("Expected `hierarchy_yaml` to be valid: {}", e))
}

#[test]
fn flatten_empty() {
    let thing_hierarchy = ThingHierarchy::new();
    let flat_entries = hierarchy_flatten(&thing_hierarchy);
    assert!(flat_entries.is_empty());
}

#[test]
fn flatten_single() {
    let thing_hierarchy = hierarchy_deserialize("t_a: {}");
    let flat_entries = hierarchy_flatten(&thing_hierarchy);
    assert_eq!(flat_entries.len(), 1);
    assert_eq!(flat_entries[0].thing_id.as_str(), "t_a");
    assert_eq!(flat_entries[0].depth, 0);
}

#[test]
fn flatten_nested() {
    let thing_hierarchy = hierarchy_deserialize(
        "\
t_a:
  t_b:
    t_c: {}
t_d: {}",
    );
    let flat_entries = hierarchy_flatten(&thing_hierarchy);
    assert_eq!(flat_entries.len(), 4);
    assert_eq!(
        (flat_entries[0].thing_id.as_str(), flat_entries[0].depth),
        ("t_a", 0)
    );
    assert_eq!(
        (flat_entries[1].thing_id.as_str(), flat_entries[1].depth),
        ("t_b", 1)
    );
    assert_eq!(
        (flat_entries[2].thing_id.as_str(), flat_entries[2].depth),
        ("t_c", 2)
    );
    assert_eq!(
        (flat_entries[3].thing_id.as_str(), flat_entries[3].depth),
        ("t_d", 0)
    );
}

#[test]
fn round_trip() {
    let yaml = "\
t_aws:
  t_aws_iam:
    t_aws_iam_policy: {}
  t_aws_ecr: {}
t_github:
  t_github_repo: {}
t_localhost: {}";
    let thing_hierarchy = hierarchy_deserialize(yaml);
    let flat_entries = hierarchy_flatten(&thing_hierarchy);
    let thing_hierarchy_rebuilt = hierarchy_rebuild(&flat_entries);
    assert_eq!(thing_hierarchy, thing_hierarchy_rebuilt);
}

#[test]
fn rebuild_empty() {
    let thing_hierarchy_rebuilt = hierarchy_rebuild(&[]);
    assert_eq!(thing_hierarchy_rebuilt, ThingHierarchy::new());
}

#[test]
fn rebuild_flat_list() {
    let thing_hierarchy = hierarchy_deserialize(
        "\
t_a: {}
t_b: {}
t_c: {}",
    );
    let flat_entries = hierarchy_flatten(&thing_hierarchy);
    assert_eq!(flat_entries.len(), 3);
    for flat_entry in &flat_entries {
        assert_eq!(flat_entry.depth, 0);
    }
    let thing_hierarchy_rebuilt = hierarchy_rebuild(&flat_entries);
    assert_eq!(thing_hierarchy, thing_hierarchy_rebuilt);
}

#[test]
fn rebuild_deeply_nested() {
    let thing_hierarchy = hierarchy_deserialize(
        "\
t_root:
  t_child:
    t_grandchild:
      t_great_grandchild: {}",
    );
    let flat_entries = hierarchy_flatten(&thing_hierarchy);
    assert_eq!(flat_entries.len(), 4);
    assert_eq!(flat_entries[0].depth, 0);
    assert_eq!(flat_entries[1].depth, 1);
    assert_eq!(flat_entries[2].depth, 2);
    assert_eq!(flat_entries[3].depth, 3);
    let thing_hierarchy_rebuilt = hierarchy_rebuild(&flat_entries);
    assert_eq!(thing_hierarchy, thing_hierarchy_rebuilt);
}
