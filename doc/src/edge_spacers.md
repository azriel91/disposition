# Edge Spacers

Edge spacer nodes are small invisible taffy leaf nodes inserted into rank containers to provide
coordinate waypoints for edge path routing. When an edge connects two diagram nodes that are
separated by one or more intermediate nodes in the layout, the edge path must route around those
intermediaries. Without spacer nodes the layout engine has no awareness of where the path should
pass, and the path calculation code cannot locate safe routing waypoints.

Spacer nodes participate in the flex layout so that after `taffy_tree.compute_layout_with_measure`
runs, each spacer has a computed absolute position. The edge path builder then uses those positions
as waypoints to route the edge around intermediate nodes.

Edges drawn with a direct curvature (`EdgeCurvature::is_direct()`, i.e. the `DirectStraight` /
`DirectCurved` variants) bypass spacers entirely. Their spacer taffy nodes are still inserted so the
tree structure is unchanged, but `EdgeSpacerBuilder::edge_spacer_length` returns `0.0` for them
instead of `EDGE_SPACER_LENGTH`, so they reserve no layout space. The effective curvature is selected
per edge from `RenderOptions::dependencies_edge_curvature` /
`RenderOptions::interactions_edge_curvature` based on whether the edge is a dependency or interaction
edge.


## Background Concepts

### NodeNestingInfo

Every diagram node has a [`NodeNestingInfo`](crate/ir_model/src/node/node_nesting_info.rs) stored
in `NodeNestingInfos`. It records two parallel sequences:

- `ancestor_chain: Vec<NodeId>` -- the sequence of node IDs from the root of the hierarchy down to
  the node itself (inclusive). For a node `c01` nested inside `c0`, the chain is
  `["c0", "c01"]`. For a top-level node `a`, the chain is `["a"]`.
- `nesting_path: Vec<usize>` -- the sequence of sibling indices at each level, from root to the
  node. Entry `nesting_path[i]` is the position of the node's `ancestor_chain[i]` among its
  siblings. For example `[2, 0]` means "third top-level node, first child".

The two sequences are always the same length and aligned: `nesting_path[i]` is the sibling index
of the node at `ancestor_chain[i]`.

### LCA Depth

The Lowest Common Ancestor (LCA) depth of two nodes is the length of the longest common prefix of
their `ancestor_chain`s, computed by
[`LcaDepthCalculator`](crate/input_ir_rt/src/ir_to_taffy_builder/edge_spacer_builder/lca_depth_calculator.rs).

    ancestor_chain of from_node: ["outer", "a",  "a01"]
    ancestor_chain of to_node:   ["outer", "b",  "b01"]
    longest common prefix:       ["outer"]
    LCA depth: 1

A depth of `0` means the two chains diverge at the first element -- the nodes share no ancestor
within the hierarchy.

    ancestor_chain of from_node: ["a", "a01"]
    ancestor_chain of to_node:   ["c", "c01"]
    longest common prefix:       []
    LCA depth: 0

### Divergent Ancestors

Given an LCA depth `d`, the divergent ancestors of two nodes are the elements at index `d` in each
`ancestor_chain`. They are the first nodes where the two chains differ -- i.e. the direct children
of the LCA that are ancestors of (or equal to) each endpoint.

    from_node ancestor_chain: ["outer", "a", "a01"]  -->  divergent_from = "a"   (index 1)
    to_node   ancestor_chain: ["outer", "c", "c01"]  -->  divergent_to   = "c"   (index 1)

The ranks of the divergent ancestors within their shared parent container determine the visual rank
span the edge must cross.

### Rank Span

Ranks are stored in `NodeRanksNested`, which holds a per-container `NodeRanks` map. For each pair
of divergent ancestors, the rank span is `[rank_low, rank_high]` where `rank_low` and `rank_high`
are the min and max of their two ranks. Intermediate ranks are the integers strictly between
`rank_low` and `rank_high`. If `rank_high - rank_low <= 1` there are no intermediate ranks and no
spacers are needed.

### Spacer Node Style

All spacer nodes are taffy leaf nodes with the following style:

    min_size: 5px x 5px
    align_self: Stretch

