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


## Offset Calculation

Offset calculation spreads multiple edge contact points along a node face so they do not all touch at the same coordinate. The offset for each edge is a signed pixel distance from the face midpoint. Offsets are computed in `fn face_offsets_compute` in [`svg_edge_infos_builder.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/svg_edge_infos_builder.rs), using [`edge_face_contact_tracker.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/edge_face_contact_tracker.rs) for the per-face arithmetic.


### Why offsets exist

When multiple edges connect to the same face of a node (e.g. three edges all exit from the Bottom face), their paths would all originate from the exact same point -- the face midpoint. Offsets shift each edge's contact point along the face so the paths fan out, making individual edges visually distinguishable.


### Concepts

- **Face contact**: a single edge endpoint touching a specific face of a specific node. Each edge contributes up to two contacts: one for its from-node face and one for its to-node face.
- **Slot**: an ordered position within the list of contacts on a single (node, face) pair. Slot 0 is the first sorted contact, slot 1 the second, etc.
- **Face length**: the pixel length of the face: width for `Top`/`Bottom` faces, collapsed height for `Left`/`Right` faces. Computed by `fn face_length_for_node`.
- **Gap**: the pixel spacing between adjacent contact points. Starts at 10% of the face length (`CONTACT_GAP_RATIO = 0.10`), clamped to at least 5 px (`CONTACT_GAP_MIN_PX = 5.0`). If `contact_count * gap > face_length`, the gap is shrunk to `face_length / contact_count` so all contacts fit.


### Algorithm

The algorithm in `face_offsets_compute` proceeds in three phases:

1. **Collect face contact entries.** For each edge in every group, if the from-face or to-face is known, a `FaceContactEntry` is recorded in a map keyed by `(NodeId, NodeFace)`. Each entry captures the `rank_distance` (absolute rank difference between from and to), the target node's x/y coordinates, and the edge's group/index.

2. **Sort entries and assign slot indices (`fn face_entries_sort_by_rank_and_coordinate`).** For each (node, face) group, entries are sorted by:
    - **Primary key**: `rank_distance` ascending. Edges spanning fewer ranks get slots closer to the face midpoint, keeping short-range paths on the inside.
    - **Secondary key** (tie-breaker) -- the target node's coordinate along the face axis: x ascending for `Top`/`Bottom` faces, y ascending for `Left`/`Right` faces. This ensures co-ranked edges follow the spatial order of their targets.
    - After sorting, each entry is assigned its slot index (0-based position in the sorted order). These slot indices are written back into `from_slot_indices` / `to_slot_indices` on the pass-1 group.

3. **Compute offset values (`EdgeFaceContactTracker::offset_for_index`).** For each (node, face) with `n` contacts and computed `gap`:
    - `offset[i] = (i - (n - 1) / 2.0) * gap`
    - This distributes offsets symmetrically around 0 (the face midpoint). The first slot gets the most negative offset (leftward/upward), the middle slot(s) get ~0, and the last slot gets the most positive offset (rightward/downward).


### Direction reversal for `BottomToTop` and `RightToLeft`

For `TopToBottom` and `LeftToRight`, the offset sign convention (negative = left/up, positive = right/down) naturally matches the visual flow -- edges to left-side targets get negative offsets (shifting contact leftward), reducing crossover. For `BottomToTop` and `RightToLeft`, the visual flow is reversed, so the same sort order would produce offsets that *increase* crossover. To fix this, all computed offsets are **negated** when `rank_dir` is `BottomToTop` or `RightToLeft`. This is implemented in `face_offsets_compute` after `offset_calculate` returns each value.


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


### Protrusion depth assignment (`fn protrusions_assign`)

This function assigns protrusion depths to all endpoints within a single rank gap:

27. **Find the tightest constraint.**

    The minimum `rank_gap_px` across all entries in the gap determines the available space. The maximum allowed protrusion is `min_gap_px * MAX_GAP_FRACTION` (where `MAX_GAP_FRACTION = 0.48`, leaving at least 4% of the gap for the horizontal/vertical routing segment between the two sides).

