# Edge Descriptions Plan

## Overview

This document describes the steps needed to add per-edge description labels to
the diagram without having them overlap other nodes or other edges' labels.

The high-level approach is:

1. Determine which face each edge exits/enters per node **before** layout, using
   rank and sibling data rather than post-layout pixel coordinates.
2. Wrap each diagram node's taffy subtree in a new `envelope_node` that adds
   flex-row/column slots for edge label leaf nodes on each face.
3. Read the computed label positions after layout and emit `SvgEdgeLabelInfo`
   elements.
4. Route edge paths to the envelope boundary (rather than the inner wrapper
   boundary) so labels sit between the node rectangle and the edge path.

---

## Prerequisites / shared concerns

### P1 -- Promote `NodeFace` to `ir_model`

`NodeFace` is currently `pub(super)` in
`crate/input_ir_rt/src/taffy_to_svg_elements_mapper/edge_model.rs`. Every phase
below needs it.

Move it to a new file
`crate/ir_model/src/node/node_face.rs` and make it public. Update all
existing use sites in `taffy_to_svg_elements_mapper/`.

`NodeFace` variants: `Top`, `Bottom`, `Left`, `Right`.

---

## Phase 1 -- Pre-layout face assignment

**Goal:** compute `(from_face, to_face)` for every edge using only rank and
sibling data, and store the per-node counts in `IrDiagram` so `IrToTaffyBuilder`
can build the right number of edge label slots.

### Step 1.1 -- `EdgeFaceAssignment` type

New file: `crate/ir_model/src/edge/edge_face_assignment.rs`

```rust
pub struct EdgeFaceAssignment {
    pub from_face: Option<NodeFace>,
    pub to_face: Option<NodeFace>,
}
```

`None` for both faces means the edge is a contained edge (one endpoint is an
ancestor of the other) and does not touch either node's face.

### Step 1.2 -- `EdgeFaceAssignments` collection

New file: `crate/ir_model/src/edge/edge_face_assignments.rs`

```rust
pub struct EdgeFaceAssignments<'id>(Map<EdgeId<'id>, EdgeFaceAssignment>);
```

Standard newtype with `get`, `insert`, `iter`, `is_empty`.

### Step 1.3 -- `NodeFaceEdges` collection

New file: `crate/ir_model/src/node/node_face_edges.rs`

```rust
pub struct NodeFaceEdges<'id>(Map<NodeId<'id>, Map<NodeFace, Vec<EdgeId<'id>>>>);
```

Provides:
- `edges_for(node_id, face) -> &[EdgeId]`
- `face_edge_count(node_id, face) -> usize`

Derived from `EdgeFaceAssignments`: for every `(edge_id, assignment)`, push
`edge_id` into `from_node -> from_face` and `to_node -> to_face`.

### Step 1.4 -- `EdgeFaceAssigner` calculator

New file: `crate/input_ir_rt/src/edge_face_assigner.rs`

```rust
pub struct EdgeFaceAssigner;
impl EdgeFaceAssigner {
    pub fn compute<'id>(
        edge_groups: &EdgeGroups<'id>,
        entity_types: &EntityTypes<'id>,
        node_nesting_infos: &NodeNestingInfos<'id>,
        node_ranks_nested: &NodeRanksNested<'id>,
        rank_dir: RankDir,
    ) -> EdgeFaceAssignments<'id>
}
```

For each edge in each group, apply the rules below (in priority order):

| Case | from_face | to_face |
|---|---|---|
| Self-loop (`from == to`) | `Bottom` | `Bottom` |
| Contained edge (one chain is prefix of other) | `None` | `None` |
| Cycle edge (same LCA rank, sibling distance > 1) | clockwise (see below) | clockwise |
| Forward edge (`lca_rank_from < lca_rank_to`) | rank-dir face (see below) | opposite face |
| Reverse edge (`lca_rank_from > lca_rank_to`) | opposite of forward | rank-dir face |

