# Edge Descriptions

Edge descriptions are text labels displayed near the endpoints of edges in the
diagram.  They are specified via `entity_descs` in `InputDiagram`, keyed by the
edge **instance** ID (not the edge group ID).

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

In your `InputDiagram` (YAML or Rust), add an entry to `entity_descs`:

```yaml
thing_dependencies:
  edge_dep:
    kind: sequence
    things:
      - t_a
      - t_b

entity_descs:
  edge_dep__0: "A depends on B"
```

## Step-by-Step: How a Description Reaches the SVG

### Step 1 -- `InputDiagram.entity_descs`

`InputDiagram.entity_descs` is a `Map<Id, String>`.  The user places a
description string keyed by the edge instance ID (e.g. `edge_dep__0`).

Source: `crate/input_model/src/input_diagram.rs`.

### Step 2 -- `InputToIrDiagramMapper` copies entity_descs

`InputToIrDiagramMapper::build_entity_descs` copies the `entity_descs` map
verbatim into `IrDiagram.entity_descs`.

Simultaneously, two derived structures are computed that are needed later:

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

### Step 3 -- `IrToTaffyBuilder` builds taffy nodes and layout

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
taffy node including every `EdgeLabel` leaf.

For `DiagramLod::Normal`, the handler looks up
`entity_descs.get(edge_id.as_ref())`.  If a description is found, it computes
the text width and line count to return the leaf's size.  If not found the leaf
collapses to zero size (no label slot reserved).

Source: `crate/input_ir_rt/src/ir_to_taffy_builder.rs` -- `node_size_measure`.

#### Step 3c -- `edge_label_taffy_nodes_build`

After layout, `edge_label_taffy_nodes_build` assembles the collected
`EdgeLabelLeafBuilt` entries into an
`Map<EdgeId, EdgeLabelTaffyNodeIds>`.  Each entry maps an edge ID to its
optional `from_label_taffy_node_id` and `to_label_taffy_node_id`.

A leaf is assigned to `from_label_taffy_node_id` when its `node_id` matches the
edge's `from` endpoint and the pre-assigned `from_face` is `Some`.  The
`to_label_taffy_node_id` is assigned symmetrically.

Self-loop edges use only a `from_label` slot (since `from == to`, one slot
is sufficient). Contained edges (where one endpoint is an ancestor of the
other) produce label slots on both endpoints using forward or reverse faces
depending on hierarchy direction.

Source: `crate/input_ir_rt/src/ir_to_taffy_builder.rs` --
`edge_label_taffy_nodes_build`, `edge_id_to_node_ids_build`.

#### Step 3d -- `highlighted_spans_compute`

For `DiagramLod::Normal`, `highlighted_spans_compute` iterates over
`edge_label_taffy_nodes`.  For each entry it:

1. Looks up `entity_descs.get(edge_id.as_ref())`.  If no description is found,
   the edge is skipped.
2. Reads the taffy layout width of the `from_label` slot (or `to_label` as
   fallback) to get the wrapping constraint.
3. Calls `wrap_text_monospace` to compute wrapped lines.
4. Builds `EntityHighlightedSpan` values (x, y, width, height, text) relative
   to the label node's top-left corner.
5. Inserts the spans into `entity_highlighted_spans` keyed by the edge's `Id`.

Source: `crate/input_ir_rt/src/ir_to_taffy_builder.rs` --
`highlighted_spans_compute`.

### Step 4 -- `TaffyToSvgElementsMapper` builds `SvgEdgeLabelInfo`

`SvgEdgeLabelsBuilder::build` iterates over `edge_label_taffy_nodes`.  For
each edge it:

1. Looks up `entity_highlighted_spans.get(edge_id.as_ref())` to get the
   pre-computed span list.
2. Calls `endpoint_info_build` for both `from_label` and `to_label` taffy
   nodes.  This reads the absolute SVG position of the label slot from the
   taffy layout and offsets each span's (x, y) accordingly.