28. **Small gap fallback.**

    If the maximum protrusion is less than `MIN_PROTRUSION_PX` (3.0 px), all entries receive a minimal protrusion of `min(MIN_PROTRUSION_PX, min_gap_px * 0.5)`. If there is exactly one entry, it receives `max(max_protrusion * 0.5, MIN_PROTRUSION_PX)`.

29. **Partition by gap side and sort.**

    Entries are partitioned into `Low` and `High` groups. Each group is sorted by face offset (ascending), then cross-axis coordinate. This spatial ordering ensures that edges whose contact points are further apart receive longer protrusions, and edges closer together receive shorter ones, reducing visual cross-over.

30. **Identify crossing edges.**

    Edges that appear on both sides of the gap are identified. Their protrusion depths on each side are assigned independently based on that side's spatial ordering, so the high-side ordering is not dictated by the low-side sort.

31. **Distribute protrusion slots.**

    All `N` entries share a pool of `N` evenly-spaced protrusion depths in `[MIN_PROTRUSION_PX, max_protrusion]`:
    - `slot[k] = max_protrusion - k * (max_protrusion - MIN_PROTRUSION_PX) / (N - 1)`
    - Slot 0 gets the longest protrusion; slot N-1 gets the shortest.

32. **Slot assignment order.**

    Slots are assigned in four groups:

    1. Single-side low entries (only on `Low` side) -- get the first slots (longest protrusions).
    2. Crossing low entries (from-endpoints of crossing edges) -- get the next slots, in low-side spatial order.
    3. Crossing high entries (to-endpoints of crossing edges) -- get slots in forward order within their range, sorted by high-side spatial order. This ensures earlier edges on the high side also receive longer protrusions.
    4. Single-side high entries (only on `High` side) -- fill the remaining slots (shortest protrusions).


### Helper functions

33. **`rank_gap_px`**: computes the pixel distance in the rank direction for one non-cycle endpoint. For the from-endpoint, this is the distance from the from-node's face center to the first spacer entry (or to-node if no spacers). For the to-endpoint, it is the distance from the to-node's face center to the last spacer exit (or from-node if no spacers). Cycle edge endpoints use a different computation in `cycle_edge_collect_rank_gap_entries` (see below).
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

50. For cycle edges with `Top` or `Bottom` faces, only the **from-endpoint** is registered in the adjacent rank gap in Step 2 of `calculate` (not the to-endpoint). This lets it compete for protrusion slots alongside non-cycle edges in the same gap. Step 6 (`fn protrusions_assign_cycle_edges`) then copies the assigned depth to the to-endpoint, producing a symmetric U-shaped arc.

    - **`rank_gap_px` for cycle edges**: the pixel distance is computed directly from layout coordinates, not from the distance to the other endpoint. The **adjacent rank boundary** is found by iterating over all nodes at the adjacent rank within the same scope (`node_ranks_nested.ranks_for(parent_container)`), then taking:
      - For `Top` face: the **maximum** `y + height_collapsed` (bottom edge) of adjacent rank-R-1 nodes.
      - For `Bottom` face: the **minimum** `y` (top edge) of adjacent rank-R+1 nodes.

      Then `rank_gap_px = node.y - adjacent_boundary` (Top) or `adjacent_boundary - (node.y + node.height_collapsed)` (Bottom). If no adjacent-rank nodes exist, or the computed gap is non-positive, the endpoint is not registered (falls through to Step 6).

    - **Gap side**: the from-endpoint of a cycle edge is on the `High` side for `Top` face, or `Low` side for `Bottom` face. This makes it a `single_side` entry in `protrusions_assign`, receiving a unique slot.

    - **Sharing the gap with non-cycle edges**: if non-cycle edges also have endpoints in the same gap (e.g. an edge from rank R-1 to rank R contributes its to-endpoint on the `High` side of gap (R-1, R)), all entries compete together for slots. The tightest `rank_gap_px` across all entries determines the maximum protrusion via `MAX_GAP_FRACTION = 0.48`. This ensures protrusions never exceed 48% of the actual gap, leaving room for the routing segment, arrowhead, and other entries on the opposite side.