**Rank-direction face** (the face of the `from` node for a forward edge):
- `LeftToRight` / `RightToLeft` (reversed) → `Right`
- `TopToBottom` / `BottomToTop` (reversed) → `Bottom`
- For reversed directions the face ordering is negated: `RightToLeft` forward
  edge exits `Left`, `BottomToTop` exits `Top`.

**Clockwise cycle face** (mirrors `EdgePathBuilderPass1::cycle_edge_faces_select`
but uses sibling indices instead of pixel coordinates):
- `from_sibling_index < to_sibling_index` in a LTR/RTL context: `(Top, Top)`.
- `from_sibling_index > to_sibling_index` in a LTR/RTL context: `(Bottom, Bottom)`.
- Same logic for TTB/BTT context using vertical sibling ordering.

LCA-rank computation should reuse the same logic as
`SvgEdgeInfosBuilder::nodes_lca_ranks_compute` (Step P1 extracts this into a
shared location, or it is duplicated).

### Step 1.5 -- Add fields to `IrDiagram`

In `crate/ir_model/src/ir_diagram.rs`, add two new fields:

```rust
pub edge_face_assignments: EdgeFaceAssignments<'id>,
pub node_face_edges: NodeFaceEdges<'id>,
```

In `InputToIrDiagramMapper::map`
(`crate/input_ir_rt/src/input_to_ir_diagram_mapper.rs`), after step 14
(NodeRanksNested) and 13 (NodeNestingInfos):

- Step 15: `edge_face_assignments = EdgeFaceAssigner::compute(...)`
- Step 16: `node_face_edges = NodeFaceEdges::from(&edge_face_assignments, &edge_groups)`

Update `IrDiagram::into_static` to call `.into_static()` on both new fields.

---

## Phase 2 -- Envelope taffy nodes

**Goal:** wrap each diagram node's taffy subtree with an `envelope_node` that
has flex-row/column edge label slots on each face.

The existing `wrapper_node` becomes `diagram_node_wrapper_node` inside the
envelope. All existing rank containers and child nodes stay inside
`diagram_node_wrapper_node` unchanged.

### Step 2.1 -- `TaffyNodeCtx::EdgeLabel` variant

In `crate/taffy_model/src/taffy_node_ctx.rs`, add a third variant:

```rust
EdgeLabel(EdgeLabelCtx)
```

New file `crate/taffy_model/src/edge_label_ctx.rs`:

```rust
pub struct EdgeLabelCtx {
    pub edge_id: EdgeId<'static>,
    pub node_id: NodeId<'static>,  // the endpoint node this label is attached to
    pub face: NodeFace,
}
```

### Step 2.2 -- `EdgeLabelTaffyNodeIds` type

New file: `crate/taffy_model/src/edge_label_taffy_node_ids.rs`

```rust
pub struct EdgeLabelTaffyNodeIds {
    /// Label slot on the `from` endpoint's face. None for contained/self-loop edges.
    pub from_label_taffy_node_id: Option<taffy::NodeId>,
    /// Label slot on the `to` endpoint's face. None for contained/self-loop edges.
    pub to_label_taffy_node_id: Option<taffy::NodeId>,
}
```

### Step 2.3 -- Update `TaffyNodeMappings`

In `crate/taffy_model/src/taffy_node_mappings.rs`, add:

```rust
/// Map from edge ID to its edge label taffy leaf node IDs.
pub edge_label_taffy_nodes: Map<EdgeId<'static>, EdgeLabelTaffyNodeIds>,
/// Map from diagram node ID to its envelope taffy node ID.
pub node_id_to_envelope_taffy_node: Map<NodeId<'static>, taffy::NodeId>,
```

Keeping `node_id_to_envelope_taffy_node` separate (rather than changing
`NodeToTaffyNodeIds` variants) minimises churn in all existing code that reads
`node_id_to_taffy`.

### Step 2.4 -- Envelope node structure

Two methods on `IrToTaffyBuilder`:

- `fn taffy_envelope_node_build` -- builds the full envelope structure.
- `fn taffy_envelope_node_build_face_leaves` -- extracted helper (per naming
  convention: called function name is prefixed by the calling function name)
  that creates label leaves for one face and appends them to an accumulator.

`taffy_envelope_node_build` takes `diagram_node_wrapper_node` and
`node_face_edges`, and builds:

```yaml
envelope_node:               # (flex column, align_items: Stretch)
  edge_wrapper_top:          # (flex row,    children = label leaves for Top edges)
  edge_and_diagram_wrapper:  # (flex row,    align_items: Stretch)
    edge_wrapper_left:       # (flex column, children = label leaves for Left edges)
    diagram_node_wrapper_node
    edge_wrapper_right:      # (flex column, children = label leaves for Right edges)
  edge_wrapper_bottom:       # (flex row,    children = label leaves for Bottom edges)
```

Each label leaf is created with:

```rust
TaffyNodeCtx::EdgeLabel(EdgeLabelCtx { edge_id, node_id, face })
```

The `edge_wrapper_*` nodes with zero children are still created so the flex
layout stays consistent; they will have zero size.

The return type is `(taffy::NodeId, Vec<EdgeLabelLeafBuilt>)` where
`EdgeLabelLeafBuilt` (defined in
`crate/input_ir_rt/src/ir_to_taffy_builder/taffy_node_build_context.rs`) is:

```rust
pub(crate) struct EdgeLabelLeafBuilt {
    pub(crate) edge_id: EdgeId<'static>,
    pub(crate) node_id: NodeId<'static>,
    pub(crate) face: NodeFace,
    pub(crate) taffy_node_id: taffy::NodeId,
}
```

This carries per-leaf info out of envelope construction so step 2.5 can
populate `edge_label_taffy_nodes` after all nodes have been built.

### Step 2.5 -- Wire into `IrToTaffyBuilder`

Modify `build_taffy_nodes_for_node_without_child_hierarchy` and
`build_taffy_nodes_for_node_with_child_hierarchy` in
`crate/input_ir_rt/src/ir_to_taffy_builder.rs`:

1. Build `diagram_node_wrapper_node` (the existing `wrapper_node`) as before.
2. Call `taffy_envelope_node_build` with `diagram_node_wrapper_node` and
   `node_face_edges`. Collect the returned `Vec<EdgeLabelLeafBuilt>` alongside
   the returned `envelope_node`.
3. Return `envelope_node` (not `wrapper_node`) as the ID stored in rank
   containers and passed up the call stack.
4. Record `node_id → envelope_node` in `node_id_to_envelope_taffy_node`.
5. After all nodes are built, merge the collected `EdgeLabelLeafBuilt` entries
   into `edge_label_taffy_nodes`. For each `EdgeLabelLeafBuilt`:
   - Look up `edge_face_assignments.get(&built.edge_id)` and the raw edge
     (from `edge_groups`) to obtain its `.from` and `.to` node IDs.
   - If `assignment.from_face.is_some()` and `built.node_id == edge.from`,
     this leaf is the `from_label_taffy_node_id` for that edge.
   - If `assignment.to_face.is_some()` and `built.node_id == edge.to`,
     this leaf is the `to_label_taffy_node_id` for that edge.
   - Both slots default to `None` for contained and self-loop edges.

Add `node_face_edges` to `TaffyNodeBuildContext` alongside the existing fields
so it is available during the recursive child-node build (used by the two
`build_taffy_nodes_for_node_*` functions, which receive the context).

`edge_face_assignments` is only needed during the post-build collection in
step 5 above, so it can be passed directly at that call site rather than
threaded through `TaffyNodeBuildContext`.

### Step 2.6 -- Edge label text measurement

In `IrToTaffyBuilder::node_size_measure`, add a match arm for
`TaffyNodeCtx::EdgeLabel(ctx)`:

- Look up `entity_descs.get(ctx.edge_id.as_ref())` for the description text.
  `EntityDescs` is keyed by `Id<'static>` and `EdgeId` wraps `Id`, so
  `ctx.edge_id.as_ref()` gives `&Id<'static>` which works as the lookup key.
