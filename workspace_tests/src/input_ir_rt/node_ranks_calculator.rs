use disposition::{
    ir_model::{
        edge::{Edge, EdgeGroup, EdgeGroups},
        entity::EntityType,
        node::{NodeId, NodeNestingInfo, NodeNestingInfos, NodeRank, NodeRanks, NodeRanksNested},
    },
    model_common::{edge::EdgeGroupId, entity::EntityTypes, Id, Set},
};
use disposition_input_ir_rt::NodeRanksCalculator;
use pretty_assertions::assert_eq;

// === Helpers === //

/// Constructs a [`NodeId`] from a `&'static str`.
fn node_id(s: &'static str) -> NodeId<'static> {
    NodeId::from(Id::new(s).expect("test node ID must be valid"))
}

/// Constructs an [`EdgeGroupId`] from a `&'static str`.
fn edge_group_id(s: &'static str) -> EdgeGroupId<'static> {
    EdgeGroupId::from(Id::new(s).expect("test edge group ID must be valid"))
}

/// Constructs a [`NodeNestingInfo`] from an ordered list of ancestor IDs.
///
/// The last element is the node itself; earlier elements are its ancestors
/// from root downward. The `nesting_path` is set to ascending indices, which
/// is sufficient for the rank calculator (it only uses `ancestor_chain`).
fn nesting_info(ancestors: &[&'static str]) -> NodeNestingInfo<'static> {
    NodeNestingInfo {
        nesting_path: (0..ancestors.len()).collect(),
        ancestor_chain: ancestors.iter().copied().map(node_id).collect(),
    }
}

/// Builds [`NodeNestingInfos`] from a list of `(node_id, [ancestor, ...,
/// self])` pairs.
fn nesting_infos(entries: &[(&'static str, &[&'static str])]) -> NodeNestingInfos<'static> {
    entries
        .iter()
        .map(|(id, ancestors)| (node_id(id), nesting_info(ancestors)))
        .collect()
}

/// Builds dependency [`EdgeGroups`] and matching [`EntityTypes`] from a list
/// of `(group_id, from, to)` triples.
///
/// Each triple produces one `DependencyEdgeSequenceDefault` edge group
/// recognised by [`NodeRanksCalculator`].
fn dep_edge_groups(
    triples: &[(&'static str, &'static str, &'static str)],
) -> (EdgeGroups<'static>, EntityTypes<'static>) {
    let mut edge_groups = EdgeGroups::new();
    let mut entity_types = EntityTypes::new();
    for (group_id_str, from_str, to_str) in triples {
        let gid = edge_group_id(group_id_str);
        let id: Id<'static> = gid.clone().into_inner();
        edge_groups.insert(
            gid,
            EdgeGroup::from(vec![Edge::new(node_id(from_str), node_id(to_str))]),
        );
        entity_types.insert(
            id,
            Set::from_iter([EntityType::DependencyEdgeSequenceDefault]),
        );
    }
    (edge_groups, entity_types)
}

/// Asserts that the root-level ranks in a [`NodeRanksNested`] exactly match
/// `expected`.
fn assert_root_ranks(result: &NodeRanksNested, expected: &[(&'static str, u32)]) {
    let expected_ranks: NodeRanks = expected
        .iter()
        .map(|(id, rank)| (node_id(id), NodeRank::new(*rank)))
        .collect();
    assert_eq!(expected_ranks, result.root);
}

/// Asserts that the ranks for the named container in a [`NodeRanksNested`]
/// exactly match `expected`.
fn assert_container_ranks(
    result: &NodeRanksNested,
    container: &'static str,
    expected: &[(&'static str, u32)],
) {
    let expected_ranks: NodeRanks = expected
        .iter()
        .map(|(id, rank)| (node_id(id), NodeRank::new(*rank)))
        .collect();
    let actual = result
        .containers
        .get(&node_id(container))
        .unwrap_or_else(|| panic!("container '{container}' not found in node_ranks_nested"));
    assert_eq!(&expected_ranks, actual);
}

// === Tests === //

/// Case 1: root-level `NodeRank`s increase when edges are between root-level
/// siblings.
///
/// Hierarchy: `a`, `b` (leaf nodes, no children)
/// Edge: `a -> b`
/// Expected root ranks: `a: 0`, `b: 1`
/// No containers (neither node has children).
#[test]
fn test_node_ranks_root_level_sibling_edges() {
    let node_nesting_infos = nesting_infos(&[("a", &["a"]), ("b", &["b"])]);
    let (edge_groups, entity_types) = dep_edge_groups(&[("edge_a_b", "a", "b")]);

    let result = NodeRanksCalculator::calculate(&edge_groups, &entity_types, &node_nesting_infos);

    assert_root_ranks(&result, &[("a", 0), ("b", 1)]);
    assert!(
        result.containers.is_empty(),
        "expected no containers because neither node has children"
    );
}

/// Case 2: root-level `NodeRank`s increase when an edge connects children of
/// different root-level siblings.
///
/// Hierarchy: `a: {a_child}`, `b: {b_child}`
/// Edge: `a_child -> b_child`
/// LCA = root, divergent ancestors = `a`, `b`
/// Expected root ranks: `a: 0`, `b: 1` (edge is lifted to root level)
/// Expected container ranks: `a: {a_child: 0}`, `b: {b_child: 0}`
///   (within each container no sibling-level edge exists; both children rank 0)
#[test]
fn test_node_ranks_root_level_lifted_from_child_edges() {
    let node_nesting_infos = nesting_infos(&[
        ("a", &["a"]),
        ("a_child", &["a", "a_child"]),
        ("b", &["b"]),
        ("b_child", &["b", "b_child"]),
    ]);
    let (edge_groups, entity_types) =
        dep_edge_groups(&[("edge_a_child_b_child", "a_child", "b_child")]);

    let result = NodeRanksCalculator::calculate(&edge_groups, &entity_types, &node_nesting_infos);

    assert_root_ranks(&result, &[("a", 0), ("b", 1)]);
    assert_container_ranks(&result, "a", &[("a_child", 0)]);
    assert_container_ranks(&result, "b", &[("b_child", 0)]);
}

/// Case 3: nested-level `NodeRank`s increase when edges connect nested
/// siblings that share the same parent.
///
/// Hierarchy: `p: {p_a, p_b}`
/// Edge: `p_a -> p_b`
/// LCA = `p`, divergent ancestors = `p_a`, `p_b`
/// Expected root ranks: `p: 0`
/// Expected container ranks for `p`: `p_a: 0`, `p_b: 1`
#[test]
fn test_node_ranks_nested_level_same_parent_edge() {
    let node_nesting_infos = nesting_infos(&[
        ("p", &["p"]),
        ("p_a", &["p", "p_a"]),
        ("p_b", &["p", "p_b"]),
    ]);
    let (edge_groups, entity_types) = dep_edge_groups(&[("edge_p_a_p_b", "p_a", "p_b")]);

    let result = NodeRanksCalculator::calculate(&edge_groups, &entity_types, &node_nesting_infos);

    assert_root_ranks(&result, &[("p", 0)]);
    assert_container_ranks(&result, "p", &[("p_a", 0), ("p_b", 1)]);
}

/// Case 4: nested-level `NodeRank`s do NOT increase for nodes when an edge
/// connects nested siblings under different parents.
///
/// Hierarchy: `p1: {p1_a}`, `p2: {p2_a}`
/// Edge: `p1_a -> p2_a`
/// LCA = root, divergent ancestors = `p1`, `p2`
/// Expected root ranks: `p1: 0`, `p2: 1` (root level IS affected)
/// Expected container ranks: `p1: {p1_a: 0}`, `p2: {p2_a: 0}`
///   (no sibling-level edge exists inside either container; both children rank
/// 0)
#[test]
fn test_node_ranks_nested_level_different_parent_edge() {
    let node_nesting_infos = nesting_infos(&[
        ("p1", &["p1"]),
        ("p1_a", &["p1", "p1_a"]),
        ("p2", &["p2"]),
        ("p2_a", &["p2", "p2_a"]),
    ]);
    let (edge_groups, entity_types) = dep_edge_groups(&[("edge_p1_a_p2_a", "p1_a", "p2_a")]);

    let result = NodeRanksCalculator::calculate(&edge_groups, &entity_types, &node_nesting_infos);

    assert_root_ranks(&result, &[("p1", 0), ("p2", 1)]);
    assert_container_ranks(&result, "p1", &[("p1_a", 0)]);
    assert_container_ranks(&result, "p2", &[("p2_a", 0)]);
}

/// Case 5: multi-nested-level `NodeRank`s increase when edges connect
/// siblings sharing the same deeply-nested parent.
///
/// Hierarchy: `outer: {inner: {inner_a, inner_b}}`
/// Edge: `inner_a -> inner_b`
/// LCA = `inner`, divergent ancestors = `inner_a`, `inner_b`
/// Expected root ranks: `outer: 0`
/// Expected container ranks for `outer`: `inner: 0`
/// Expected container ranks for `inner`: `inner_a: 0`, `inner_b: 1`
#[test]
fn test_node_ranks_multi_nested_same_parent_edge() {
    let node_nesting_infos = nesting_infos(&[
        ("outer", &["outer"]),
        ("inner", &["outer", "inner"]),
        ("inner_a", &["outer", "inner", "inner_a"]),
        ("inner_b", &["outer", "inner", "inner_b"]),
    ]);
    let (edge_groups, entity_types) =
        dep_edge_groups(&[("edge_inner_a_inner_b", "inner_a", "inner_b")]);

    let result = NodeRanksCalculator::calculate(&edge_groups, &entity_types, &node_nesting_infos);

    assert_root_ranks(&result, &[("outer", 0)]);
    assert_container_ranks(&result, "outer", &[("inner", 0)]);
    assert_container_ranks(&result, "inner", &[("inner_a", 0), ("inner_b", 1)]);
}

/// Case 6: multi-nested-level `NodeRank`s do NOT increase inside inner
/// containers when an edge connects deeply-nested siblings under different
/// top-level parents.
///
/// Hierarchy: `outer_x: {inner_x: {x_child}}`, `outer_y: {inner_y: {y_child}}`
/// Edge: `x_child -> y_child`
/// LCA = root, divergent ancestors = `outer_x`, `outer_y`
/// Expected root ranks: `outer_x: 0`, `outer_y: 1` (root level IS affected)
/// Expected container ranks for `outer_x`: `inner_x: 0`
/// Expected container ranks for `inner_x`: `x_child: 0`
/// Expected container ranks for `outer_y`: `inner_y: 0`
/// Expected container ranks for `inner_y`: `y_child: 0`
///   (only the root level is affected; all inner containers retain rank 0)
#[test]
fn test_node_ranks_multi_nested_different_top_level_parent_edge() {
    let node_nesting_infos = nesting_infos(&[
        ("outer_x", &["outer_x"]),
        ("inner_x", &["outer_x", "inner_x"]),
        ("x_child", &["outer_x", "inner_x", "x_child"]),
        ("outer_y", &["outer_y"]),
        ("inner_y", &["outer_y", "inner_y"]),
        ("y_child", &["outer_y", "inner_y", "y_child"]),
    ]);
    let (edge_groups, entity_types) =
        dep_edge_groups(&[("edge_x_child_y_child", "x_child", "y_child")]);

    let result = NodeRanksCalculator::calculate(&edge_groups, &entity_types, &node_nesting_infos);

    assert_root_ranks(&result, &[("outer_x", 0), ("outer_y", 1)]);
    assert_container_ranks(&result, "outer_x", &[("inner_x", 0)]);
    assert_container_ranks(&result, "inner_x", &[("x_child", 0)]);
    assert_container_ranks(&result, "outer_y", &[("inner_y", 0)]);
    assert_container_ranks(&result, "inner_y", &[("y_child", 0)]);
}