3. Produces an `SvgEdgeLabelInfo` with `from_label` and `to_label` fields, each
   holding a `SvgEdgeLabelEndpointInfo` that contains the label slot bounds and
   the list of `SvgTextSpan` values.

The `SvgEdgeLabelInfo` values are collected into `SvgElements.edge_label_infos`.

Source: `crate/input_ir_rt/src/taffy_to_svg_elements_mapper/svg_edge_labels_builder.rs`.

### Step 5 -- `SvgElementsToSvgMapper` renders the labels

`render_edge_labels` iterates over `SvgElements.edge_label_infos`.  For each
entry whose `from_label` or `to_label` has non-empty `text_spans`, it writes:

```svg
<g id="{edge_id}__from_label" class="...">
  <text x="..." y="..." stroke-width="0">line text</text>
  ...
</g>
```

Source: `crate/input_ir_rt/src/svg_elements_to_svg_mapper.rs` --
`render_edge_labels`.

## Key Requirements

### 1 -- Correct edge ID key

The `entity_descs` key must be the edge **instance** ID in the format
`{edge_group_id}__{edge_index}`, not the edge group ID by itself.

### 2 -- Level of detail

Edge descriptions are only rendered at `DiagramLod::Normal`.  The small (`sm`)
dimension uses `DiagramLod::Simple` by default and will not show descriptions.
Use `DimensionAndLod::default_md()`, `default_lg()`, `default_2xl()`, or
`default_no_limit()` to get `Normal` detail.

### 3 -- All edges produce label slots

Self-loop edges (`from == to`) produce a single `from_label` slot on the
`Bottom` face of the node; `to_label` is `None` since one slot is sufficient.

Contained edges (where one endpoint is an ancestor of the other in the node
hierarchy) produce label slots on both endpoints. The faces used depend on
hierarchy direction, mirroring the forward/reverse face logic for regular edges:

- `from` is ancestor of `to` (downward): `from_face` = rank-dir face,
  `to_face` = opposite face (e.g. `Bottom`/`Top` for `TopToBottom`).
- `to` is ancestor of `from` (upward): `from_face` = opposite face,
  `to_face` = rank-dir face.

## Data Flow Summary

```
InputDiagram
  entity_descs: { "edge_dep__0": "A depends on B" }
       |
       | InputToIrDiagramMapper::map
       v
IrDiagram
  entity_descs: { "edge_dep__0": "A depends on B" }
  edge_face_assignments: { "edge_dep__0": { from_face: Bottom, to_face: Top } }
  node_face_edges: { t_a: { Bottom: ["edge_dep__0"] },
                     t_b: { Top:    ["edge_dep__0"] } }
       |
       | IrToTaffyBuilder::build_taffy_trees_for_dimension
       v
TaffyNodeMappings
  edge_label_taffy_nodes:
    "edge_dep__0": { from_label: taffy_X, to_label: taffy_Y }
  entity_highlighted_spans:
    "edge_dep__0": [ EntityHighlightedSpan { text: "A depends on B", ... } ]
       |
       | TaffyToSvgElementsMapper::map / SvgEdgeLabelsBuilder::build
       v
SvgElements
  edge_label_infos:
    [ SvgEdgeLabelInfo {
        edge_id: "edge_dep__0",
        from_label: Some(SvgEdgeLabelEndpointInfo {
          text_spans: [ SvgTextSpan { x, y, text: "A depends on B" } ]
        }),
        to_label: Some(SvgEdgeLabelEndpointInfo { ... })
      } ]
       |
       | SvgElementsToSvgMapper::render_edge_labels
       v
SVG string
  <g id="edge_dep__0__from_label" ...>
    <text x="..." y="..." stroke-width="0">A depends on B</text>
  </g>
  <g id="edge_dep__0__to_label" ...>
    ...
  </g>
```
