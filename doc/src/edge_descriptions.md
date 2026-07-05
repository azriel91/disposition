# Edge Descriptions

Edge descriptions are text labels associated with edges in the diagram.  They
are specified via `edge_descs` in `InputDiagram`, keyed by the edge
**instance** ID (not the edge group ID).

> **Note:** `EdgeDescs` is **not** rendered through face-label slots.
> Description text is rendered via `edge_description_container` nodes
> interleaved between rank containers -- except for cycle edges (see
> [Same-Rank (Cycle Edge) Placement](#same-rank-cycle-edge-placement) below),
> whose container is inserted as a sibling *within* the shared rank instead.
> See `edge_description_containers_plan.md` for the implementation plan of
> that feature.

Face-label slots (documented below) are structural taffy leaf nodes placed at
the `from`/`to` node faces.  They exist purely for edge contact-point
positioning and face-offset calculations (see `edge_paths.md` -- Offset
Calculation).  They always measure as zero size and carry no rendered text.


## Text Measurement and Markdown Rendering

Edge description text measurement and rendering follows one of two paths
depending on the diagram level of detail:

### DiagramLod::Simple

At `DiagramLod::Simple`, each edge description is rendered as a single taffy
leaf node with `TaffyNodeCtx::EdgeDescription` context. The leaf is measured
using the description text as plain text (no markdown parsing), and spans are
computed by `HighlightedSpansComputer::compute_edge_desc_containers` after
layout.

### DiagramLod::Normal

At `DiagramLod::Normal`, the single description leaf is replaced by an
`md_content_node` sub-tree built via `MdNodeBuilder`. The markdown text is
parsed by `MdBlocksParser` into `MdBlock` structures, which are then converted
into a flex sub-tree with per-token and per-image leaves.

After layout, `MdSpansComputer::compute_edge_descs` processes these sub-trees
to merge adjacent word leaves on the same visual line into consolidated text
spans with markdown styling (bold, italic, code, headings, links) and converts
image leaves into `MdImageSpan` values.

The results are stored in `TaffyNodeMappings::edge_description_highlighted_spans`
and `TaffyNodeMappings::edge_description_image_spans`, then mapped to
`SvgTextSpan` and `SvgImageSpan` values by `SvgEdgeDescriptionsBuilder::build`
for final SVG rendering.


## Edge ID Format

Edge IDs are generated in the form:

```
{edge_group_id}__{edge_index}
```

For example, an edge group named `edge_dep` whose first (and only) edge is
index 0 has the edge ID `edge_dep__0`.

For a `symmetric` group named `edge_sym` between nodes `t_a` and `t_b` the two
edges are:

| Index | Direction | Edge ID       |
|-------|-----------|---------------|
| 0     | t_a -> t_b | `edge_sym__0` |
| 1     | t_b -> t_a | `edge_sym__1` |

## How to Add a Description

In your `InputDiagram` (YAML or Rust), add an entry to `edge_descs`:

```yaml
thing_dependencies:
  edge_dep:
    kind: sequence
    things:
      - t_a
      - t_b

edge_descs:
  edge_dep__0: "A depends on B"
```

The description text is rendered in an `edge_description_container` node
positioned between the rank containers of the edge's divergent ancestors.  See
`edge_description_containers_plan.md` for details, and
[Same-Rank (Cycle Edge) Placement](#same-rank-cycle-edge-placement) below for
the exception when the divergent ancestors share a rank.


## Same-Rank (Cycle Edge) Placement

When an edge's divergent ancestors share a rank -- a cycle edge, e.g. a
`cyclic` dependency group, or any edge (dependency or interaction) between two
nodes that a dependency cycle placed on the same rank -- there is no gap
*between* rank containers to interleave a container into: both ancestors live
in the same rank container's children.

For this case, `EdgeDescriptionBuilder::build` inserts the
`edge_description_container` as a direct child of the shared rank, at the
sibling index between the two divergent ancestors, rather than as a sibling of
rank containers. This mirrors how `EdgeSpacerBuilder` places same-level
cross-rank spacers (see [edge_spacers.md](edge_spacers.md) -- Same-Level
Cross-Rank Spacers): both use the shared
[`RankSiblingInserter`](crate/input_ir_rt/src/ir_to_taffy_builder/rank_sibling_inserter.rs)
helper to compute the sibling insertion index
(`(from_sibling_index + to_sibling_index) / 2 + 1`) and to insert at the
effective index, accounting for other insertions already made at that rank.

Multiple edges whose divergent ancestors are the *same* pair of same-ranked
siblings (e.g. a cyclic dependency plus a symmetric interaction group between
the same two nodes) share one container, grouped by `(rank,
sibling_index_middle)` so that a different pair of same-ranked siblings gets
its own container rather than being merged in.

Unlike a cross-rank `edge_description_container` (which mirrors
`rank_container_style.flex_direction` unchanged, since it is inserted *as a
sibling of* rank containers and multiple descriptions sharing that position
should lay out along the same axis rank siblings use), a same-rank
container's own children layout is *inverted*
(`EdgeDescriptionBuilder::container_style_build`, via
`taffy_container_builder::flex_direction_invert`): the container is inserted
*as a rank sibling itself*, directly between the two divergent ancestors,
which already occupy the rank's own stacking axis. Mirroring that axis for
multiple described edges sharing the slot would stack their boxes along the
same axis the two divergent ancestors sit on, widening (or heightening) the
gap between them per extra description. Since the divergent ancestors' own
edges run *along* that axis, the descriptions instead stack along the
perpendicular (cross) axis -- e.g. under `rank_dir: top_to_bottom`, two
described edges between the same pair of same-ranked (horizontally adjacent)
siblings stack vertically (`Column`) rather than widening the horizontal gap
between them.

This placement is scoped per LCA level exactly like same-level cross-rank
spacers: `EdgeDescriptionBuilder::build` is called once per level (root, and
once per container that is an LCA for at least one described edge), each with
its own independently-scoped `rank_to_taffy_ids`, so two cyclic pairs at
different nesting depths (e.g. a root-level cycle and, separately, a cycle
between two children of one of those root nodes) cannot collide.

The description's own rendered position is also a routing waypoint for its
owning edge's path: `SpacerCoordinatesResolver::description_contact_resolve`
reads the description leaf's post-layout rect and bends the edge's path to
touch it, applied unconditionally regardless of edge curvature (see
`edge_spacers.md` -- Edge Description Container Spacers). This mirrors how
`label_face_offset_compute` bends a path's face contact to sit beside an edge
label's own box. See [Description Contact Waypoint](#description-contact-waypoint)
below for how that waypoint is chosen -- it differs for same-rank vs
cross-rank edges.


## Description Contact Waypoint

`description_contact_resolve` branches on
`EdgeDescriptionTaffyNodes::is_cross_rank` (`true` for
`EdgeDescPosition::BetweenRanks`, `false` for `EdgeDescPosition::SameRank`).
In both cases, the description box sits directly *on* the connection between
the edge's two divergent ancestors -- between ranks for a cross-rank edge, or
directly between the two same-ranked siblings for a same-rank (cycle) edge --
so in both cases the path threads *through* the box (`entry != exit`), the
same way an ordinary spacer's corridor is threaded via
`EdgeSpacerCoordinatesCalculator::calculate`:

- **Cross-rank (`BetweenRanks`)**: threaded via `calculate_description_thread`
  -- see [Cross-Rank Contact](#cross-rank-contact) below.
- **Same-rank (cycle edges)**: threaded via
  `calculate_description_thread_same_rank`, which additionally rotates onto
  the axis the two divergent ancestors are laid out on *within* their shared
  rank -- see [Same-Rank Contact](#same-rank-contact) below.

Both share the same divergent-ancestor sibling-order input
(`sibling_index_from_cmp_to`, stored on `EdgeDescriptionTaffyNodes` alongside
the taffy node IDs, computed in `EdgeDescriptionBuilder::edge_desc_build` from
the same `sibling_index_from`/`sibling_index_to` values used for
`sibling_index_middle`), because both must account for an edge travelling
*against* the diagram's canonical `RankDir` flow (e.g. a `symmetric`
interaction group's reverse edge) -- naively assigning entry/exit purely from
`RankDir` would force such an edge through its waypoints in the wrong order,
backtracking through the box. This was a real, observed regression:
`edge_ix_client_server__1` in `020_interaction_halo_with_labels.yaml`
(`t_server -> t_client`, i.e. high-rank to low-rank under
`rank_dir: left_to_right`) once rendered as `... 456 -> 245(entry) ->
285(exit) -> 91 ...`, visibly looping back on itself.


### Cross-Rank Contact

A cross-rank edge's description box sits directly on the rank corridor
between its divergent ancestors. `EdgeSpacerCoordinatesCalculator::
calculate_description_thread` returns a proper corridor pair (`entry !=
exit`), the same shape `calculate` produces for ordinary spacers. The fixed
cross-axis coordinate mirrors `calculate`'s `cx`/`cy` convention (unchanged
between a `RankDir` and its reverse pair -- `top_y` for
`LeftToRight`/`RightToLeft`, `left_x` for `TopToBottom`/`BottomToTop`);
`Ordering::Less` (this edge's `from` is before its `to`, i.e. it travels in
the topological-forward direction) reuses `calculate`'s canonical entry/exit
assignment for that `RankDir` (substituting the fixed value for `cx`/`cy`);
`Ordering::Greater` (a reverse-direction edge, e.g. a `symmetric` interaction
group's response edge) swaps entry and exit so the pair always runs in *this
edge's own* travel direction rather than the diagram's canonical one:

| `RankDir` | fixed axis | `from` before `to` (`Less`) | else (`Greater`) |
|---|---|---|---|
| `LeftToRight` | `y = top_y` | entry=`(left_x,top_y)` exit=`(right_x,top_y)` | entry=`(right_x,top_y)` exit=`(left_x,top_y)` |
| `RightToLeft` | `y = top_y` | entry=`(right_x,top_y)` exit=`(left_x,top_y)` | entry=`(left_x,top_y)` exit=`(right_x,top_y)` |
| `TopToBottom` | `x = left_x` | entry=`(left_x,top_y)` exit=`(left_x,bottom_y)` | entry=`(left_x,bottom_y)` exit=`(left_x,top_y)` |
| `BottomToTop` | `x = left_x` | entry=`(left_x,bottom_y)` exit=`(left_x,top_y)` | entry=`(left_x,top_y)` exit=`(left_x,bottom_y)` |

`Ordering::Equal` should not occur (two distinct divergent ancestors always
have distinct sibling indices); treated the same as `Greater`.

Concretely, before this fix, `edge_dep_client_server__0` in
`020_interaction_halo_with_labels.yaml` (`rank_from: 0, rank_to: 1`) used a
single-point calculation and rendered pinned at the box's `left_x` with
wildly varying, out-of-box `y` values -- downstream spacer-ordering and
protrusion logic (built to expect a real two-point corridor, like every
other spacer kind) mishandled the degenerate zero-length waypoint. Threading
through the box properly fixed this.

This waypoint pair is folded into `SpacerCoordinatesResolver::resolve`'s
merged, sorted spacer list exactly like any other spacer kind, so protrusion
and turn-minimization logic (built generically over entry/exit corridors)
handle it without any special-casing.


### Same-Rank Contact

A same-rank (cycle) edge's divergent ancestors are laid out side by side
*within* their shared rank -- horizontally when the rank's own children stack
via `Row`/`RowReverse` (`RankDir::TopToBottom`/`BottomToTop`), vertically when
they stack via `Column`/`ColumnReverse` (`RankDir::LeftToRight`/
`RightToLeft`). The description box sits directly between them, on that
within-rank axis, so the path threads through it just like the cross-rank
case -- but on the axis the *siblings* are laid out on, not the diagram's
overall rank axis.

Because within-rank sibling order always matches declaration order
regardless of `RankDir`'s forward/reverse convention (see
[Sibling order for reversed rank directions](edge_paths.md#sibling-order-for-reversed-rank-directions)
in `edge_paths.md`), `Ordering::Less`/`Greater` here means the same thing
(`from`'s divergent ancestor sits earlier/later along the shared rank) for
both members of a forward/reverse `RankDir` pair -- unlike the cross-rank
case, where the physical meaning of `Less`/`Greater` flips between a
`RankDir` and its reverse pair. Only the horizontal-vs-vertical layout axis
depends on `RankDir`.

`EdgeSpacerCoordinatesCalculator::calculate_description_thread_same_rank`
therefore reuses `calculate_description_thread`'s table by rotating
`rank_dir` onto whichever of its two canonical rows matches the axis
same-rank siblings are actually laid out on: `TopToBottom`/`BottomToTop`
(horizontal siblings) both use the `LeftToRight` row (fixed `y = top_y`);
`LeftToRight`/`RightToLeft` (vertical siblings) both use the `TopToBottom`
row (fixed `x = left_x`).

Before this fix, the same-rank case used a single-point calculation (a fixed
corner of the box, biased by `sibling_index_from_cmp_to` to avoid two edges
sharing a box backtracking through its center) instead of threading through
-- visible in `019_interaction_halo.yaml`'s `edge_ix_client_server__0`
(`t_client`/`t_server`, same rank since there are no `thing_dependencies`
between them, only `thing_interactions`), which touched only its box's
top-left corner rather than running along its top edge. Also see
[Same-Rank (Cycle Edge) Placement](#same-rank-cycle-edge-placement) above
for the companion `FlexDirection` fix: the `edge_description_container` for a
same-rank group inverts `rank_container_style.flex_direction` (via
`taffy_container_builder::flex_direction_invert`) so multiple described
edges sharing that same-rank slot stack along the axis perpendicular to the
two divergent ancestors, instead of widening/heightening the gap between
them.


## Step-by-Step: How Face-Label Slots Are Built

Face-label slots are the taffy leaf nodes placed in each node's envelope
face-wrappers.  They are used downstream as contact-point anchors for edge path
routing.

### Step 1 -- `InputDiagram.edge_descs`

`InputDiagram.edge_descs` is a `Map<Id, String>`.  The user places a
description string keyed by the edge instance ID (e.g. `edge_dep__0`).

Source: `crate/input_model/src/input_diagram.rs`.

The map is carried through the pipeline as `IrDiagram.edge_descs`.  It is
**not** consulted during face-label slot construction or measurement.

### Step 2 -- `InputToIrDiagramMapper` computes face assignments

`InputToIrDiagramMapper` copies `edge_descs` verbatim into
`IrDiagram.edge_descs` and simultaneously computes two derived structures:

- `EdgeFaceAssignments` -- maps each edge ID to the face of its `from` node
  that the edge leaves and the face of its `to` node that the edge enters.
  Computed by `EdgeFaceAssigner::compute` from rank and sibling data
  (no pixel coordinates needed).

- `NodeFaceEdges` -- maps `(NodeId, NodeFace)` to the list of edge IDs that
  use that face.  Derived from `EdgeFaceAssignments` and `EdgeGroups` by
  `NodeFaceEdges::from_assignments`.

Source: `crate/input_ir_rt/src/input_to_ir_diagram_mapper.rs`,
`crate/input_ir_rt/src/edge_face_assigner.rs`,
`crate/ir_model/src/node/node_face_edges.rs`.

### Step 3 -- `IrToTaffyBuilder` builds face-label slots

This step happens inside `IrToTaffyBuilder::build_taffy_trees_for_dimension`.

#### Step 3a -- Envelope node construction

For each diagram node a taffy **envelope node** is built that wraps the node's
own content (`taffy_envelope_node_build`).  The envelope has four face-wrapper
containers (top, bottom, left, right).

For each face that has edges (looked up via `NodeFaceEdges::edges_for`), an
`EdgeLabel` leaf taffy node is created with `TaffyNodeCtx::EdgeLabel` context.
These leaves are collected in `edge_label_leaves`.

Source: `crate/input_ir_rt/src/ir_to_taffy_builder.rs` --
`taffy_envelope_node_build`, `taffy_envelope_node_build_face_leaves`.

#### Step 3b -- Layout measurement (`node_size_measure`)

During `compute_layout_with_measure`, `node_size_measure` is called for each
taffy node.  For `EdgeLabel` leaves the handler returns zero size -- no text
measurement is performed and `edge_descs` is not consulted here.

The leaf collapses to zero size in the layout, reserving a structural slot in
the envelope without affecting the node's rendered dimensions.

Source: `crate/input_ir_rt/src/ir_to_taffy_builder.rs` -- `node_size_measure`.

#### Step 3c -- `edge_label_taffy_nodes_build`

After layout, `edge_label_taffy_nodes_build` assembles the collected
`EdgeLabelLeafBuilt` entries into a `Map<EdgeId, EdgeLabelTaffyNodeIds>`.
Each entry maps an edge ID to its optional `from_label_taffy_node_id` and
`to_label_taffy_node_id`.

A leaf is assigned to `from_label_taffy_node_id` when its `node_id` matches the
edge's `from` endpoint and the pre-assigned `from_face` is `Some`.  The
`to_label_taffy_node_id` is assigned symmetrically.

Self-loop edges use only a `from_label` slot (since `from == to`, one slot
is sufficient). Contained edges (where one endpoint is an ancestor of the
other) produce label slots on both endpoints using forward or reverse faces
depending on hierarchy direction.

This map is stored as `TaffyNodeMappings::edge_label_taffy_nodes` and consumed
by the edge path routing code (see `edge_paths.md` -- Offset Calculation,
label-based offset).

Source: `crate/input_ir_rt/src/ir_to_taffy_builder.rs` --
`edge_label_taffy_nodes_build`, `edge_id_to_node_ids_build`.


## Key Requirements

### 1 -- Correct edge ID key

The `edge_descs` key must be the edge **instance** ID in the format
`{edge_group_id}__{edge_index}`, not the edge group ID by itself.

### 2 -- All edges produce face-label slots

Face-label slots are created for every edge that has a valid face assignment,
regardless of whether a description exists in `edge_descs`.  The slots are
always zero-size but are necessary for the edge path routing calculations.

Self-loop edges (`from == to`) produce a single `from_label` slot on the
rank-direction face of the node (`Bottom` for `TopToBottom`, `Top` for
`BottomToTop`, `Right` for `LeftToRight`, `Left` for `RightToLeft`);
`to_label` is `None` since one slot is sufficient.

Contained edges (where one endpoint is an ancestor of the other in the node
hierarchy) produce label slots on both endpoints. The faces used depend on
hierarchy direction, mirroring the forward/reverse face logic for regular edges:

- `from` is ancestor of `to` (downward): `from_face` = rank-dir face,
  `to_face` = opposite face (e.g. `Bottom`/`Top` for `TopToBottom`).
- `to` is ancestor of `from` (upward): `from_face` = opposite face,
  `to_face` = rank-dir face.

### 3 -- Face selection: unified pre-layout source (Option B)

Face assignment is computed once before taffy layout runs, by
`EdgeFaceAssigner::compute`.  The result is stored as
`IrDiagram::edge_face_assignments`.

`SvgEdgeInfosBuilder::build_edge_pass1_infos` looks up the pre-computed
assignment for each edge instead of re-computing faces from pixel geometry.
This guarantees that the face a label slot is reserved on always matches the
face the edge path exits.

**Special cases:**

- **Self-loops** (`from == to`): pre-layout assigns
  `(from_face: <rank-dir face>, to_face: None)` -- only one label slot is
  created.  Pass 1 duplicates the from face for both routing contacts, so the
  offset and protrusion machinery treats the loop as two contacts on the same
  face, and pass 2 routes it through the curvature-specific builders (an
  orthogonal U-shape with cycle-edge protrusions, or a curved loop).
- **Contained edges** (one endpoint is a pixel-level ancestor of the other):
  face-based contact points are bypassed (returns `(None, None)`), consistent
  with the pass-2 `is_node_contained_in` early-return.
- **Cycle edges** (same LCA rank, non-adjacent siblings):
  `EdgeFaceAssigner::cycle_faces` uses the same face mapping as
  `cycle_edge_faces_select` -- sibling index is a reliable proxy for
  horizontal/vertical relative position within a rank level.
- **Missing assignment** (should not occur for well-formed diagrams):
  falls back to the post-layout `faces_select`.


## Data Flow Summary

```
InputDiagram
  edge_descs: { "edge_dep__0": "A depends on B" }
       |
       | InputToIrDiagramMapper::map
       v
IrDiagram
  edge_descs: { "edge_dep__0": "A depends on B" }  --> edge_description_containers_plan.md
  edge_face_assignments: { "edge_dep__0": { from_face: Bottom, to_face: Top } }
  node_face_edges: { t_a: { Bottom: ["edge_dep__0"] },
                     t_b: { Top:    ["edge_dep__0"] } }
       |
       | IrToTaffyBuilder::build_taffy_trees_for_dimension
       v
TaffyNodeMappings
  edge_label_taffy_nodes:
    "edge_dep__0": { from_label: taffy_X (size 0), to_label: taffy_Y (size 0) }
       |
       | TaffyToSvgElementsMapper::map / SvgEdgeLabelsBuilder::build
       v
SvgElements
  edge_label_infos:
    [ SvgEdgeLabelInfo {
        edge_id: "edge_dep__0",
        from_label: Some(SvgEdgeLabelEndpointInfo {
          x, y, width, height,
          text_spans: []   -- always empty; no text from face-label slots
        }),
        to_label: Some(SvgEdgeLabelEndpointInfo { ... })
      } ]
       |
       | SvgEdgeInfosBuilder::face_offsets_compute (label-based offset)
       | SvgElementsToSvgMapper::render_edge_labels (no SVG output; text_spans empty)
       v
Edge path contact points use from_label / to_label bounds as offset anchors
(slot-based fallback applies since label size is 0).
```
