# Node Ranks

Node ranks drive the layout order of nodes within a diagram. Each node is
assigned a [`NodeRank`](crate/ir_model/src/node/node_rank.rs) that places it
along the main flex axis of its container. Ranks are stored in a
[`NodeRanksNested`](crate/ir_model/src/node/node_ranks_nested.rs) and computed
by [`NodeRanksCalculator`](crate/input_ir_rt/src/node_ranks_calculator.rs).


## Types


### `NodeRank`

Source: [`crate/ir_model/src/node/node_rank.rs`](crate/ir_model/src/node/node_rank.rs)

A newtype around `u32` representing the layout position of a node within its
level. The value is non-negative and has no fixed upper bound -- it is
determined entirely by the dependency structure of the diagram.

Key properties:

- **Lower rank = positioned earlier.** A node with rank `0` appears at the
  left (or top, depending on the flex direction) of its container; a node with
  rank `2` appears after any node with rank `1`.
- **Determined by dependency edges.** The `to` node of a dependency edge
  receives a strictly higher rank than the `from` node, so that dependencies
  flow from left to right (or top to bottom).
- **Per-level, not global.** Two nodes in different containers can both have
  rank `0` without conflicting, because ranks are computed independently for
  each container level.
- **Cycles share a rank.** If nodes form a dependency cycle, all nodes in the
  cycle are contracted into a single logical unit and receive the same rank
  value.

Example values: `0`, `1`, `2`, `3`.


### `NodeRanksNested`

Source: [`crate/ir_model/src/node/node_ranks_nested.rs`](crate/ir_model/src/node/node_ranks_nested.rs)

A hierarchy-aware collection of rank maps. Instead of one flat
`Map<NodeId, NodeRank>` for the whole diagram, `NodeRanksNested` holds a
separate [`NodeRanks`](crate/ir_model/src/node/node_ranks.rs)
(`Map<NodeId, NodeRank>`) for each level in the node hierarchy.

Fields:

- **`root`** -- ranks for the top-level nodes (direct children of the diagram
  root). Populated from dependency edges whose lowest common ancestor (LCA) is
  the diagram root.
- **`containers`** -- ranks for each container node's direct children, keyed
  by the container `NodeId`. Only containers with at least one direct child are
  included. Populated from dependency edges whose LCA is that container.

Ranks within each entry are computed independently. A node at root level and a
node inside a container never share the same rank computation.

#### `node_rank_for`

`NodeRanksNested::node_rank_for(node_id, node_nesting_infos)` looks up the
rank for any node by:

1. Retrieving the node's `NodeNestingInfo` from `node_nesting_infos`.
2. Reading the `ancestor_chain` -- an ordered list of ancestor `NodeId`s from
   the outermost ancestor down to the node itself.
3. Finding the parent: the second-to-last element of the chain. If the chain
   has length 1 the node is at the root level and the parent is `None`.
4. Calling `ranks_for(parent)` to get the `NodeRanks` for that level, then
   looking up the node's rank within it.


## Why hierarchy-aware ranks exist

In a nested node hierarchy, dependency edges may connect nodes at different
nesting depths. A single flat rank assignment would conflate nodes from
unrelated containers, forcing their ranks to be globally consistent even when
they have nothing to do with each other. By computing `NodeRanks` per level,
each container's children are ranked relative to their siblings only, which
correctly drives their layout position within their own container without
disturbing other levels.