The 5 x 5 px minimum size ensures the spacer always occupies a non-zero area so the path builder
can compute a meaningful entry and exit point. `align_self: Stretch` makes the spacer expand to
fill the cross axis of its rank container, which keeps it vertically or horizontally centered with
the other nodes in its row.

Each spacer carries a `TaffyNodeCtx::EdgeSpacer(EdgeSpacerCtx { edge_id, rank })` context value
so the path builder can identify which edge the spacer belongs to and at which rank it sits.


## Five Kinds of Spacer

    EdgeSpacerTaffyNodes {
        # Same-level cross-rank spacers, keyed by intermediate rank.
        rank_to_spacer_taffy_node_id:                          Map<NodeRank, taffy::NodeId>,

        # Cross-container spacers, ordered by layout position.
        cross_container_spacer_taffy_node_ids:                 Vec<taffy::NodeId>,

        # BetweenRanks edge description container spacers, ordered by layout position.
        edge_desc_container_spacer_taffy_node_ids:             Vec<taffy::NodeId>,

        # SameRank edge description container crossing spacers, ordered by layout position.
        same_rank_edge_desc_container_spacer_taffy_node_ids:   Vec<taffy::NodeId>,

        # Text-content (node-label) spacers, ordered by layout position.
        text_content_spacer_taffy_node_ids:                    Vec<taffy::NodeId>,
    }

### 1. Same-Level Cross-Rank Spacers

Built by `EdgeSpacerBuilder::build`. Used when the two endpoints of an edge diverge at the same
hierarchy level and span more than one rank. A spacer is inserted at each intermediate rank in the
same rank-container group that the divergent ancestors belong to.

### 2. Cross-Container Spacers

Built by `EdgeSpacerBuilder::build_cross_container_spacers`. Used when one endpoint is inside a
container and the other is outside, and the edge path must pass alongside the container's sibling
children to reach the deeply-nested endpoint. Spacers are placed at the ranks of those intermediate
sibling children within the container.

### 3. Edge Description Container Spacers

Built by `EdgeSpacerBuilder::build_edge_desc_container_spacers`. For a **cross-rank**
(`BetweenRanks`) container, used when an edge's rank span `[rank_low, rank_high)` crosses the
position `rank_P` of an `edge_description_container` that belongs to a **different** described edge
(condition: `rank_low <= rank_P < rank_high`). For a **same-rank** (cycle edge) container, the
sibling-index-axis analog applies instead (`EdgeSpacerBuilder::build_edge_desc_container_spacers_for_edge_same_rank`):
another edge sharing the same rank whose own two divergent siblings' indices span across the
container's `sibling_index_middle` -- e.g. the reverse direction of the same cyclic pair -- is
crossing it too, even without a description of its own. In both cases a single spacer leaf is
appended as a direct child of that container, providing a coordinate waypoint so the crossing edge's
path builder routes through the container's visual space rather than jumping over it. The described
edge whose container it is does **not** receive a spacer inside its own container -- instead its path
bends directly to the description box's own resolved rect, via
`SpacerCoordinatesResolver::description_contact_resolve` (see
[edge_descriptions.md](edge_descriptions.md)). That contact is applied to the owning edge's path
**unconditionally**, regardless of `EdgeCurvature::is_direct()` -- unlike every other spacer kind
here, which direct-curvature edges bypass entirely.

