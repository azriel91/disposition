# Edge Paths

1. Nodes are connected to other nodes via edges
2. Nodes are laid out in a flex layout with recursive flex layout containers
3. Between nodes, "spacer nodes" may be inserted, which serve as coordinate markers for edge paths, so that when edge paths are calculated, the path is routed through spacer nodes to avoid drawing lines over the diagram nodes.
4. Nodes have a `NodeRank` that positions them along the rank axis within their container level. Ranks are stored in [`NodeRanksNested`](crate/ir_model/src/node/node_ranks_nested.rs), which holds a rank map for the root level and for each container node's direct children. Dependency edges that cross container boundaries are attributed to the lowest common ancestor (LCA) level. See [Node Rank Calculation](#node-rank-calculation) for details.
5. Part of the information gathered for calculating spacer nodes is collecting a `BTreeMap<NodeRank, Vec<taffy::NodeId>>`.
6. Calculation of where to place spacer nodes is done in [`ir_to_taffy_builder.rs`](crate/input_ir_rt/src/ir_to_taffy_builder.rs), in `fn build_taffy_child_nodes_for_node_by_rank`, called by `fn build_taffy_nodes_for_node_with_child_hierarchy`. Cross-container spacers (alongside sibling children of a container that an edge passes through) are only inserted for siblings whose rank is **strictly less than** the target child's rank. Siblings at the same rank as the target are side-by-side and do not block the incoming edge path; siblings at higher ranks are beyond the target and are similarly not in the path. In particular, edges that connect to a rank-0 node inside a container require no cross-container spacers at all, because there are no siblings between the container entry and rank 0. Furthermore, **at most one spacer is created per rank group**: if multiple siblings share the same rank (and therefore occupy the same layout row), a single spacer is sufficient to route the edge around the entire row -- creating one spacer per sibling would cause the path builder to zigzag through redundant waypoints.
7. Edge path calculation is done in two passes.
8. Both passes are called in [`svg_edge_infos_builder.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/svg_edge_infos_builder.rs)
9. The first pass calculates a path between the from-node and to-node without taking into account spacer nodes, and the information from this first path is used in subsequent calculations. This is defined in [`edge_path_builder_pass_1.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/edge_path_builder_pass_1.rs)
10. Between the first and second pass, offsets from where the edge path exits the from-node, and where it enters the to-node, are computed, so that multiple edges do not all touch the from-node / to-node at the same coordinate, for visual clarity. See the [Offset Calculation](#offset-calculation) section below for details.
11. Also, for orthogonal (strictly horizontal / vertical) edge paths, a "protrusion" length is calculated so that the paths exit the node perpendicular to the node face for some length, so that the path is not drawn directly on the node face as a tangential line. See the [Protrusion Calculation](#protrusion-calculation) section below for details.
12. Offsets are the coordinate shift to where the edge path contacts the node face.
13. Protrusion is the length that the edge path extends out of the node face, so that an edge path isn't drawn directly on a node face.
14. The second pass is defined in [`edge_path_builder_pass_2.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/edge_path_builder_pass_2.rs)
15. The second pass computes the edge paths with the offsets and protrusion, which should result in paths that are visually non-overlapping with other paths and node content, creating visual clarity.


## Node Rank Calculation

Node ranks are stored in [`NodeRanksNested`](crate/ir_model/src/node/node_ranks_nested.rs) and computed by [`NodeRanksCalculator`](crate/input_ir_rt/src/node_ranks_calculator.rs). Ranks are hierarchy-aware: each container node has its own [`NodeRanks`](crate/ir_model/src/node/node_ranks.rs) for its direct children, computed independently from other levels.


### Why hierarchy-aware ranks exist

In a nested node hierarchy, dependency edges may connect nodes at different nesting depths. A flat rank assignment would assign ranks globally, conflating unrelated nodes at different levels. By computing `NodeRanks` per level, each container's children are ranked relative to their siblings only, which correctly drives their layout position within their container.


### Concepts

- **Level**: a set of sibling nodes sharing the same parent container. The root level consists of the top-level nodes in the diagram. Each container node defines its own level for its direct children.
- **Container**: a node that has at least one direct child. Its direct children form a level with their own `NodeRanks`.
- **LCA (Lowest Common Ancestor)**: for two nodes `A` and `B`, the deepest node in the hierarchy that is an ancestor of both. For same-level siblings, the LCA is their shared parent. For top-level nodes, the LCA is the diagram root.
- **Divergent ancestor**: given two nodes and their LCA, the divergent ancestor of a node is the direct child of the LCA that is an ancestor of (or equal to) that node. It is the element at depth `lca_depth` in the node's `ancestor_chain`.
- **LCA-level edge**: for an edge `(from, to)`, the corresponding edge `(divergent_from, divergent_to)` between the two divergent ancestors at the LCA. Cross-container edges are "lifted" to the LCA level.


### Algorithm

`NodeRanksNested` is computed in four steps:

1. **Build container-to-children map.** Using [`NodeNestingInfos`](crate/ir_model/src/node/node_nesting_infos.rs), group each node under its parent container: the second-to-last element of its `ancestor_chain`, or `None` for top-level nodes.

2. **Collect dependency edges.** Dependency edges are extracted from `EdgeGroups` by checking entity types for `DependencyEdge*` variants.

3. **Lift edges to LCA level.** For each dependency edge `(from, to)`:
   - Look up `NodeNestingInfo` for both endpoints.
   - Compute the LCA depth: the length of the common prefix of the two `ancestor_chain`s.
   - Derive `divergent_from = ancestor_chain_from[lca_depth]` and `divergent_to = ancestor_chain_to[lca_depth]`.
   - Derive `lca_container = ancestor_chain_from[lca_depth - 1]` (or `None` if `lca_depth == 0`).
   - Skip edges where one node is an ancestor of the other (`lca_depth >= min chain length`).
   - Group the LCA-level edge `(divergent_from, divergent_to)` under `lca_container`.

4. **Compute ranks per level.** For each container and its direct children, run the SCC-based longest-path rank assignment using only the LCA-level edges for that container. Nodes in cycles receive the same rank.


### Example

For the hierarchy and edges:

```yaml
node_hierarchy:
  a: { a_child: {} }
  b: { b_child_0: {}, b_child_1: {} }
  c: { c_child: {} }
  d: { d_child: {} }

edges:
  edge_dep_a__b:                  { from: a,         to: b         }
  edge_dep_b_child_0__b_child_1:  { from: b_child_0, to: b_child_1 }
  edge_dep_b_child_0__c_child:    { from: b_child_0, to: c_child   }
```

Edge attribution:

- `edge_dep_a__b`: `ancestor_chain(a) = [a]`, `ancestor_chain(b) = [b]`, LCA depth = 0, LCA = root, LCA-level edge = `(a, b)` at root level.
- `edge_dep_b_child_0__b_child_1`: `ancestor_chain(b_child_0) = [b, b_child_0]`, `ancestor_chain(b_child_1) = [b, b_child_1]`, LCA depth = 1, LCA = `b`, LCA-level edge = `(b_child_0, b_child_1)` at `b`'s level.
- `edge_dep_b_child_0__c_child`: `ancestor_chain(b_child_0) = [b, b_child_0]`, `ancestor_chain(c_child) = [c, c_child]`, LCA depth = 0, LCA = root, divergent ancestors = `b` and `c`, LCA-level edge = `(b, c)` at root level.

Resulting `NodeRanksNested`:

```yaml
node_ranks_nested:
  root:
    a: 0
    b: 1
    c: 2  # LCA-lifted from edge_dep_b_child_0__c_child: b -> c
    d: 0
  containers:
    a:
      a_child: 0  # no edges in a's level
    b:
      b_child_0: 0
      b_child_1: 1
    c:
      c_child: 0  # edge_dep_b_child_0__c_child is at root level, not c's level
    d:
      d_child: 0  # no edges in d's level
```


## Offset Calculation (Label-Aligned with Slot Fallback)

Offset calculation places each edge's contact point along a node face at a position that either aligns with the edge's description label or, when no label is present, spreads contacts evenly so they do not overlap. The offset for each edge is a signed pixel distance from the face midpoint. Offsets are computed in `fn face_offsets_compute` in [`svg_edge_infos_builder.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/svg_edge_infos_builder.rs), with label-based offsets derived via `SvgEdgeInfosBuilder::label_face_offset_compute` / `SvgEdgeInfosBuilder::taffy_node_absolute_xy_compute`, and slot-based fallback arithmetic provided by [`edge_face_contact_tracker.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/edge_face_contact_tracker.rs).


### Why offsets exist

When multiple edges connect to the same face of a node (e.g. three edges all exit from the Bottom face), their paths would all originate from the exact same point -- the face midpoint. Offsets shift each edge's contact point along the face so the paths fan out, making individual edges visually distinguishable. When an edge has a description label rendered on the face, the offset additionally serves to route the edge path's contact point to the entry-side edge of that label, preventing the path from cutting through the label text.


### Concepts

- **Face contact**: a single edge endpoint touching a specific face of a specific node. Each edge contributes up to two contacts: one for its from-node face and one for its to-node face.
- **Slot**: an ordered position within the list of contacts on a single (node, face) pair. Slot 0 is the first sorted contact, slot 1 the second, etc.
- **Face length**: the pixel length of the face: width for `Top`/`Bottom` faces, collapsed height for `Left`/`Right` faces. Computed by `fn face_length_for_node`.
- **Gap**: the pixel spacing between adjacent contact points. Starts at 10% of the face length (`CONTACT_GAP_RATIO = 0.10`), clamped to at least 12 px (`CONTACT_GAP_MIN_PX = 12.0`, sized to clear the arrow head's 4 px half-width on both sides with margin). If `contact_count * gap > face_length`, the gap is shrunk to `face_length / contact_count` so all contacts fit.
- **Label-based offset**: when an edge has a non-zero description label node on a face (looked up from `TaffyNodeMappings::edge_label_taffy_nodes`), the contact point is routed to the entry-side edge of the label -- the side the path arrives at first. Because sibling insertion order is reversed for reversed rank directions (see [Sibling order for reversed rank directions](#sibling-order-for-reversed-rank-directions)), the entry side is the same for all rank directions:
  - `Top`/`Bottom` faces (offset along x): the left x (`label_abs_x`).
  - `Left`/`Right` faces (offset along y): the top y (`label_abs_y`).
  - `offset = label_contact - face_midpoint_along_axis` where `face_midpoint_x = node_info.x + node_info.width / 2` and `face_midpoint_y = node_info.y + node_info.height_collapsed / 2`.
  - The label-based offset is used only when the label has non-zero size (`label_width > 0` for Top/Bottom, `label_height > 0` for Left/Right); otherwise the slot-based fallback is used.


### Algorithm

The algorithm in `face_offsets_compute` proceeds in three phases:

1. **Collect face contact entries.** For each edge in every group, if the from-face or to-face is known, a `FaceContactEntry` is recorded in a map keyed by `(NodeId, NodeFace)`. Each entry captures the `rank_distance` (absolute rank difference between from and to), the target node's x/y coordinates, and the edge's group/index.

2. **Sort entries and assign slot indices (`fn face_entries_sort_by_rank_and_coordinate`).** For each (node, face) group, entries are sorted by:
    - **Primary key**: `rank_distance` ascending. Edges spanning fewer ranks get slots closer to the face midpoint, keeping short-range paths on the inside.
    - **Secondary key** (tie-breaker) -- the target node's coordinate along the face axis: x ascending for `Top`/`Bottom` faces, y ascending for `Left`/`Right` faces. This ensures co-ranked edges follow the spatial order of their targets.
    - After sorting, each entry is assigned its slot index (0-based position in the sorted order). These slot indices are written back into `from_slot_indices` / `to_slot_indices` on the pass-1 group.

3. **Compute offset values.** For each contact entry, the offset is chosen as follows:
    - **Label-based** (preferred): if the edge has a non-zero label node on this face (looked up from `edge_label_taffy_nodes` via `label_face_offset_compute`), the offset is `label_contact_on_face_axis - face_midpoint_on_face_axis`, where `label_contact` is the entry-side edge of the label (left x for `Top`/`Bottom` faces; top y for `Left`/`Right` faces). The absolute position is computed via `taffy_node_absolute_xy_compute`.
    - **Slot-based fallback**: when the edge has no label on this face, or the label has zero size, `EdgeFaceContactTracker::offset_for_index` is used: `offset[i] = (i - (n - 1) / 2.0) * gap`. This distributes offsets symmetrically around 0 (the face midpoint). The first slot gets the most negative offset (leftward/upward), the middle slot(s) get ~0, and the last slot gets the most positive offset (rightward/downward).
    - **Self-loop separation enforcement** (`fn face_offsets_self_loop_separation_enforce`): a self-loop's from and to contacts share one (node, face), but come from different sources -- the from contact is label-aligned (the edge label leaf always has a non-zero padded size, e.g. 4 px from `EDGE_LABEL_PADDING_PX`, so the zero-size fallback never triggers), while the to contact has no label leaf (`to_face` is `None` in the IR assignment) and uses the slot-based fallback. The two coordinate systems are unrelated, so the contacts can land arbitrarily close together (e.g. 3 px apart), placing the from segment inside the arrow head drawn at the to contact. When the separation is below the face's contact gap, the to offset is moved to `from_offset +/- gap`, preferring the candidate that stays within the face and has the most clearance from the other contacts on the face.


### Sibling order for reversed rank directions

For `BottomToTop` and `RightToLeft`, rank containers are laid out with a reversed flex direction (`RowReverse` / `ColumnReverse`). The reversed direction is retained because the rank-stacking parent inverts it to stack ranks bottom-up / right-to-left. Left as-is, the reversed direction would also render siblings *within* a rank in reverse declaration order, which is the reverse of what a human reading the input expects and would invert the spatial assumptions of the offset sort order, causing edge paths to cross.

To compensate, `TaffyContainerBuilder::rank_taffy_ids_reverse_if_direction_reversed` (in [`taffy_container_builder.rs`](crate/input_ir_rt/src/ir_to_taffy_builder/taffy_container_builder.rs)) reverses each rank's child insertion order when the rank container style uses a reversed flex direction. It is called after edge spacers are inserted into the per-rank vectors (so spacers flip together with their neighbouring nodes), both for first-level rank containers and for nested container children. As a result, visual order matches declaration order for **all** rank directions, and no offset negation or per-direction entry-side selection is needed downstream.


### How offsets are applied

Offsets are stored in `EdgeContactPointOffsets` (a `Vec<f32>` indexed by slot) and looked up during pass 2 via the slot indices assigned in phase 2. The offset is applied by `EdgePathBuilderPass1::face_offset_apply`:

- For `Top`/`Bottom` faces: `x += offset` (shifts the contact point horizontally along the face).
- For `Left`/`Right` faces: `y += offset` (shifts the contact point vertically along the face).


## Protrusion Calculation

Protrusion calculation is implemented in [`ortho_protrusion_calculator.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/ortho_protrusion_calculator.rs). The output is an `OrthoProtrusionParams` per edge (defined in [`edge_path_builder_pass_2_ortho.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/edge_path_builder_pass_2/edge_path_builder_pass_2_ortho.rs)), containing a `from_protrusion`, a `to_protrusion`, and a `Vec<SpacerProtrusionParams>` with per-spacer `entry_protrusion` and `exit_protrusion` values.


### Why protrusions exist

16. When multiple orthogonal edge paths share the same node face or cross the same inter-rank gap, their horizontal/vertical routing segments can overlap visually. A protrusion is a short perpendicular stub drawn where the path exits or enters a node face (or spacer boundary), pushing the main routing segment away from the face. By assigning different protrusion depths to different edges, each edge's routing segment runs at a distinct distance from the node face, preventing overlap.


### Concepts

17. **Rank gap**: the space between two adjacent ranks (e.g. between rank 1 and rank 2). Every edge endpoint that exits or enters a node face protrudes into a rank gap. Every spacer entry/exit boundary also protrudes into a rank gap.
18. **Rank gap key**: a `(rank_low, rank_high)` pair identifying a specific rank gap. All endpoints (from-node, to-node, spacer entry, spacer exit) that protrude into the same gap are grouped together by this key. Implemented as the `RankGapKey` struct.
19. **Gap side**: each endpoint in a rank gap protrudes from either the `Low` side (the `rank_low` boundary) or the `High` side (the `rank_high` boundary). For example, a from-node at rank 0 connecting to rank 3 protrudes from the `Low` side of gap (0, 1), while the to-node protrudes from the `High` side of gap (2, 3).
20. **Rank gap entry**: a `RankGapEntry` record representing one endpoint in one rank gap. It stores: the edge's group/index, which endpoint kind it is (`FromEndpoint`, `ToEndpoint`, `SpacerEntry`, `SpacerExit`), which gap side, the cross-axis coordinate (perpendicular to the rank direction), the face offset (slot offset from face midpoint), and the pixel distance of the rank gap.
21. **Crossing edge**: an edge that has entries on **both** sides of the same rank gap (e.g. a from-endpoint on the `Low` side and a spacer entry on the `High` side of the same gap). Crossing edges need special treatment to avoid their routing midpoints coinciding.
22. **Cross-axis coordinate**: the coordinate perpendicular to the rank direction. For `Top`/`Bottom` faces (vertical rank flow) this is the X coordinate; for `Left`/`Right` faces (horizontal rank flow) this is the Y coordinate. Used to sort endpoints spatially within a gap.
23. **Node category**: a coarse grouping of node entity types used to keep protrusion and rank-gap calculations independent across unrelated node groups. The categories are:
    - `Thing` -- `ThingDefault` nodes.
    - `Tag` -- `TagDefault` nodes.
    - `Process` -- `ProcessDefault` and `ProcessStepDefault` nodes.
    - `Other` -- nodes with no recognised entity type.

    When computing rank-gap boundaries (for cycle edges) or divergent-sibling extents (for all edges), only nodes within the **same category** as the edge's from-node are considered. This prevents thing-node edges from being routed around process nodes, and tag-node edges from being routed around thing nodes, even when those unrelated nodes share the same rank in the layout. Implemented by `OrthoProtrusionCalculator::node_category` and applied in `cycle_edge_collect_rank_gap_entries` and `min_protrusion_divergent_sibling_extent`.


### Algorithm overview (`fn calculate`)

The algorithm in `OrthoProtrusionCalculator::calculate` has four steps:

23. **Step 1: Resolve spacer coordinates and initialize output.**

    Spacer coordinates are resolved once per edge (via `spacer_coordinates_resolve`). The output `Vec<Vec<OrthoProtrusionParams>>` is initialized with all protrusions set to `0.0` and `spacer_protrusions` sized to match the resolved spacer count.

24. **Step 2: Collect rank gap entries.**

    For each edge across all groups:

    - **Same-rank (cycle) edges** with `Top` or `Bottom` faces are handled by `cycle_edge_collect_rank_gap_entries`. Both the from-endpoint and the to-endpoint are registered as same-side entries in the adjacent rank gap: `Top` face -> gap `(rank-1, rank)` on the `High` side; `Bottom` face -> gap `(rank, rank+1)` on the `Low` side. The `rank_gap_px` for each endpoint is the pixel distance from the node's face to the nearest boundary of the adjacent rank (the maximum bottom edge of rank-R-1 nodes for `Top`, or the minimum top edge of rank-R+1 nodes for `Bottom`). **Only nodes of the same category as the from-node** are included in this boundary search, so thing-node cycle edges are not pushed out by process nodes at the same rank. Cycle edges with `Left` or `Right` faces are skipped here and fall through to the `MIN_PROTRUSION_PX` safety net in Step 6.
    - **Non-cycle edges**: The from-endpoint is registered in the rank gap between the from-node's rank and the adjacent rank toward the to-node. The to-endpoint is registered in the rank gap between the to-node's rank and the adjacent rank toward the from-node.
    - Each intermediate spacer contributes two entries: its entry side protrudes into the gap before it, and its exit side protrudes into the gap after it. The first spacer's entry shares the same gap as the from-endpoint (opposite side), and the last spacer's exit shares the same gap as the to-endpoint (opposite side).
    - Each entry records its `GapSide`, cross-axis coordinate, face offset, and rank gap pixel distance (computed by `rank_gap_px` for node endpoints or `spacer_gap_px` for spacer-to-spacer gaps).

25. **Step 3: Assign protrusion depths (`fn protrusions_assign`).**

    For each rank gap, all collected entries are assigned distinct protrusion depths. See [Protrusion depth assignment](#protrusion-depth-assignment) below.

26. **Step 4: Propagate node protrusions to shared spacer sides (fallback).**

    If the first spacer's entry protrusion or the last spacer's exit protrusion was not assigned (because the node face was `None`), it falls back to the from/to protrusion value as a safety net.

27. **Step 5: Enforce minimum protrusions to clear divergent ancestor siblings (`fn protrusions_adjust_for_divergent_siblings`).**

    For edges where the from/to nodes are at different nesting levels, each endpoint's protrusion must be large enough to clear all same-rank sibling nodes of the endpoint's **Divergent ancestor** at the LCA level. **Only nodes of the same category as the endpoint node** are considered as siblings, so a thing-node endpoint is not made to clear process nodes that happen to share the same rank.

    The FROM endpoint adjustment is always applied. The **TO endpoint adjustment is skipped when the edge has cross-container spacers**: in that case the spacer already handles routing inside the to-node's container, so the `to_protrusion` only needs to reach the spacer exit (not exit the container's far boundary). Applying the adjustment with a spacer present would force `to_protrusion` all the way to the container boundary, causing the path to overshoot the spacer, re-enter the container from the outside, and produce a visual zigzag.


### Protrusion depth assignment (`fn protrusions_assign`)

This function assigns protrusion depths to all endpoints within a single rank gap:

27. **Find the tightest constraint.**

    The minimum `rank_gap_px` across all entries in the gap determines the available space. The total protrusion band shared by **both** sides of the gap is `available = min_gap_px * MAX_GAP_FRACTION` (where `MAX_GAP_FRACTION = 0.8`). The from-side and to-side fans grow from opposite gap boundaries toward each other, so capping their **combined** depth at `available` leaves `(1 - MAX_GAP_FRACTION) * gap` (20%) as the central routing segment and guarantees the deepest from-tip and deepest to-tip never cross.

28. **Small gap fallback.**

    If `available` is less than `MIN_PROTRUSION_PX` (3.0 px), or the two sides' floors (see Step 31) cannot both fit within `available`, all entries receive a minimal protrusion of `min(MIN_PROTRUSION_PX, min_gap_px * 0.5)` (to-endpoints are still floored in `protrusion_write`).

29. **Partition by gap side and sort.**

    Entries are partitioned into `Low` and `High` groups. Each group is sorted by face offset (ascending), then cross-axis coordinate. This spatial ordering ensures that edges whose contact points are further apart receive longer protrusions, and edges closer together receive shorter ones, reducing visual cross-over.

30. **Identify crossing edges.**

    Edges that appear on both sides of the gap are identified. Their protrusion depths on each side are assigned independently based on that side's spatial ordering, so the high-side ordering is not dictated by the low-side sort.

31. **Split the band proportionally and assign per-side depths.**

    Each side is ordered deepest-first as `single-side entries ++ crossing entries` (low side: `single_low ++ crossing_low`; high side: `crossing_high ++ single_high`), preserving the previous convention where single-side edges protrude further than crossing edges on the same side.

    Each side reserves a **floor**: a side containing any `ToEndpoint` uses `TO_PROTRUSION_MIN_PX` (11.0 px, the arrow-head clearance) as its shallowest depth; from / spacer-only sides use `MIN_PROTRUSION_PX` (3.0 px). A backward edge reverses the from/to sides, so the actual endpoint kinds are inspected rather than the side name.

    The growable slack above the two floors (`slack = available - low_floor - high_floor`) is split **proportionally to each side's endpoint count**, so per-protrusion spacing stays even even when the from/to counts are imbalanced:
    - `low_band  = low_floor  + slack * n_low  / (n_low + n_high)`
    - `high_band = high_floor + slack * n_high / (n_low + n_high)`

    By construction `low_band + high_band <= available`. Within each side's `[floor, band]` range, the `n` entries receive distinct deepest-first depths `band - i * (band - floor) / (n - 1)` (index 0 = `band`, the deepest); a lone entry sits at the midpoint of its band.

32. **Arrow-head clearance floor for to-endpoints.**

    Every edge has an arrow head drawn at its to-endpoint, occupying `ARROW_HEAD_LENGTH` (8.0 px) of the path's final straight segment. To keep the Z/S bend clear of the arrow head, `protrusion_write` floors every `ToEndpoint` protrusion to `TO_PROTRUSION_MIN_PX` (`ARROW_HEAD_LENGTH + ARROW_HEAD_CLEARANCE_PX = 11.0` px), capped at `entry.rank_gap_px * MAX_GAP_FRACTION` so tight gaps are never overshot. The per-side floor in Step 31 already reserves this clearance on any to-side, so `protrusion_write` is consistent with the band allocation. Because the floor is applied centrally in `protrusion_write`, it covers the per-side distribution and the small-gap fallback alike, and Steps 4-5 (spacer fallback, divergent-sibling adjustment) see the floored value. Cycle edges and self-loops also receive the clearance: their single rank-gap entry is registered with the `ToEndpoint` kind (see [Gap-based protrusion for cycle edges](#gap-based-protrusion-for-cycle-edges-fn-cycle_edge_collect_rank_gap_entries)), and the unregistered fallback in `protrusions_assign_cycle_edges` stacks depths from `TO_PROTRUSION_MIN_PX` upward.


### Helper functions

33. **`rank_gap_px`**: computes the pixel distance in the rank direction for one non-cycle endpoint. For the from-endpoint, this is the distance from the from-node's face center to the first spacer entry (or to-node if no spacers). For the to-endpoint, it is the distance from the to-node's face center to the last spacer exit (or from-node if no spacers). The result is then **capped at the boundary of the other node's divergent ancestor**: for a Bottom-face protrusion the cap is the distance from the from-node's face to the Top face of the to-node's divergent ancestor (i.e. the near boundary of the destination container). This prevents the per-side band from exceeding the actual inter-rank gap when nodes are deeply nested, which would otherwise cause both protrusion tips to land at the same coordinate and suppress the Z/S bend. Cycle edge endpoints use a different computation in `cycle_edge_collect_rank_gap_entries` (see below).
34. **`spacer_gap_px`**: computes the pixel distance between two consecutive spacers along the rank axis (from the exit of one spacer to the entry of the next).
35. **`spacer_gap_key`**: computes the `RankGapKey` for the gap between two consecutive spacers by interpolating ranks between the from-node and to-node.
36. **`face_offset_resolve`**: resolves the face offset (slot offset) for a single endpoint from `face_offsets_by_node_face`. Spacer endpoints have a face offset of `0.0`.
37. **`cross_axis_coord`**: returns the cross-axis coordinate: X for `Top`/`Bottom` faces, Y for `Left`/`Right` faces.
38. **`axis_distance`**: computes the absolute distance along the rank axis between two points: `|by - ay|` for `Top`/`Bottom` faces, `|bx - ax|` for `Left`/`Right` faces.
39. **`protrusion_write`**: writes the computed protrusion depth into the correct slot in the output (`from_protrusion`, `to_protrusion`, or `spacer_protrusions[i].entry_protrusion` / `exit_protrusion`), dispatching on `RankGapEndpointKind`.
40. **`node_category`**: maps a node's entity types to a [`NodeCategory`](#concepts) variant (`Thing`, `Tag`, `Process`, or `Other`). Used in `cycle_edge_collect_rank_gap_entries` and `min_protrusion_divergent_sibling_extent` to filter out unrelated node types from rank-gap and sibling-extent calculations.


### How protrusions are consumed

40. The `OrthoProtrusionParams` are passed to `EdgePathBuilderPass2Ortho::build_spacer_edge_path` (in [`edge_path_builder_pass_2_ortho.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/edge_path_builder_pass_2/edge_path_builder_pass_2_ortho.rs)), which constructs the final SVG path. The path is built in reverse (end to start) as a sequence of waypoints. Each waypoint has a coordinate and a direction. Between consecutive waypoints, a multi-leg orthogonal segment with arc-rounded corners (using `ARC_RADIUS = 4.0` px and `KAPPA` for cubic bezier approximation) is drawn.
41. At each node endpoint, if the protrusion is non-zero, an extra waypoint is added at `(contact_x + face_outward_dx * protrusion, contact_y + face_outward_dy * protrusion)`, extending the path perpendicular to the face before the main routing segment begins.
42. At each spacer, four waypoints are added: the exit + exit protrusion, the exit, the entry, and the entry - entry protrusion. The protrusion waypoints push the routing legs away from node faces so that parallel edges sharing the same gap run at distinct depths.

### Protrusion-tip crossing guard (`fn from_protrusion_capped`)

43. For edges between deeply-nested nodes the band-assigned `from_protrusion` can exceed the available gap between the two outer containers. When the divergent-sibling adjustment (Step 5) simultaneously raises `to_protrusion` to clear the destination container hierarchy, the sum `from_protrusion + to_protrusion` can exceed the full node-to-node gap. This places the from-tip *past* the to-tip in the routing direction -- the tips cross -- so any Z/S segment drawn between them would re-enter the destination container.

    Note: within a single rank gap the per-side proportional split (Step 31) already guarantees `from_protrusion + to_protrusion <= MAX_GAP_FRACTION * gap`, and the `rank_gap_px` cap (see item 33) limits each side's band to the actual inter-rank gap. The `from_protrusion_capped` guard is a rarely-hit secondary safety net for remaining edge cases (e.g. very small rank gaps, the divergent-sibling adjustment raising `to_protrusion` after band assignment, or edges where the cap does not fully constrain the protrusion).

    The fix is applied in `build_ortho_edge_path` via a helper `fn from_protrusion_capped`. For aligned opposite-face pairs (Bottom-Top, Top-Bottom, Left-Right, Right-Left), it computes `gap = |end_axis - start_axis|` and, when `from_protrusion + to_protrusion > gap`, caps the from-protrusion to `(gap - to_protrusion).max(0.0)`. For other face-pair combinations (cycle edges, L-shaped routing) it returns the from-protrusion unchanged.

    After the cap, both protrusion tips meet at the same axis coordinate. The V-spike guard in `connect_waypoints` (see item 45) then replaces the Z/S U-bend between the tips with a straight horizontal line -- no direction reversal occurs at the meeting point.

### Same-axis collinear check in `connect_waypoints`

44. `connect_waypoints` uses a dot-product check to detect when two consecutive waypoints are collinear with the departure direction and should be joined by a straight line rather than a Z/S or L-shaped bend. The original check was `dot_p.abs() > 0.95`, which accepted both collinear (`dot_p > 0.95`) and *anti*-collinear (`dot_p < -0.95`) cases.

    The anti-collinear case includes two fundamentally different situations:

    - **Same-axis return** -- the return leg from a protrusion tip back to the node contact point. Both points have the same x-coordinate (for a vertical face) or same y-coordinate (for a horizontal face). The displacement is purely backward (dot_p = -1), and a straight line is correct.

    - **Anti-collinear with perpendicular offset** -- the protrusion tips cross (see above) or the path connects two tips at different positions with a large backward component. `dot_p` approaches -1 when the perpendicular offset is small relative to the backward displacement. A straight diagonal line is *incorrect* here; an orthogonal Z/S bend is required.

    The fix replaces `dot_p.abs() > 0.95` with `dot_p > 0.95 || is_same_axis`, where `is_same_axis` is `true` when both waypoints share the same routing axis (no perpendicular component): `|dx| < 1e-3` for a vertical departure, `|dy| < 1e-3` for a horizontal departure. This correctly draws straight lines for same-axis returns while routing the anti-collinear-with-offset case through the Z/S logic.

### V-spike guard for opposite-direction tips at the same axis coordinate

45. After `from_protrusion_capped` lands both protrusion tips at the same Y (or X for horizontal routing), the two tip waypoints have **opposite** departure directions. Since the path runs from-node to to-node, the segment is traversed from the from-tip to the to-tip: the from-tip departs downward (`Bottom` face, `dir = (0,+1)`) and the to-tip departs upward (`Top` face, `dir = (0,-1)`). The standard Z/S U-bend logic would route:

    1. **Leg 1** -- downward from the from-tip `p = (px, py)` to a bend at `(px, py + ARC_RADIUS)`.
    2. **Leg 2** -- horizontally across to `(qx, py + ARC_RADIUS)`.
    3. **Leg 3** -- upward from the bend to the to-tip `q = (qx, qy)` (same Y as p).

    But the continuation from q (the `is_same_axis` return leg toward the to-contact point) immediately travels **upward** again past q, reversing direction. The resulting up-then-down **V-spike** at q is visually incoherent.

    The fix is in `connect_waypoints`: at the start of the vertical Z/S branch, before computing the bend point, a guard fires when `|py - qy| < 1e-3` **and** `p_dy * q_dy < 0` (opposite vertical directions). In that case a **straight horizontal line** is drawn from p to q and the function returns early. The horizontal analogue applies for the horizontal Z/S branch (`|px - qx| < 1e-3`, `p_dx * q_dx < 0`).

    This guard is rarely triggered after the `rank_gap_px` cap fix, since the cap prevents both protrusion tips from landing at the same coordinate in normal routing. It remains as a safety net for degenerate layouts.


## Spacer Coordinate Direction Awareness

43. Spacer nodes are 5x5 px taffy leaf nodes inserted at intermediate ranks. After taffy computes the layout, each spacer's absolute position is resolved into a `SpacerCoordinates { entry_x, entry_y, exit_x, exit_y }`, representing the entry and exit points that the edge path passes through.
44. The entry and exit points of a spacer depend on the diagram's `RankDir`. This is implemented in `fn calculate` in [`edge_spacer_coordinates_calculator.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/edge_spacer_coordinates_calculator.rs):
    - `TopToBottom` -- entry at the top midpoint (smallest y), exit at the bottom midpoint (largest y). The path passes vertically downward through the spacer.
    - `BottomToTop` -- entry at the bottom midpoint (largest y), exit at the top midpoint (smallest y). The path passes vertically upward through the spacer.
    - `LeftToRight` -- entry at the left midpoint (smallest x), exit at the right midpoint (largest x). The path passes horizontally rightward through the spacer.
    - `RightToLeft` -- entry at the right midpoint (largest x), exit at the left midpoint (smallest x). The path passes horizontally leftward through the spacer.
45. When cross-container spacers are merged with rank-based spacers, they are sorted by the main-axis coordinate (`entry_y` for vertical flows, `entry_x` for horizontal flows) so that the spacers appear in the correct visual order along the edge path. This sorting is implemented in `fn spacer_coordinates_from_spacers` in [`svg_edge_infos_builder.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/svg_edge_infos_builder.rs) and `fn spacer_coordinates_resolve` in [`ortho_protrusion_calculator.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/ortho_protrusion_calculator.rs).


## Cycle Edge Routing

46. Edges between nodes at the **same `NodeRank`** (cycle edges) need special treatment for two reasons. First, they need clockwise face selection to route around the outside of nodes (rather than connecting nearest faces, which would route through nodes). Second, their protrusions must be distributed using the adjacent rank gap's available space, so that multiple cycle edges sharing the same gap get distinct protrusion depths instead of all collapsing to the same fixed minimum. Without special handling the Z/S routing bend falls exactly at the node face boundary and the segment overlaps the node.
47. Same-rank edges are detected in `build_edge_pass1_infos` in [`svg_edge_infos_builder.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/svg_edge_infos_builder.rs) by comparing the ranks of the `from` and `to` nodes at their **LCA (Lowest Common Ancestor) level** before face selection. Using local context ranks (each node's rank within its own parent container) would give false positives for cross-container edges: two nodes in different containers can both have rank 0 in their respective parent contexts while sitting at visually different positions in the diagram. The LCA-level ranks avoid this by comparing the ranks of the *divergent ancestors* -- the direct children of the LCA that are ancestors of (or equal to) each node. When `rank_from == rank_to` at the LCA level, the `is_same_rank` flag is set to `true` and passed to `faces_select`. The `is_cycle_edge` flag is set to `true` only when all of the following conditions hold:
    - `rank_from == rank_to` at the LCA level (same visual rank),
    - the two nodes are **not** adjacent siblings (nesting-path index difference > 1).


### Self-loop routing

Self-loop edges (`from == to`) are a degenerate cycle edge: both endpoints are
at the same rank, and both checks in the `is_cycle_edge` condition return
`false` for identical nodes, so `is_cycle_edge` is `true`. They are routed as
follows:

- **Face selection**: both contacts sit on the **rank-direction face** -- the
  face a forward edge would exit (`Bottom` for `TopToBottom`, `Top` for
  `BottomToTop`, `Right` for `LeftToRight`, `Left` for `RightToLeft`),
  resolved by `EdgePathBuilderPass1::self_loop_face` /
  `EdgeFaceAssigner::forward_faces`. The IR-level `EdgeFaceAssignment` stores
  only `from_face` (one label slot); pass 1 duplicates the face into both
  `from_face` and `to_face` so the offset and protrusion machinery sees two
  contacts on the same face.
- **Offsets**: both contacts register with `EdgeFaceContactTracker`, so they
  receive distinct slot offsets (or a label-aligned offset for the from
  contact when a description label is present).
- **Protrusions**: self-loops flow through the cycle-edge protrusion path
  (`cycle_edge_collect_rank_gap_entries` for `Top`/`Bottom` faces with an
  adjacent rank, otherwise Case B of `protrusions_assign_cycle_edges`), giving
  both contacts the same depth.
- **Path**: `EdgeCurvature::Orthogonal` produces a U-shape via the standard
  waypoint machinery (same-face pair, equal protrusion tips bridged by the
  same-coordinate Z/S rule). `EdgeCurvature::Curved` uses
  `EdgePathBuilderPass1::self_loop_path_build`, a bezier loop generalised over
  the four faces.

### Clockwise face selection (`fn cycle_edge_faces_select`)

48. When `is_same_rank` is `true`, `faces_select` delegates to `cycle_edge_faces_select` in [`edge_path_builder_pass_1.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/edge_path_builder_pass_1.rs). This function returns a pair of node faces that routes the edge **clockwise around the outside** of the involved nodes.
49. The selection is purely geometric and does not depend on the `RankDir`:

    | Relative position of `from` vs `to` | `from` face | `to` face | Routing path |
    |--------------------------------------|-------------|-----------|----------------------------------------------|
    | `from` left of `to` (`dx > 0`)       | `Top`       | `Top`     | Protrude up, arc right above, enter top |
    | `from` right of `to` (`dx < 0`)      | `Bottom`    | `Bottom`  | Protrude down, arc left below, enter bottom |
    | `from` above `to` (`|dy| > |dx|`, `dy > 0`) | `Right` | `Right` | Protrude right, arc down right side, enter right |
    | `from` below `to` (`|dy| > |dx|`, `dy < 0`) | `Left`  | `Left`  | Protrude left, arc up left side, enter left |

    The tie-breaking condition `dx.abs() >= dy.abs()` means that when the horizontal displacement equals the vertical displacement, the horizontal rule applies.


### Gap-based protrusion for cycle edges (`fn cycle_edge_collect_rank_gap_entries`)

50. For cycle edges with `Top` or `Bottom` faces, a **single entry** is registered in the adjacent rank gap in Step 2 of `calculate`. This lets the edge compete for protrusion slots alongside non-cycle edges in the same gap. The entry uses the **`ToEndpoint` kind** so the arrow-head clearance floor in `protrusion_write` applies (cycle edges and self-loops also carry an arrow head at their to-endpoint); its sorting keys (face offset, cross-axis coordinate) are taken from the from side, which is equivalent because both endpoints share the same face and rank. Step 6 (`fn protrusions_assign_cycle_edges`) then copies the assigned depth to the from-endpoint, producing a symmetric U-shaped arc.

    - **`rank_gap_px` for cycle edges**: the pixel distance is computed directly from layout coordinates, not from the distance to the other endpoint. The **adjacent rank boundary** is found by iterating over all nodes at the adjacent rank within the same scope (`node_ranks_nested.ranks_for(parent_container)`), then taking:
      - For `Top` face: the **maximum** `y + height_collapsed` (bottom edge) of adjacent rank-R-1 nodes.
      - For `Bottom` face: the **minimum** `y` (top edge) of adjacent rank-R+1 nodes.

      Then `rank_gap_px = node.y - adjacent_boundary` (Top) or `adjacent_boundary - (node.y + node.height_collapsed)` (Bottom). If no adjacent-rank nodes exist, or the computed gap is non-positive, the endpoint is not registered (falls through to Step 6).

    - **Gap side**: the cycle edge's entry is on the `High` side for `Top` face, or `Low` side for `Bottom` face. This makes it a `single_side` entry in `protrusions_assign`, receiving a unique slot.

    - **Sharing the gap with non-cycle edges**: if non-cycle edges also have endpoints in the same gap (e.g. an edge from rank R-1 to rank R contributes its to-endpoint on the `High` side of gap (R-1, R)), all entries compete together for the gap's protrusion band. The tightest `rank_gap_px` across all entries sizes the band via `MAX_GAP_FRACTION = 0.8`, which is then split proportionally between the two sides (see [Protrusion depth assignment](#protrusion-depth-assignment-fn-protrusions_assign)). This ensures the from-side and to-side fans together never exceed 80% of the actual gap, leaving room for the routing segment and arrowhead.

### Cycle edge protrusion finalisation (`fn protrusions_assign_cycle_edges`)

51. After gap-based assignment (Steps 2–5), Step 6 calls `protrusions_assign_cycle_edges` which handles two cases:

    **Case A -- registered / adjusted cycle edges** (`to_protrusion > 0` or `from_protrusion > 0`): the edge was assigned a depth by the gap-based step (written to `to_protrusion`, already floored for arrow-head clearance by `protrusion_write`) and/or raised by the divergent-sibling adjustment (Step 5). Both endpoints are equalized at the larger depth (with `MIN_PROTRUSION_PX` as a floor), producing a symmetric U-shaped arc.

    **Case B -- unregistered cycle edges** (both protrusions zero): edges that returned early from `cycle_edge_collect_rank_gap_entries` (boundary ranks, no adjacent rank nodes, or `Left`/`Right` faces). These are grouped by `(from_face, rank_from)` -- all edges routing in the same direction at the same rank -- then sorted by face offset then cross-axis coordinate (matching the ordering in `protrusions_assign`). Within each group of N edges, depths are stacked from the arrow-head clearance floor:

    - `slot[N-1]` (last sorted entry) -> `TO_PROTRUSION_MIN_PX` (shortest, innermost arc)
    - `slot[0]` (first sorted entry) -> `TO_PROTRUSION_MIN_PX + (N-1) × MIN_PROTRUSION_PX` (longest, outermost arc)
    - Adjacent slots are `MIN_PROTRUSION_PX` apart

    Both `from_protrusion` and `to_protrusion` are set to the same assigned depth. Unregistered cycle edges protrude into open space (boundary ranks or `Left`/`Right` faces), so no gap cap applies.

52. The stacking order -- first-sorted entry gets the longest protrusion -- mirrors the slot assignment in `protrusions_assign` for single-side entries. This matches the face-offset + cross-axis sorting used for the adjacent-rank case, so the visual layering is consistent whether or not an adjacent rank exists.


### Z/S bend direction for same-coordinate protrusion tips

53. The `connect_waypoints` function in [`edge_path_builder_pass_2_ortho.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/edge_path_builder_pass_2/edge_path_builder_pass_2_ortho.rs) connects two consecutive waypoints with a Z/S-shaped three-leg segment when both waypoints have the same axis orientation (both vertical or both horizontal). The bend point is determined by a sign computed from the relative positions of the two waypoints.
54. For cycle edges using `Top`/`Top` or `Bottom`/`Bottom` faces, both protrusion tips end up at the same Y coordinate (because both nodes are at the same `y` in the typical same-rank layout). The standard heuristic (`sign = if py < qy { -1 } else { 1 }`) would pick `sign = 1` when `py == qy`, placing the bend below the protrusion tips and **inside** the node bounding boxes.
55. To fix this, when the two waypoints are at the **same coordinate on the routing axis** (`|py - qy| < 1e-3` for vertical, `|px - qx| < 1e-3` for horizontal), the bend direction is determined from the **departure direction** (`p_dir`) of the source waypoint instead:
    - Vertical Z/S: if `p_dy < 0.0` (face points upward, e.g. `Top`) then `sign = -1` (bend upward); if `p_dy > 0.0` (face points downward, e.g. `Bottom`) then `sign = +1` (bend downward).
    - Horizontal Z/S: if `p_dx < 0.0` (face points leftward, e.g. `Left`) then `sign = -1` (bend leftward); if `p_dx > 0.0` (face points rightward, e.g. `Right`) then `sign = +1` (bend rightward).

    This ensures the U-shaped bend of the routing segment is placed entirely outside the node bounding boxes.

### Small-gap guard for Z/S bends

56. A second edge case arises when the gap between the two protrusion tips is **smaller than `ARC_RADIUS`**. The standard formula `bend = qy + sign * ARC_RADIUS` (for vertical) or `bend = qx + sign * ARC_RADIUS` (for horizontal) can then place the bend **past `p`** in the direction opposite to `p`'s departure, making Leg 1 travel against the departure direction.

    **Example**: in a `TopToBottom` diagram with an edge from a nested node `alice` (inside `alice_outer`) to another nested node `charlie_1` (inside `charlie_outer`), the path runs from-node to to-node, so the segment is traversed from the from-tip to the to-tip. The from-protrusion tip `p = (58, 152.696)` is 36.7 px below alice's bottom face and the to-protrusion tip `q = (70, 155)` is at the top of `charlie_outer`. The gap `qy - py = 2.304 < ARC_RADIUS = 4.0`, so with `sign = -1` the formula gives `bend_y = qy - ARC_RADIUS = 151 < py = 152.696`. Leg 1 then goes **upward** from `p` (back toward alice) even though `p.dir = (0, +1)` says the path should depart **downward**.

57. There is a second failure mode: placing the bend **below both tips** (e.g. `bend_y = max(py, qy) + ARC_RADIUS = 159`) fixes Leg 1 (which now travels downward from `p`), but makes Leg 3 travel **upward** from the bend to `q`. Since the next path segment continues downward from `q` into the to-node, this creates a sharp direction reversal (V-spike) at `q`. In the visual arrow direction the edge loops backward -- going downward past `q` before returning upward to it.

58. The guard after the sign/bend computation detects the Leg-1 failure and recomputes the bend. For the **typical case** where `p` and `q` are on opposite sides of each other in the departure direction (e.g. `py < qy` for a downward-departing `p`, as in the from-tip to to-tip example above), the bend is reset to the **midpoint** `(py + qy) / 2`. This places the bend strictly inside the routing gap between the two containers, so both Leg 1 and Leg 3 travel in the correct direction and no backward loop appears:

    - Vertical, downward departure (`p_dy > 0.0`), `bend_y <= py` **and** `py < qy`: reset `bend_y = (py + qy) / 2`.
    - Vertical, upward departure (`p_dy < 0.0`), `bend_y >= py` **and** `py > qy`: reset `bend_y = (py + qy) / 2`.
    - Horizontal: symmetric conditions on `p_dx`, `bend_x`, `px`, and `qx`.

    For the **unusual case** where `p` and `q` are on the same side (e.g. `py >= qy` for a downward-departing `p`, which does not arise in normal `TopToBottom` routing), the bend is placed `ARC_RADIUS` beyond `p` in its departure direction so Leg 1 is still correct.
