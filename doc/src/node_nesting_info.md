# Node Nesting Info

`NodeNestingInfo` captures where a single node sits in the node hierarchy.
`NodeNestingInfos` (`Map<NodeId, NodeNestingInfo>`) holds that information for
every node in the diagram.

## Types

### `NodeNestingInfo` (`crate/ir_model/src/node/node_nesting_info.rs`)

Two parallel vectors, always the same length:

| Field | Type | Meaning |
|---|---|---|
| `nesting_path` | `Vec<usize>` | Sibling index at each level from root to this node. |
| `ancestor_chain` | `Vec<NodeId>` | The `NodeId` at each level from root to this node (inclusive). |

The two vectors are aligned: `ancestor_chain[i]` is the `NodeId` of the
ancestor at depth `i`, and `nesting_path[i]` is its position among its
siblings at that depth.

For a top-level node the vectors have length 1.  
For a node nested `n` levels deep the vectors have length `n + 1`.

### Example

Given the hierarchy (iteration order matches insertion order):

```text
proc_app_dev          (top-level, index 0)
  step_repo_clone     (first child of proc_app_dev, index 0)
  step_project_build  (second child of proc_app_dev, index 1)
t_aws                 (top-level, index 1)
  t_aws_iam           (first child of t_aws, index 0)
```

The resulting `NodeNestingInfos` entries are:

| Node | `nesting_path` | `ancestor_chain` |
|---|---|---|
| `proc_app_dev` | `[0]` | `[proc_app_dev]` |
| `step_repo_clone` | `[0, 0]` | `[proc_app_dev, step_repo_clone]` |
| `step_project_build` | `[0, 1]` | `[proc_app_dev, step_project_build]` |
| `t_aws` | `[1]` | `[t_aws]` |
| `t_aws_iam` | `[1, 0]` | `[t_aws, t_aws_iam]` |

Notice that `nesting_path[0]` reflects the node's position among all top-level
siblings, not just siblings of the same entity type.

## How it is computed

Source: `NodeNestingInfosBuilder` in
`crate/input_ir_rt/src/input_to_ir_diagram_mapper/node_nesting_infos_builder.rs`.

Called from step 13 of `InputToIrDiagramMapper::map`, after the `NodeHierarchy`
has been assembled (see `diagram_generation.md`):

```text
node_nesting_infos = NodeNestingInfosBuilder::build(&node_hierarchy);
```

### Algorithm

`build` starts a depth-first walk of the `NodeHierarchy` with two accumulators
initially empty:

- `parent_path: &[usize]` -- sibling-index path to the current node's parent.
- `parent_ancestor_chain: &[NodeId]` -- ancestor chain up to and including the
  current node's parent.

`build_recursive` iterates over the children of the current hierarchy level in
order, giving each child its `index` (0-based position):

```text
for (index, (node_id, child_hierarchy)) in hierarchy.iter().enumerate():
    nesting_path    = parent_path    + [index]
    ancestor_chain  = parent_ancestor_chain + [node_id]

    result.insert(node_id, NodeNestingInfo { nesting_path, ancestor_chain })

    if child_hierarchy is not empty:
        build_recursive(child_hierarchy, nesting_path, ancestor_chain, result)
```

The recursion is pre-order (parent inserted before children) and the `result`
map is flat -- every node at every depth is a direct entry in the map.

### Input: `NodeHierarchy`

`NodeHierarchy` is a recursive `Map<NodeId, NodeHierarchy>` that covers all
node types in a single tree:

1. Tags (top of the map, required for CSS peer selectors to target
   processes/things/edges).
2. Processes (each process contains its step nodes as direct children).
3. Things (same nesting as the user's input `thing_hierarchy`).

The insertion order of the map determines the sibling indices recorded in
`nesting_path`.

## How the fields are used elsewhere

### `ancestor_chain`

- **Parent lookup** (`NodeRanksNested::node_rank_for`): the parent of a node is
  `ancestor_chain[len - 2]`. Top-level nodes (chain length 1) have no parent
  entry and belong to the root level.
- **LCA computation** (`NodeRanksCalculator`, `EdgeSpacerBuilder`): the LCA
  depth of two nodes is the length of the common prefix of their
  `ancestor_chain`s. `ancestor_chain[lca_depth]` gives the divergent ancestor
  on each side.
- **Containment check** (`EdgeSpacerBuildDecider`): whether a node is inside a
  given container is tested with `ancestor_chain.contains(container_node_id)`.
- **Target child lookup** (`EdgeSpacerBuildDecider`): the direct child of a
  container that is the ancestor of an inside endpoint is
  `ancestor_chain[container_depth + 1]`.

### `nesting_path`

- **Spacer insertion index** (`EdgeSpacerBuilder`): the base insertion index
  for a cross-rank spacer is derived from the sibling indices of the two
  divergent ancestors at the LCA level --
  `(nesting_path[lca_depth]_from + nesting_path[lca_depth]_to) / 2 + 1`.
- **LCA sibling distance** (`EdgeSpacerBuildDecider`): the distance between two
  divergent ancestors is
  `abs_diff(nesting_path[lca_depth]_from, nesting_path[lca_depth]_to)`. A
  distance of 1 means adjacent siblings (no node sits between them); 2 or more
  means at least one intermediate sibling exists.