A same-rank crossing spacer is tracked in a **separate** list,
`same_rank_edge_desc_container_spacer_taffy_node_ids`, and its `EdgeSpacerCtx` carries this edge's
own `sibling_index_from.cmp(&sibling_index_to)` ordering (`same_rank_sibling_index_from_cmp_to`).
`SpacerCoordinatesResolver::resolve` uses this to resolve the spacer's entry/exit via
`EdgeSpacerCoordinatesCalculator::calculate_description_thread_same_rank` -- the same
direction-aware, rotated-axis calculation the owning edge's own contact already uses -- instead of
the direction-oblivious generic `calculate` used for every other spacer kind. This matters because a
same-rank container's children stack *perpendicular* to the edge's own travel direction: resolving
with the diagram's canonical direction regardless of which end the crossing edge actually approaches
first can select the far end as the nominal entry, forcing the connector between the `from`-node's
protrusion leg and that entry point to cut straight through the container's interior to reach it,
instead of jogging clear of the container first. (Before this fix, `edge_dep_server_proc_1_proc_2__1`
in `021_interaction_halo_with_desc_cyclic.yaml` -- whose divergent ancestors sit in the reverse order
along the shared rank relative to the described `edge_dep_server_proc_1_proc_2__0` -- visibly cut
through the top of `proc1_proc2_desc`'s box.)

### 4. Text-Content (Node-Label) Spacers

Built by `EdgeSpacerBuilder::build_text_content_spacers`. Used when a container node that renders a
**non-empty top text band** -- a title and/or a `thing_desc` -- is entered by a cross-container edge
to reach a node nested strictly inside it. Node names are rendered as markdown in the same band as
descriptions, so a wide title is just as crossable as a description; the gate keys off the rendered
text (`TaffyBuildCtx::node_md_text`), not the presence of a description, and only genuinely text-less
containers are skipped. The text block renders at the top of the node body, above its rank
containers; without a waypoint there, the entering edge cuts straight across the label. One spacer
leaf is appended **to the right of the text content** -- the text node and its spacers are wrapped in
a `FlexDirection::Row`, so a single spacer per edge (not one per text line) marks a column just past
the label. The edge then bows around the label and descends on its outer side before returning to its
rank column.

Scope and behaviour differ from the other kinds in three ways:

- **To-side only, cross-rank entries only.** A spacer is created only for the container in which the
  **`to`** node is nested (`edge.to`'s `ancestor_chain` contains the container; the `from` node for
  `BottomToTop`). The from side (where the edge merely exits its source) gets none, the diagram root
  gets none (it contains both endpoints, so the decider skips it), and the container that *is* the
  endpoint node gets none (the path stops at its face). Only **vertical** rank directions
  (`TopToBottom` / `BottomToTop`) build them -- horizontal flows enter past a side-strip label, not
  through it. A spacer is also skipped when the edge's divergent ancestors share a rank (e.g. a
  same-rank cyclic edge between adjacent siblings): such an edge enters the container from the side,
  past the label, so a text spacer would needlessly loop it up to the label band and back.
- **Excluded from the cross-container column snap.** Unlike cross-container spacers, text-content
  spacers are **not** passed to `cross_container_spacers_snap_to_column`
  ([edge_paths.md](edge_paths.md)). This keeps the detour **local** to the text band -- the edge
  approaches at its normal column, bows out only around the label, then returns to its rank column
  for the descendants -- rather than the whole descent column being pulled onto the label's far side.
- **Stored in `text_content_spacer_taffy_node_ids`**, merged as a routing waypoint and ordered by
  main-axis coordinate (the label sits above rank 0, so it is the first waypoint).