Dependency edges that cross container boundaries are not ignored -- they are
attributed to the lowest common ancestor (LCA) level of the two endpoints.
This is described in detail in the [LCA edge lifting](#step-3-lift-edges-to-lca-level) step.


## Concepts

Before reading the algorithm it helps to have precise definitions for the
following terms.

- **Level** -- the set of sibling nodes that share the same direct parent
  container. The root level contains the top-level nodes in the diagram. Each
  container node defines its own level for its direct children.
- **Container** -- a node that has at least one direct child. Its direct
  children form their own level with their own `NodeRanks`.
- **`ancestor_chain`** -- for a given node, the ordered sequence of `NodeId`s
  starting from the outermost ancestor (a top-level node) and ending with the
  node itself. A top-level node has a chain of length 1. A direct child of a
  top-level node has a chain of length 2, and so on.
- **LCA (Lowest Common Ancestor)** -- for two nodes `A` and `B`, the deepest
  node in the hierarchy that is an ancestor of (or equal to) both. The LCA
  depth is the length of the common prefix shared by the two `ancestor_chain`s.
- **Divergent ancestor** -- given two nodes and their LCA, the divergent
  ancestor of a node is the direct child of the LCA that lies on the path to
  that node. It is the element at index `lca_depth` in the node's
  `ancestor_chain`.
- **LCA-level edge** -- for a dependency edge `(from, to)`, the corresponding
  edge `(divergent_from, divergent_to)` at the LCA level. When `from` and `to`
  are already siblings their LCA-level edge is just `(from, to)` unchanged.
  When they live in different containers the edge is "lifted" so that only the
  sibling-level relationship is captured.
- **NestingAwareNodeRank** -- a rank ordering between two siblings N1 and N2 at
  a given level that arises because there exists a dependency edge from *any*
  node in N1's subtree to *any* node in N2's subtree (N1 and N2 themselves
  included). The actual edge endpoints may be arbitrarily deeply nested; only
  the sibling-level relationship `N1 < N2` is recorded at that level.
  NestingAwareNodeRank relationships are produced by the LCA edge lifting step
  (Step 3 of `NodeRanksCalculator`) and consumed by layout ordering, edge
  spacer placement, and edge path routing -- see
  [Where NestingAwareNodeRank applies](#where-nestingawarenoderank-applies).
- **SCC (Strongly Connected Component)** -- a maximal set of nodes that are
  mutually reachable along directed edges. A cycle of nodes forms a single SCC.
  Nodes with no cyclic connections form singleton SCCs.


## Where NestingAwareNodeRank applies

The LCA edge lifting step (Step 3) converts every raw dependency edge into a
`NestingAwareNodeRank` relationship at the appropriate level. The resulting
`NodeRanksNested` values therefore encode NestingAwareNodeRank throughout. The
following three parts of the pipeline rely on it.

### 1. Taffy layout order

Source: `IrToTaffyBuilder` in `crate/input_ir_rt/src/ir_to_taffy_builder.rs`

Each node is placed into a *rank container* -- a taffy flex node that groups
all siblings with the same `NodeRank`. The rank containers are sorted by rank
and stacked along the parent's flex axis.

Because ranks reflect NestingAwareNodeRank, a container node N1 is laid out
before N2 whenever *any* descendant of N1 has a dependency edge to *any*
descendant of N2, even if N1 and N2 have no direct edge between them. The
visual ordering of the diagram thus mirrors the overall dependency flow of the
entire subtree, not just direct edges.

### 2. Same-level cross-rank edge spacers

Source: `EdgeSpacerBuilder::edge_spacers_build` in
`crate/input_ir_rt/src/ir_to_taffy_builder/edge_spacer_builder.rs`

When determining the visual rank span of an edge (to know how many intermediate
rank rows it must jump over), the code does not use the ranks of the actual edge
endpoints. It uses the ranks of the *divergent ancestors* -- the two siblings at
the LCA level. Those ancestor ranks are NestingAwareNodeRank values.

For example, an edge from `a_child` (rank 0 inside `a`) to `c_child` (rank 0
inside `c`) has divergent ancestors `a` (rank 0) and `c` (rank 2) at the root
level. The visual rank span is 0..2, so spacer nodes are inserted at rank 1.
If NestingAwareNodeRank were not used and only direct ranks were consulted, the
span would appear to be 0..0 and no spacers would be inserted, causing the edge
path to draw over the intermediate rank-1 nodes.

### 3. Cross-container spacers

Source: `EdgeSpacerBuilder::build_cross_container_spacers_for_edge` in
`crate/input_ir_rt/src/ir_to_taffy_builder/edge_spacer_builder.rs`

When one edge endpoint is inside a container and the other is outside, spacers
are inserted alongside the container's siblings that lie at ranks *strictly
before* the target child. The `target_rank` (the rank of the direct child of
the container that contains the inside endpoint) is itself a NestingAwareNodeRank
value -- it may have been elevated by an edge from a deeply nested descendant
to a node outside the container. Using this rank correctly identifies which
sibling rows the incoming edge path must route around before it reaches the
target child.


## `NodeRanksCalculator`

Source: [`crate/input_ir_rt/src/node_ranks_calculator.rs`](crate/input_ir_rt/src/node_ranks_calculator.rs)

The entry point is:

```rust
NodeRanksCalculator::calculate(edge_groups, entity_types, node_nesting_infos, layout_edges)
```

It returns a `NodeRanksNested<'id>` computed in four steps.


### Step 1: Build container-to-children map

Function: `container_to_children_build`

Iterates over every entry in `node_nesting_infos` and groups each node under
its direct parent:

- Read the node's `ancestor_chain`.
- The parent is `chain[chain.len() - 2]` -- the second-to-last element.
- If the chain has length 1 the node is a top-level node; its parent key is
  `None` (the root level).
- Otherwise the parent key is `Some(parent_id)`.

The result is `Map<Option<NodeId>, Vec<NodeId>>`: a map from each container
(or `None` for root) to the list of its direct children.

This map drives the outer loop in `calculate` -- one `NodeRanks` is computed
for each entry.


### Step 2: Collect dependency edges

Function: `dependency_edges_collect`

Scans every edge group in `edge_groups` and keeps only the groups whose entity
type is one of the three dependency variants:

- `DependencyEdgeCyclicDefault`
- `DependencyEdgeSequenceDefault`
- `DependencyEdgeSymmetricDefault`

The `layout_edges` parameter (built from `thing_layout_edges`) is appended
directly to this list -- these edges have no backing edge group or entity
type, since they're never rendered, but contribute to rank identically to
dependency edges.

Self-loops (edges where `from == to`) are skipped because they carry no rank
information.

The result is a flat `Vec<(NodeId, NodeId)>` of `(from, to)` pairs covering
all dependency and layout edges in the diagram.


### Step 3: Lift edges to LCA level

Functions: `lca_level_edges_build`, `lca_level_edge_compute`

Dependency edges may connect nodes at different depths in the hierarchy. This
step maps every raw edge to the equivalent edge between siblings at their LCA
level, producing [NestingAwareNodeRank](#concepts) relationships.

For each edge `(from, to)`:

1. Look up `ancestor_chain_from` and `ancestor_chain_to` in
   `node_nesting_infos`. If either node is absent, skip the edge.
2. Walk the two chains together, counting how many leading elements are equal.
   This count is `lca_depth`.
3. **Skip if one node is an ancestor of the other.** If `lca_depth` is equal
   to or greater than the length of either chain, then one endpoint lies on
   the path to the other endpoint. Such an edge does not create a sibling-level
   rank relationship and is discarded.
4. Derive the divergent ancestors:
   - `divergent_from = chain_from[lca_depth]`
   - `divergent_to   = chain_to[lca_depth]`
5. **Skip self-loops at LCA level.** If `divergent_from == divergent_to` the
   two endpoints have the same divergent ancestor, which would create a
   self-loop at the LCA level. Such edges are discarded.
6. Derive the LCA container:
   - If `lca_depth > 0`: `lca_container = Some(chain_from[lca_depth - 1])`
   - If `lca_depth == 0`: `lca_container = None` (the diagram root)

The LCA-level edge `(divergent_from, divergent_to)` is then inserted into the
output map under the key `lca_container`.

The result is `Map<Option<NodeId>, Vec<(NodeId, NodeId)>>`: a map from each
container (or `None` for root) to the list of LCA-level edges that belong to
that container's level.


### Step 4: Compute ranks per level

Function: `ranks_compute`

Called once for each entry in the container-to-children map. Takes the list
of nodes at that level (`all_node_ids`) and the LCA-level edges for that
container, and returns a `NodeRanks` for those nodes.

#### Special cases

- **No nodes** -- returns an empty `NodeRanks`.
- **No dependency edges** -- all nodes at this level have no ordering
  constraint relative to each other; every node receives rank `0`.

#### General case

When there is at least one dependency edge, the algorithm proceeds in five
sub-steps.

**Sub-step A: Assign numeric indices.**
Each `NodeId` in `all_node_ids` is mapped to a consecutive integer index
`0..node_count`. All graph operations use these indices for efficiency.

**Sub-step B: Build an adjacency list.**
For each dependency edge `(from_id, to_id)`, the corresponding numeric indices
are looked up and an entry is added to `adjacency[from_idx]`. Edges referencing
nodes not in `all_node_ids` are ignored (they were filtered out during LCA
lifting, but this is a safety check).

**Sub-step C: Run Tarjan's SCC algorithm.**
Tarjan's algorithm is run on the adjacency list using an iterative DFS (to
avoid stack overflows on large graphs). It produces a `scc_ids` array where
`scc_ids[node_index]` is the integer SCC id for that node. Nodes in the same
cycle share the same SCC id.

**Sub-step D: Build a DAG of SCCs and compute SCC ranks.**
The SCC ids define a condensation DAG. For each inter-SCC edge in the original
adjacency list, an edge is added to `scc_adjacency`. Duplicate edges are
removed and in-degrees are recomputed. Kahn's algorithm (BFS topological sort)
is then run on the SCC DAG with a longest-path extension:

- All SCCs with in-degree 0 are seeded into the processing queue with rank 0.
- When an SCC is dequeued, each of its successors in the DAG is offered a
  candidate rank of `current_rank + 1`. If the candidate is greater than the
  successor's current rank, the successor's rank is updated. Once a successor's
  in-degree reaches 0 it is enqueued.

The result is a `Vec<u32>` of length `scc_count` where each entry is the
longest-path rank for that SCC.

**Sub-step E: Map SCC ranks back to node ranks.**
For each node, its rank is `scc_ranks[scc_ids[node_index]]`. This is wrapped in
a `NodeRank` and collected into the output `NodeRanks` map.


## Full worked example

Consider the following hierarchy and dependency edges:

```yaml
node_hierarchy:
  a: { a_child: {} }
  b: { b_child_0: {}, b_child_1: {} }
  c: { c_child: {} }

edges:
  edge_a_b:                { from: a,         to: b         }
  edge_b0_b1:              { from: b_child_0, to: b_child_1 }
  edge_b0_cc:              { from: b_child_0, to: c_child   }
```

`ancestor_chain` for each node:

| Node | `ancestor_chain` |
|---|---|
| `a` | `[a]` |
| `a_child` | `[a, a_child]` |
| `b` | `[b]` |
| `b_child_0` | `[b, b_child_0]` |
| `b_child_1` | `[b, b_child_1]` |
| `c` | `[c]` |
| `c_child` | `[c, c_child]` |

**Step 1** produces this container-to-children map:

| Container (`None` = root) | Direct children |
|---|---|
| `None` | `a`, `b`, `c` |
| `a` | `a_child` |
| `b` | `b_child_0`, `b_child_1` |
| `c` | `c_child` |

**Step 2** collects three dependency edges (assuming all three are dependency
type): `(a, b)`, `(b_child_0, b_child_1)`, `(b_child_0, c_child)`.

**Step 3** lifts each edge:

- `(a, b)`: chains `[a]` and `[b]` -- common prefix length = 0 so
  `lca_depth = 0`. `divergent_from = a`, `divergent_to = b`,
  `lca_container = None`. LCA-level edge `(a, b)` at root.
- `(b_child_0, b_child_1)`: chains `[b, b_child_0]` and `[b, b_child_1]`
  -- common prefix `[b]`, `lca_depth = 1`. `divergent_from = b_child_0`,
  `divergent_to = b_child_1`, `lca_container = Some(b)`.
  LCA-level edge `(b_child_0, b_child_1)` at `b`'s level.
- `(b_child_0, c_child)`: chains `[b, b_child_0]` and `[c, c_child]`
  -- common prefix is empty, `lca_depth = 0`. `divergent_from = b`,
  `divergent_to = c`, `lca_container = None`.
  LCA-level edge `(b, c)` at root. Note that this edge originated from deep
  inside `b` and `c`, but is attributed entirely to the root level.

LCA-level edge map after step 3:

| Container | Edges |
|---|---|
| `None` (root) | `(a, b)`, `(b, c)` |
| `b` | `(b_child_0, b_child_1)` |

**Step 4** computes ranks per level:

*Root level* -- nodes `a`, `b`, `c`; edges `a->b`, `b->c`.
Each node is its own SCC (no cycles). SCC DAG is `a->b->c`. Kahn + longest
path gives `a: 0`, `b: 1`, `c: 2`.

*`a`'s level* -- node `a_child`; no edges.
No dependency edges -- `a_child: 0`.

*`b`'s level* -- nodes `b_child_0`, `b_child_1`; edge `b_child_0->b_child_1`.
No cycles. Kahn gives `b_child_0: 0`, `b_child_1: 1`.

*`c`'s level* -- node `c_child`; no edges.
The edge `(b_child_0, c_child)` was lifted to root level and does not appear
here. `c_child: 0`.

Final `NodeRanksNested`:

```yaml
node_ranks_nested:
  root:
    a: 0
    b: 1
    c: 2   # rank lifted from edge_b0_cc (b -> c at root level)
  containers:
    a:
      a_child: 0
    b:
      b_child_0: 0
      b_child_1: 1
    c:
      c_child: 0  # edge_b0_cc was attributed to root, not c's level
```


## Cycles

If a set of nodes at the same level form a dependency cycle -- for example
`x -> y -> z -> x` -- Tarjan's algorithm groups them into a single SCC. All
three nodes receive the same rank value. The SCC is treated as a single unit in
the DAG, and its rank is determined by any predecessors outside the cycle.

For example, given edges `a -> x`, `x -> y`, `y -> z`, `z -> x` at the same
level:

- `{x, y, z}` form one SCC with SCC rank 1 (because `a` has SCC rank 0 and
  feeds into the cycle).
- `a` is a singleton SCC with rank 0.
- Result: `a: 0`, `x: 1`, `y: 1`, `z: 1`.


## Edge types considered

Only **dependency** edge groups contribute to rank computation. Interaction
edges are ignored. The three recognised dependency entity types are:

- `DependencyEdgeCyclicDefault` -- cyclic dependency (explicit cycle notation).
- `DependencyEdgeSequenceDefault` -- sequence dependency (A must come before B).
- `DependencyEdgeSymmetricDefault` -- symmetric dependency (mutual constraint).

Edge groups whose entity type does not match any of these three are silently
skipped in step 2.

**Layout edges** (from `thing_layout_edges`) also contribute to rank, exactly
like dependency edges, but are passed into `calculate` directly as a
`&[Edge]` rather than living in `edge_groups`. This is what guarantees they
never render: `EdgeFaceAssigner`, `SvgEdgeInfosBuilder`, and
`TailwindClassesBuilder` all operate on `edge_groups` alone, so an edge that
never enters that map can never produce an SVG `<path>`, no matter its
contribution to rank.
