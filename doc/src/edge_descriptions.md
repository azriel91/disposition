# Edge Descriptions

Edge descriptions are text labels associated with edges in the diagram.  They
are specified via `edge_descs` in `InputDiagram`, keyed by the edge
**instance** ID (not the edge group ID).

> **Note:** `EdgeDescs` is **not** rendered through face-label slots.
> Description text is rendered via `edge_description_container` nodes
> interleaved between rank containers.  See
> `edge_description_containers_plan.md` for the implementation plan of that
> feature.

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
`edge_description_containers_plan.md` for details.


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