To keep the return jogs (text-band column back to each edge's rank column) at visually distinct
depths, the text `Row` is given a **bottom margin** of `N * TEXT_CONTENT_SPACER_GAP_PX` (where `N`
is the number of text spacers), enlarging only the label -> rank-0 gap so the protrusion band has
room to stagger one jog depth per edge.

> Note: appending the spacers grows the node's width by more than the bare spacer + gap sum, due to
> a taffy slack-distribution quirk in the text `Row` that has not been fully pinned down. This is
> cosmetic and may be revisited.


## When Spacer Building is Triggered

Spacer building is orchestrated by `IrToTaffyBuilder` in
[`crate/input_ir_rt/src/ir_to_taffy_builder.rs`](crate/input_ir_rt/src/ir_to_taffy_builder.rs).

After all diagram nodes and their child hierarchies have been added to the taffy tree, spacers are
inserted in five stages per dimension:

1. **Nested same-level spacers.** For every container diagram node (one that has children),
   `build_taffy_nodes_for_node_with_child_hierarchy` calls `EdgeSpacerBuilder::build` once per
   entity type (`ThingDefault`, `TagDefault`, `ProcessDefault`), passing `lca_node_id = Some(&container_id)`.
   This inserts spacers for edges whose LCA is exactly this container.

2. **Nested cross-container spacers.** The same function also calls
   `EdgeSpacerBuilder::build_cross_container_spacers` once per container, inserting spacers for
   edges that enter the container from outside.

3. **Nested edge description container spacers.** After `EdgeDescriptionBuilder::build` has
   produced `edge_description_container` nodes for this nesting level,
   `build_taffy_nodes_for_node_with_child_hierarchy` calls
   `EdgeSpacerBuilder::build_edge_desc_container_spacers` once per entity type with
   `lca_node_id = Some(&container_id)`. Each edge whose rank span crosses a `BetweenRanks`
   `edge_description_container` at this level, or whose sibling-index span crosses a `SameRank` one
   (other than its own container, in either case) receives a spacer leaf appended as a child of that
   container.

4. **Top-level same-level spacers.** After all containers have been processed,
   `build_taffy_trees_for_dimension` calls `EdgeSpacerBuilder::build` once per entity type with
   `lca_node_id = None`. This inserts spacers for top-level edges whose LCA is the diagram root.

5. **Top-level edge description container spacers.** Immediately after the top-level
   `EdgeDescriptionBuilder::build` calls, `build_taffy_trees_for_dimension` calls
   `EdgeSpacerBuilder::build_edge_desc_container_spacers` once per entity type with
   `lca_node_id = None`. Each top-level edge whose rank span (or, for a same-rank container,
   sibling-index span) crosses a top-level `edge_description_container` (other than its own)
   receives a spacer inside that container.

All five stages accumulate their results into a single `Map<EdgeId, EdgeSpacerTaffyNodes>`. Stages
1-3 return results that are merged into `nested_edge_spacer_taffy_nodes` and carried back up to
the top level. Stages 4-5 extend the top-level map directly. The fully-merged map is stored as
`TaffyNodeMappings::edge_spacer_taffy_nodes`.

Edge description container spacer results are merged entry-by-entry (not with a plain `extend`)
to preserve any rank or cross-container spacer entries that may already exist for the same edge.


## Same-Level Cross-Rank Spacers: EdgeSpacerBuilder::build

Source: [`crate/input_ir_rt/src/ir_to_taffy_builder/edge_spacer_builder.rs`](crate/input_ir_rt/src/ir_to_taffy_builder/edge_spacer_builder.rs)

### Signature

    EdgeSpacerBuilder::build(
        taffy_tree:          &mut TaffyTree<TaffyNodeCtx>,
        edge_groups:         &EdgeGroups,
        node_nesting_infos:  &NodeNestingInfos,
        node_ranks_nested:   &NodeRanksNested,
        entity_types:        &EntityTypes,
        target_entity_type:  &EntityType,
        rank_to_taffy_ids:   &mut BTreeMap<NodeRank, Vec<taffy::NodeId>>,
        lca_node_id:         Option<&NodeId>,
    ) -> Map<EdgeId, EdgeSpacerTaffyNodes>

The `rank_to_taffy_ids` parameter is modified in place: spacers are inserted into its vectors.
The function also maintains an internal `rank_spacer_counts: BTreeMap<NodeRank, Vec<usize>>` to
track how many spacers have already been inserted at each position within each rank, so that
successive edges insert their spacers at the correct offset.

### Algorithm: edge_spacers_build (per edge)

For each edge in every edge group the function calls the private `edge_spacers_build` helper. The
helper performs the following steps:

**Step 1 -- Resolve nesting infos.**
Look up `NodeNestingInfo` for both `edge.from` and `edge.to` in `node_nesting_infos`. Return early
(`None`) if either is missing.

**Step 2 -- Compute LCA depth and divergent ancestors.**

    lca_depth      = LcaDepthCalculator::calculate(nesting_info_from, nesting_info_to)
    divergent_from = nesting_info_from.ancestor_chain[lca_depth]
    divergent_to   = nesting_info_to.ancestor_chain[lca_depth]

Return `None` if either index is out of bounds (one node is an ancestor of the other).

**Step 3 -- Entity type filter.**
Both `divergent_from` and `divergent_to` must match `target_entity_type` in `entity_types`. If
either does not match, return `None`. This ensures that, for example, a cross-rank edge between
things does not accidentally insert spacers into a process rank container.

**Step 4 -- Compute rank span.**
Using `node_ranks_nested`, look up the rank of `divergent_from` and `divergent_to` in the rank map
for their shared parent container. The shared parent container is:

    lca_container = nesting_info_from.ancestor_chain[lca_depth - 1]  (None if lca_depth == 0)

    rank_from = container_ranks[divergent_from]
    rank_to   = container_ranks[divergent_to]
    rank_low  = min(rank_from, rank_to)
    rank_high = max(rank_from, rank_to)

Return `None` if `rank_low == rank_high` (same visual row, no intermediate ranks) or if
`rank_high - rank_low <= 1` (adjacent rows, no intermediate ranks).

**Step 5 -- LCA level filter.**
The `lca_node_id` parameter controls which hierarchy level this call handles:

- `lca_node_id = None` (top-level call): return `None` unless `lca_depth == 0`.
- `lca_node_id = Some(expected_id)` (nested call): return `None` if `lca_depth == 0`, or if
  `nesting_info_from.ancestor_chain[lca_depth - 1] != expected_id`.

This ensures each edge's same-level spacers are inserted exactly once, into the correct rank
container group.

**Step 6 -- Compute insertion base index.**

    from_sibling_index = nesting_info_from.nesting_path[lca_depth]
    to_sibling_index   = nesting_info_to.nesting_path[lca_depth]
    insertion_base_index = (from_sibling_index + to_sibling_index) / 2 + 1

This places the spacer between the two divergent sibling subtrees.

**Step 7 -- Insert a spacer at every intermediate rank.**
For each `rank` in the exclusive range `(rank_low + 1 .. rank_high)`:

1. Create a taffy leaf node with `TaffyNodeCtx::EdgeSpacer { edge_id, rank }` and the spacer style.
2. Compute the effective insertion index (see next section).
3. Insert the new taffy node ID into `rank_to_taffy_ids[rank]` at the effective index.
4. Update `rank_spacer_counts[rank]` to record the insertion.
5. Record `rank -> spacer_taffy_node_id` in `EdgeSpacerTaffyNodes::rank_to_spacer_taffy_node_id`.

### Insertion Index Computation

Because multiple edges may insert spacers at the same rank, later insertions must account for the
nodes that earlier insertions have already placed. The effective index is computed as follows:

    spacers_at_or_before = sum(rank_spacer_counts[rank][0 .. insertion_base_index])
    effective_index      = min(insertion_base_index + spacers_at_or_before, current_len)

Where `current_len` is `rank_to_taffy_ids[rank].len()` at the time of insertion. Capping at
`current_len` ensures the spacer is appended when the computed index would exceed the end of the
vector.

After insertion, `rank_spacer_counts[rank]` is updated by inserting `1` at `effective_index`,
shifting all subsequent counts right. This keeps the count vector aligned with the position vector
so future insertions at the same rank compute the correct offset.

Both the base-index formula and the effective-index/count-tracking logic above live in a shared
helper, [`RankSiblingInserter`](crate/input_ir_rt/src/ir_to_taffy_builder/rank_sibling_inserter.rs)
(`insertion_base_index_compute`, `node_insert`), rather than being private to `EdgeSpacerBuilder`.
`EdgeDescriptionBuilder` calls the same helper to place same-rank edge description containers (see
[edge_descriptions.md](edge_descriptions.md) -- Same-Rank (Cycle Edge) Placement) between two
same-ranked divergent ancestors, keeping its own separate `same_rank_insertion_counts` tracker since
the two builders insert into `rank_to_taffy_ids` at different times.


## Cross-Container Spacers: EdgeSpacerBuilder::build_cross_container_spacers

Source: [`crate/input_ir_rt/src/ir_to_taffy_builder/edge_spacer_builder.rs`](crate/input_ir_rt/src/ir_to_taffy_builder/edge_spacer_builder.rs)

### Signature

    EdgeSpacerBuilder::build_cross_container_spacers(
        taffy_tree:                 &mut TaffyTree<TaffyNodeCtx>,
        edge_groups:                &EdgeGroups,
        node_nesting_infos:         &NodeNestingInfos,
        node_ranks_nested:          &NodeRanksNested,
        rank_to_taffy_ids:          &mut BTreeMap<NodeRank, Vec<taffy::NodeId>>,
        container_node_id:          &NodeId,
        container_node_hierarchy:   &NodeHierarchy,
    ) -> Map<EdgeId, EdgeSpacerTaffyNodes>

Returns immediately with an empty map if `container_node_hierarchy` has one or fewer direct
children, because a single-child container has no siblings that could block an edge.

### Purpose

When an edge has one endpoint outside a container and the other endpoint nested inside (at any
depth), the edge path must travel alongside the container's internal children to reach the target.
Cross-container spacers mark the positions of those intermediate siblings so the path builder can
use them as waypoints.

Only siblings whose rank is **strictly less than** the target child's rank receive a spacer. A
sibling at the same rank as the target is placed side-by-side with it and does not obstruct the
incoming edge. Siblings at higher ranks are beyond the target and are also outside the path.
Consequently, if the target child is at rank 0 there are no qualifying siblings and no spacers are
created.

At most one spacer is created per rank value within a given edge. Multiple siblings at the same
rank occupy the same layout row, so a single spacer is sufficient to route around the entire row.

### Decision Logic: EdgeSpacerBuildDecider::decide

Source: [`crate/input_ir_rt/src/ir_to_taffy_builder/edge_spacer_builder/edge_spacer_build_decider.rs`](crate/input_ir_rt/src/ir_to_taffy_builder/edge_spacer_builder/edge_spacer_build_decider.rs)

For each edge, `EdgeSpacerBuildDecider::decide` returns either `EdgeSpacerBuildDecision::Build`
or `EdgeSpacerBuildDecision::Skip`. It proceeds through the following checks in order:

Note: the decision rests only on containment and the target's position *inside* the container --
**not** on the root-level distance between the two divergent ancestors. An edge into a deeply-ranked
child must route around the container's lower-rank siblings even when its divergent ancestor is
adjacent to the from-node's divergent ancestor. (The actual "is a spacer needed?" question is
answered by the [Spacer Insertion](#spacer-insertion) loop, which only routes around siblings at
ranks strictly before the target; a target at rank 0 therefore yields no spacers regardless.)

**Check 1 -- Nesting info availability.**
If `NodeNestingInfo` is missing for `edge.from` or `edge.to`, skip with
`NestingInfoFromNotFound` / `NestingInfoToNotFound`.

**Check 2 -- Containment check.**
Determine which endpoints are inside `container_node_id` by searching each endpoint's
`ancestor_chain` for the container ID:

    container_contains_from = container_node_id in info_from.ancestor_chain
    container_contains_to   = container_node_id in info_to.ancestor_chain

Skip with `ContainerNodeContainsBothFromAndToNodes` if both are inside.
Skip with `ContainerNodeContainsNeitherFromAndToNodes` if neither is inside.
Continue only when exactly one endpoint is inside.

**Check 3 -- Find the target child.**
Let `info_inside` be the nesting info of the contained endpoint. Find the depth of
`container_node_id` in `info_inside.ancestor_chain`:

    container_depth = position of container_node_id in info_inside.ancestor_chain
    target_child_id = info_inside.ancestor_chain[container_depth + 1]

If `container_depth + 1` is out of bounds (the contained endpoint IS the container node itself),
skip with `ContainerNodeIsFromOrToNode`.

**Result.**
Return `EdgeSpacerBuildDecision::Build { target_child_id }` where `target_child_id` is the direct
child of the container whose subtree contains the inside endpoint.

### Spacer Insertion

After `EdgeSpacerBuildDecider::decide` returns `Build`, the following steps are performed in
`build_cross_container_spacers_for_edge`:

1. Look up `target_rank`: the rank of `target_child_id` among the container's direct children
   using `node_ranks_nested.ranks_for(Some(container_node_id))`.

2. Iterate over every other direct child (`sibling_id`) of the container:

   a. Skip `sibling_id == target_child_id`.

   b. Look up `sibling_rank` using the same container's rank map.

   c. Skip if `sibling_rank >= target_rank` (sibling is at or past the target, not between the
      container entry and the target).

   d. Skip if this rank has already received a spacer for this edge (tracked via a local
      `ranks_with_spacers: BTreeSet<NodeRank>`).

   e. Create a taffy leaf node with `TaffyNodeCtx::EdgeSpacer { edge_id, rank: sibling_rank }` and
      the spacer style.

   f. Append the new node ID to `rank_to_taffy_ids[sibling_rank]` (cross-container spacers are
      always appended, not inserted at a computed position).

   g. Record the node ID in `EdgeSpacerTaffyNodes::cross_container_spacer_taffy_node_ids`.

3. If any spacers were created, merge them into the `edge_spacer_taffy_nodes` map under the edge's
   ID.


## Data Produced

All spacer nodes from both kinds of building are accumulated into:

    TaffyNodeMappings::edge_spacer_taffy_nodes: Map<EdgeId, EdgeSpacerTaffyNodes>

The `EdgeSpacerTaffyNodes` for each edge contains:

    EdgeSpacerTaffyNodes {
        rank_to_spacer_taffy_node_id:                         Map<NodeRank, taffy::NodeId>,
        cross_container_spacer_taffy_node_ids:                Vec<taffy::NodeId>,
        edge_desc_container_spacer_taffy_node_ids:            Vec<taffy::NodeId>,
        same_rank_edge_desc_container_spacer_taffy_node_ids:  Vec<taffy::NodeId>,
        text_content_spacer_taffy_node_ids:                   Vec<taffy::NodeId>,
    }

`rank_to_spacer_taffy_node_id` maps each intermediate rank value to the single spacer node
inserted there (same-level cross-rank spacers). Because each edge inserts at most one
same-level spacer per rank, a map keyed by rank is sufficient.

`cross_container_spacer_taffy_node_ids` is an unkeyed list of cross-container spacer node IDs.
Multiple cross-container spacers can share the same global rank value (each container has its own
independent rank numbering), so a map keyed by rank would conflate them. Instead the list is
ordered by iteration order, and the edge path builder uses the computed absolute positions after
layout to sort and route through them correctly.

`edge_desc_container_spacer_taffy_node_ids` is an unkeyed list of `BetweenRanks` edge description
container spacer node IDs. Each entry corresponds to a spacer appended inside one
`edge_description_container` that the edge's path crosses. Like cross-container spacers, the edge
path builder sorts all kinds together by main-axis coordinate after layout to determine the correct
waypoint order, and their coordinates are resolved via the direction-oblivious generic
`EdgeSpacerCoordinatesCalculator::calculate`.

`same_rank_edge_desc_container_spacer_taffy_node_ids` is the `SameRank` (cycle edge) counterpart:
an unkeyed list of spacer node IDs appended inside a same-rank `edge_description_container` that the
edge crosses without owning. These are tracked separately because their coordinates are resolved via
`EdgeSpacerCoordinatesCalculator::calculate_description_thread_same_rank` instead -- the rotated,
direction-aware calculation, using the `sibling_index_from_cmp_to` ordering recorded on each node's
own `EdgeSpacerCtx` -- not the generic `calculate`, since a same-rank container's children stack
perpendicular to the edge's own travel direction (see [Edge Description Container
Spacers](#3-edge-description-container-spacers)).

`text_content_spacer_taffy_node_ids` is an unkeyed list of text-content (node-label) spacer node
IDs -- one per cross-container edge that enters a described container to reach a node nested inside
it (see [Text-Content (Node-Label) Spacers](#4-text-content-node-label-spacers)). They are merged
and sorted with the other kinds by main-axis coordinate, but -- unlike the cross-container list --
are **excluded from the cross-container column snap**, so the detour around the label stays local
to the text band.

`TaffyNodeMappings::edge_spacer_taffy_nodes` is consumed by `TaffyToSvgElementsMapper` and the
edge path routing code in
[`edge_path_builder_pass_1.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/edge_path_builder_pass_1.rs).
The coordinates of spacer nodes are extracted by
[`EdgeSpacerCoordinatesCalculator`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/edge_spacer_coordinates_calculator.rs),
which walks up the taffy tree to accumulate each spacer's absolute position and returns entry and
exit points based on the configured `RankDir`.


## End-to-End Example

### Diagram input

    things (row layout):
      t_a:                   # rank 0
      t_b:                   # rank 1
      t_c:                   # rank 2
        t_c0:                # rank 0 within t_c
        t_c1:                # rank 1 within t_c

    edges:
      edge_a_c1: from t_a, to t_c1

### NodeNestingInfo

    t_a:
      ancestor_chain: ["t_a"]
      nesting_path:   [0]

    t_b:
      ancestor_chain: ["t_b"]
      nesting_path:   [1]

    t_c:
      ancestor_chain: ["t_c"]
      nesting_path:   [2]

    t_c0:
      ancestor_chain: ["t_c", "t_c0"]
      nesting_path:   [2, 0]

    t_c1:
      ancestor_chain: ["t_c", "t_c1"]
      nesting_path:   [2, 1]

### Same-level cross-rank spacers for edge_a_c1

Triggered by the top-level call `EdgeSpacerBuilder::build(..., lca_node_id = None)`:

    lca_depth      = 0   (chains ["t_a"] and ["t_c", "t_c1"] share no prefix)
    divergent_from = "t_a"   (ancestor_chain_from[0])
    divergent_to   = "t_c"   (ancestor_chain_to[0])
    lca_container  = None    (lca_depth == 0, root level)
    rank_from      = 0
    rank_to        = 2
    rank_low       = 0,  rank_high = 2
    intermediate ranks: [1]

    insertion_base_index = (0 + 2) / 2 + 1 = 2

A spacer leaf node tagged `EdgeSpacer { edge_id: edge_a_c1, rank: 1 }` is inserted at index 2 of
`rank_to_taffy_ids[rank=1]`, which is the rank 1 row of the top-level things container. This row
contains `t_b`. The spacer is positioned adjacent to `t_b` and serves as a waypoint so the edge
path passes through rank 1 space without crossing `t_b`.

    EdgeSpacerTaffyNodes for edge_a_c1:
      rank_to_spacer_taffy_node_id: { rank=1: <spacer_node_id> }

### Cross-container spacers for edge_a_c1

Triggered by `EdgeSpacerBuilder::build_cross_container_spacers(..., container_node_id = "t_c")`:

**Decider:**

    container_contains_from = "t_c" in ["t_a"]          = false
    container_contains_to   = "t_c" in ["t_c", "t_c1"]  = true
    exactly one endpoint is inside, continue

    info_inside = NodeNestingInfo of t_c1
    container_depth = position of "t_c" in ["t_c", "t_c1"] = 0
    target_child_id = ancestor_chain_inside[0 + 1] = "t_c1"

    Result: Build { target_child_id: "t_c1" }

**Spacer insertion:**

    target_rank = rank of "t_c1" within t_c = 1

    siblings of t_c1 inside t_c: [t_c0]
      t_c0: sibling_rank = 0
        sibling_rank (0) < target_rank (1) --> qualify
        ranks_with_spacers does not contain 0 --> qualify
        Create EdgeSpacer { edge_id: edge_a_c1, rank: 0 } and append to rank_to_taffy_ids[rank=0]

    EdgeSpacerTaffyNodes for edge_a_c1 (merged):
      rank_to_spacer_taffy_node_id:          { rank=1: <same_level_spacer_node_id> }
      cross_container_spacer_taffy_node_ids: [<cross_container_spacer_node_id>]

The cross-container spacer sits in rank 0 of `t_c`'s rank containers alongside `t_c0`. After
layout its absolute position is used by the edge path builder as a waypoint inside the container,
ensuring the path routes around `t_c0` as it descends toward `t_c1`.
