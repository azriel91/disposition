//! Flat entry representation of the thing hierarchy.
//!
//! The [`ThingHierarchy`] is a recursive tree structure. For UI rendering
//! we flatten it into a `Vec<FlatEntry>` where each entry carries the
//! [`ThingId`] and its nesting depth. After mutations on the flat list
//! we rebuild the tree with [`hierarchy_rebuild`].

use disposition::input_model::thing::{ThingHierarchy, ThingId};

/// A single entry in the flattened thing hierarchy.
///
/// # Fields
///
/// * `thing_id`: the `ThingId`, e.g. `"t_aws"`.
/// * `depth`: nesting level, where `0` is a top-level entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlatEntry {
    /// The `ThingId` for this entry, e.g. `"t_aws"`.
    pub thing_id: ThingId<'static>,
    /// Nesting depth (`0` = top-level).
    pub depth: usize,
}

/// Recursively flatten a [`ThingHierarchy`] into a `Vec<FlatEntry>`.
///
/// Entries are produced in depth-first, declaration order so that the visual
/// ordering in the UI matches the YAML source.
///
/// # Example
///
/// Given:
///
/// ```yaml
/// t_aws:
///   t_aws_iam: {}
/// t_github: {}
/// ```
///
/// Produces:
///
/// ```text
/// FlatEntry { thing_id: ThingId("t_aws"),     depth: 0 }
/// FlatEntry { thing_id: ThingId("t_aws_iam"), depth: 1 }
/// FlatEntry { thing_id: ThingId("t_github"),  depth: 0 }
/// ```
pub fn hierarchy_flatten(hierarchy: &ThingHierarchy<'_>) -> Vec<FlatEntry> {
    let mut entries = Vec::new();
    hierarchy_flatten_recursive(hierarchy, 0, &mut entries);
    entries
}

fn hierarchy_flatten_recursive(
    hierarchy: &ThingHierarchy<'_>,
    depth: usize,
    entries: &mut Vec<FlatEntry>,
) {
    hierarchy.iter().for_each(|(thing_id, children)| {
        entries.push(FlatEntry {
            thing_id: ThingId::from(thing_id.as_ref().clone().into_static()),
            depth,
        });
        hierarchy_flatten_recursive(children, depth + 1, entries);
    });
}

/// Rebuild a [`ThingHierarchy`] from a flat entry list.
///
/// The algorithm recursively groups entries by depth. Each entry at
/// `expected_depth` becomes a key in the resulting map, and all immediately
/// deeper entries become its children.
///
/// # Panics
///
/// Does not panic but silently clamps invalid depths (e.g. a depth jump
/// of more than one level from the previous entry is treated as depth
/// `prev_depth + 1`).
pub fn hierarchy_rebuild(entries: &[FlatEntry]) -> ThingHierarchy<'static> {
    if entries.is_empty() {
        return ThingHierarchy::new();
    }

    rebuild_subtree(entries, 0, 0).0
}

/// Recursively build a `ThingHierarchy` from a slice of `FlatEntry`,
/// starting at `start_index` and expecting entries at `expected_depth`.
///
/// Returns `(hierarchy, next_index)` where `next_index` is the index of the
/// first entry that does NOT belong to this subtree (i.e. its depth is less
/// than `expected_depth`).
fn rebuild_subtree(
    entries: &[FlatEntry],
    start_index: usize,
    expected_depth: usize,
) -> (ThingHierarchy<'static>, usize) {
    let mut hierarchy = ThingHierarchy::new();
    let mut i = start_index;

    while i < entries.len() {
        let entry = &entries[i];

        if entry.depth < expected_depth {
            // This entry belongs to a parent level -- stop.
            break;
        }

        let thing_id = entry.thing_id.clone();

        i += 1;

        // Collect children (entries at depth > current).
        let (children, next_i) = rebuild_subtree(entries, i, expected_depth + 1);
        i = next_i;

        hierarchy.insert(thing_id, children);
    }

    (hierarchy, i)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_hierarchy(yaml: &str) -> ThingHierarchy<'static> {
        serde_saphyr::from_str(yaml).expect("valid YAML")
    }

    #[test]
    fn flatten_empty() {
        let h = ThingHierarchy::new();
        let flat = hierarchy_flatten(&h);
        assert!(flat.is_empty());
    }

    #[test]
    fn flatten_single() {
        let h = make_hierarchy("t_a: {}");
        let flat = hierarchy_flatten(&h);
        assert_eq!(flat.len(), 1);
        assert_eq!(flat[0].thing_id.as_str(), "t_a");
        assert_eq!(flat[0].depth, 0);
    }

    #[test]
    fn flatten_nested() {
        let h = make_hierarchy(
            "\
t_a:
  t_b:
    t_c: {}
t_d: {}",
        );
        let flat = hierarchy_flatten(&h);
        assert_eq!(flat.len(), 4);
        assert_eq!((flat[0].thing_id.as_str(), flat[0].depth), ("t_a", 0));
        assert_eq!((flat[1].thing_id.as_str(), flat[1].depth), ("t_b", 1));
        assert_eq!((flat[2].thing_id.as_str(), flat[2].depth), ("t_c", 2));
        assert_eq!((flat[3].thing_id.as_str(), flat[3].depth), ("t_d", 0));
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
        let h = make_hierarchy(yaml);
        let flat = hierarchy_flatten(&h);
        let rebuilt = hierarchy_rebuild(&flat);
        assert_eq!(h, rebuilt);
    }

    #[test]
    fn rebuild_empty() {
        let rebuilt = hierarchy_rebuild(&[]);
        assert_eq!(rebuilt, ThingHierarchy::new());
    }
}
