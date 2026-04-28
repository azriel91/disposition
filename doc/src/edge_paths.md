# Edge Paths

1. Nodes are connected to other nodes via edges
2. Nodes are laid out in a flex layout with recursive flex layout containers
3. Between nodes, "spacer nodes" may be inserted, which serve as coordinate markers for edge paths, so that when edge paths are calculated, the path is routed through spacer nodes to avoid drawing lines over the diagram nodes.
4. Nodes also have a `NodeRank`, which is "the highest rank of nodes connected to this node, plus one". If there are no nodes connected to this node, the `NodeRank` is `0`.
5. Part of the information gathered for calculating spacer nodes is collecting a `BTreeMap<NodeRank, Vec<taffy::NodeId>>`.
6. Calculation of where to place spacer nodes is done in [`ir_to_taffy_builder.rs`](crate/input_ir_rt/src/ir_to_taffy_builder.rs), in `fn build_taffy_child_nodes_for_node_by_rank`, called by `fn build_taffy_nodes_for_node_with_child_hierarchy`.
7. Edge path calculation is done in two passes.
8. Both passes are called in [`svg_edge_infos_builder.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/svg_edge_infos_builder.rs)
9. The first pass calculates a path between the from-node and to-node without taking into account spacer nodes, and the information from this first path is used in subsequent calculations. This is defined in [`edge_path_builder_pass_1.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/edge_path_builder_pass_1.rs)
10. Between the first and second pass, offsets from where the edge path exits the from-node, and where it enters the to-node, are computed, so that multiple edges do not all touch the from-node / to-node at the same coordinate, for visual clarity. See the [Offset Calculation](#offset-calculation) section below for details.
11. Also, for orthogonal (strictly horizontal / vertical) edge paths, a "protrusion" length is calculated so that the paths exit the node perpendicular to the node face for some length, so that the path is not drawn directly on the node face as a tangential line. See the [Protrusion Calculation](#protrusion-calculation) section below for details.
12. Offsets are the coordinate shift to where the edge path contacts the node face.
13. Protrusion is the length that the edge path extends out of the node face, so that an edge path isn't drawn directly on a node face.
14. The second pass is defined in [`edge_path_builder_pass_2.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/edge_path_builder_pass_2.rs)
15. The second pass computes the edge paths with the offsets and protrusion, which should result in paths that are visually non-overlapping with other paths and node content, creating visual clarity.


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


### Algorithm overview (`fn calculate`)

The algorithm in `OrthoProtrusionCalculator::calculate` has four steps:

23. **Step 1: Resolve spacer coordinates and initialize output.**

    Spacer coordinates are resolved once per edge (via `spacer_coordinates_resolve`). The output `Vec<Vec<OrthoProtrusionParams>>` is initialized with all protrusions set to `0.0` and `spacer_protrusions` sized to match the resolved spacer count.

24. **Step 2: Collect rank gap entries.**

    For each edge across all groups:

    - The from-endpoint is registered in the rank gap between the from-node's rank and the adjacent rank toward the to-node.
    - The to-endpoint is registered in the rank gap between the to-node's rank and the adjacent rank toward the from-node.
    - Each intermediate spacer contributes two entries: its entry side protrudes into the gap before it, and its exit side protrudes into the gap after it. The first spacer's entry shares the same gap as the from-endpoint (opposite side), and the last spacer's exit shares the same gap as the to-endpoint (opposite side).
    - Each entry records its `GapSide`, cross-axis coordinate, face offset, and rank gap pixel distance (computed by `rank_gap_px` for node endpoints or `spacer_gap_px` for spacer-to-spacer gaps).

25. **Step 3: Assign protrusion depths (`fn protrusions_assign`).**

    For each rank gap, all collected entries are assigned distinct protrusion depths. See [Protrusion depth assignment](#protrusion-depth-assignment) below.

26. **Step 4: Propagate node protrusions to shared spacer sides (fallback).**

    If the first spacer's entry protrusion or the last spacer's exit protrusion was not assigned (because the node face was `None`), it falls back to the from/to protrusion value as a safety net.


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

33. **`rank_gap_px`**: computes the pixel distance in the rank direction for one endpoint. For the from-endpoint, this is the distance from the from-node's face center to the first spacer entry (or to-node if no spacers). For the to-endpoint, it is the distance from the to-node's face center to the last spacer exit (or from-node if no spacers).
34. **`spacer_gap_px`**: computes the pixel distance between two consecutive spacers along the rank axis (from the exit of one spacer to the entry of the next).
35. **`spacer_gap_key`**: computes the `RankGapKey` for the gap between two consecutive spacers by interpolating ranks between the from-node and to-node.
36. **`face_offset_resolve`**: resolves the face offset (slot offset) for a single endpoint from `face_offsets_by_node_face`. Spacer endpoints have a face offset of `0.0`.
37. **`cross_axis_coord`**: returns the cross-axis coordinate: X for `Top`/`Bottom` faces, Y for `Left`/`Right` faces.
38. **`axis_distance`**: computes the absolute distance along the rank axis between two points: `|by - ay|` for `Top`/`Bottom` faces, `|bx - ax|` for `Left`/`Right` faces.
39. **`protrusion_write`**: writes the computed protrusion depth into the correct slot in the output (`from_protrusion`, `to_protrusion`, or `spacer_protrusions[i].entry_protrusion` / `exit_protrusion`), dispatching on `RankGapEndpointKind`.


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