### Cycle edge protrusion finalisation (`fn protrusions_assign_cycle_edges`)

51. After gap-based assignment (Steps 2–5), Step 6 calls `protrusions_assign_cycle_edges` which handles two cases:

    **Case A -- registered cycle edges** (`from_protrusion > 0`): the from-endpoint was assigned a depth by the gap-based step. This depth is copied to `to_protrusion` (with `MIN_PROTRUSION_PX` as a floor) so both endpoints protrude equally, producing a symmetric U-shaped arc.

    **Case B -- unregistered cycle edges** (`from_protrusion == 0`): edges that returned early from `cycle_edge_collect_rank_gap_entries` (boundary ranks, no adjacent rank nodes, or `Left`/`Right` faces). These are grouped by `(from_face, rank_from)` -- all edges routing in the same direction at the same rank -- then sorted by face offset then cross-axis coordinate (matching the ordering in `protrusions_assign`). Within each group of N edges, depths are assigned:

    - `slot[0]` (first sorted entry) -> `N × MIN_PROTRUSION_PX` (longest, outermost arc)
    - `slot[N-1]` (last sorted entry) -> `1 × MIN_PROTRUSION_PX` (shortest, innermost arc)
    - Intermediate slots evenly spaced between `N × MIN` and `1 × MIN`

    Both `from_protrusion` and `to_protrusion` are set to the same assigned depth.

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

    **Example**: in a `TopToBottom` diagram with an edge from a nested node `alice` (inside `alice_outer`) to another nested node `charlie_1` (inside `charlie_outer`), the to-protrusion tip `p = (70, 155)` is at the top of `charlie_outer` and the from-protrusion tip `q = (58, 152.696)` is 36.7 px below alice's bottom face. The gap `py - qy = 2.304 < ARC_RADIUS = 4.0`, so with `sign = +1` the formula gives `bend_y = qy + ARC_RADIUS = 156.696 > py = 155`. Leg 1 then goes **downward** from `p` (into `charlie_outer`) even though `p.dir = (0, -1)` says the path should depart **upward**.

57. There is a second failure mode: placing the bend **above both tips** (e.g. `bend_y = min(py, qy) - ARC_RADIUS = 148.696`) fixes Leg 1 (which now travels upward from `p`), but makes Leg 3 travel **downward** from the bend to `q`. Since the next path segment continues upward from `q` toward the from-node, this creates a sharp direction reversal (V-spike) at `q`. In the visual arrow direction the edge loops backward -- going upward past `q` before returning downward to `p`.

58. The guard after the sign/bend computation detects the Leg-1 failure and recomputes the bend. For the **typical case** where `p` and `q` are on opposite sides of each other in the departure direction (e.g. `py > qy` for an upward-departing `p`), the bend is reset to the **midpoint** `(py + qy) / 2`. This places the bend strictly inside the routing gap between the two containers, so both Leg 1 and Leg 3 travel in the correct direction and no backward loop appears:

    - Vertical, upward departure (`p_dy < 0.0`), `bend_y >= py` **and** `py > qy`: reset `bend_y = (py + qy) / 2`.
    - Vertical, downward departure (`p_dy > 0.0`), `bend_y <= py` **and** `py < qy`: reset `bend_y = (py + qy) / 2`.
    - Horizontal: symmetric conditions on `p_dx`, `bend_x`, `px`, and `qx`.

    For the **unusual case** where `p` and `q` are on the same side (e.g. `py <= qy` for an upward-departing `p`, which does not arise in normal `TopToBottom` routing), the bend is placed `ARC_RADIUS` beyond `p` in its departure direction so Leg 1 is still correct.