- Apply the same monospace wrapping logic used for `DiagramNode`.
- `NodeMeasureContext` already carries `entity_descs`, so no new context field
  is needed.

---

## Phase 3 -- SVG rendering of edge labels

**Goal:** after layout, read edge label positions and emit them as SVG elements.

### Step 3.1 -- `SvgEdgeLabelInfo` type

New file: `crate/svg_model/src/svg_edge_label_info.rs`

```rust
pub struct SvgEdgeLabelInfo<'id> {
    pub edge_id: EdgeId<'id>,
    pub from_label: Option<SvgEdgeLabelEndpointInfo>,
    pub to_label: Option<SvgEdgeLabelEndpointInfo>,
}

pub struct SvgEdgeLabelEndpointInfo {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub text_spans: Vec<SvgTextSpan>,
}
```

### Step 3.2 -- `SvgEdgeLabelsBuilder`

New file:
`crate/input_ir_rt/src/taffy_to_svg_elements_mapper/svg_edge_labels_builder.rs`

Iterates `TaffyNodeMappings::edge_label_taffy_nodes`. For each
`EdgeLabelTaffyNodeIds`:
- Call `SvgNodeInfoBuilder::node_absolute_xy_coordinates` on the label taffy
  node to get `(x, y)`.
- Read `layout.size.width` / `height` from `taffy_tree.layout(label_node_id)`.
- Read syntax-highlighted spans from `entity_highlighted_spans` using the
  `edge_id` (same mechanism as node text spans, once `highlighted_spans_compute`
  is extended to handle `TaffyNodeCtx::EdgeLabel` nodes).
- Build and return `Vec<SvgEdgeLabelInfo>`.

### Step 3.3 -- Add to `SvgElements` and `TaffyToSvgElementsMapper`

In `SvgElements` (`crate/svg_model/src/svg_elements.rs`), add:

```rust
pub edge_label_infos: Vec<SvgEdgeLabelInfo<'static>>,
```

Call `SvgEdgeLabelsBuilder::build` in `TaffyToSvgElementsMapper` and store the
result.

### Step 3.4 -- SVG template

In `SvgElementsToSvgMapper`, iterate `edge_label_infos` and emit a `<text>`
element for each non-empty `from_label` / `to_label`. Use the same `<tspan>`
structure as node text spans. Apply Tailwind / entity CSS classes via the
`edge_id`.

---

## Phase 4 -- Edge path routing around label nodes

**Goal:** edge paths connect to the envelope boundary (outer edge of the label
wrappers) rather than the wrapper boundary, so labels are not drawn over.

### Step 4.1 -- Envelope bounds in `SvgNodeInfo`

`SvgNodeInfoBuilder` currently reads `x, y, width, height_collapsed` from the
`wrapper_taffy_node_id` (now `diagram_node_wrapper_node`). This must stay as-is
for the SVG `<rect>` path -- the visible node rectangle should not change.

Add envelope fields to `SvgNodeInfo` in `crate/svg_model/src/svg_node_info.rs`:

```rust
pub envelope_x: f32,
pub envelope_y: f32,
pub envelope_width: f32,
pub envelope_height_collapsed: f32,
```

In `SvgNodeInfoBuilder::build`, look up `node_id_to_envelope_taffy_node` to get
the envelope taffy node and compute its absolute coordinates using the same
`node_absolute_xy_coordinates` helper.

### Step 4.2 -- Face selection uses envelope bounds

In `SvgEdgeInfosBuilder::build_edge_pass1_infos`, pass envelope bounds to
`EdgePathBuilderPass1::faces_select` and `select_edge_faces` (currently these
read `svg_node_info.x/y/width/height_collapsed`). Switch them to use
`envelope_x/y/width/envelope_height_collapsed` so face contact points lie on
the envelope boundary.

Contained-edge detection (`is_node_contained_in`) should continue to use the
inner `wrapper` bounds, not the envelope bounds, to avoid false positives from
envelope overlap.

### Step 4.3 -- Offset and protrusion calculations use envelope bounds

`face_offsets_compute` and `OrthoProtrusionCalculator::calculate` use
`svg_node_info_map` to look up node dimensions for face lengths and protrusion
extents. Update those lookups to use `envelope_width` / `envelope_height_collapsed`
instead of `width` / `height_collapsed`. This ensures protrusions clear the
full label area, not just the inner node rectangle.

### Step 4.4 -- Reconcile pre-layout vs post-layout face selection

The pre-layout face assignment (Phase 1) and the post-layout face selection
(currently in `SvgEdgeInfosBuilder`) may give different results in ambiguous
cases (e.g. diagonal layouts). Options:

- **Option A (recommended initially):** Keep both computations. Use pre-layout
  results only for building envelope slots (Phase 2). Post-layout results drive
  path routing as before. Accept that a label slot may be on a different face
  than the path exits if the two disagree.
- **Option B (cleaner long-term):** Replace the post-layout face selection with
  the pre-layout result. Store `EdgeFaceAssignments` in `TaffyNodeMappings` and
  read it in `SvgEdgeInfosBuilder` instead of re-deriving faces. This requires
  verifying the approximation is accurate enough for all layout configurations.

---

## Open questions

### OQ1 -- Contained edges

Contained edges (one endpoint is an ancestor of the other) have no face
assignment and therefore no envelope label slots. If `entity_descs` contains a
description for such an edge, it will not be rendered as a label. Consider
rendering it mid-path as a `<text>` element offset from the path midpoint.

### OQ2 -- `entity_highlighted_spans` extension

Currently `highlighted_spans_compute` only handles `TaffyNodeCtx::DiagramNode`.
It must be extended to iterate edge label leaves (`TaffyNodeCtx::EdgeLabel`) and
produce syntax-highlighted spans keyed by `edge_id` (not `node_id`). A separate
map `edge_highlighted_spans: Map<EdgeId, Vec<HighlightedSpan>>` may be cleaner
than adding a mixed key to the existing `EntityHighlightedSpans`.

### OQ3 -- Multiple edges per face

When two edges share the same face on the same node, each gets its own label
slot. These stack in a flex column (left/right) or flex row (top/bottom). The
slot order should match the face-offset slot order produced by
`face_offsets_compute` (rank distance ascending, then target coordinate) so
labels align with their corresponding edge exit points.

Currently `taffy_envelope_node_build_face_leaves` iterates
`node_face_edges.edges_for(node_id, face)` in insertion order, which is the
order edges appear in `EdgeGroups`. Before step 2.5 is finalised, verify that
this order agrees with `face_offsets_compute`'s slot ordering, or add an
explicit sort inside `taffy_envelope_node_build_face_leaves`.

### OQ4 -- Edge label slot styling

Edge label slots should have `flex_shrink: 0` and appropriate padding/margin
(similar to node text nodes) so they do not compress. The envelope flex layout
should use `align_items: Stretch` to keep label wrappers and the diagram node
wrapper at the same cross-axis size.

### OQ5 -- Edges with no description

If `entity_descs` has no entry for an edge, the label leaf still exists in the
taffy tree but measures to zero size. The envelope will therefore have zero-size
wrappers for those faces. This is correct behaviour -- no visible label is
emitted and the layout is unchanged from the no-envelope case.

### OQ6 -- `taffy_to_svg_elements_mapper/svg_node_info_builder.rs` call to `wrapper_taffy_node_id`

The method `NodeToTaffyNodeIds::wrapper_taffy_node_id()` is currently used to
obtain the layout for `SvgNodeInfo`. After Phase 2 this still refers to
`diagram_node_wrapper_node`. Verify that process height subtraction logic
(which currently subtracts `proc_info.total_height` from `height_expanded`)
continues to use the inner `wrapper` dimensions, not the envelope.
