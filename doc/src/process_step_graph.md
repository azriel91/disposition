# Process Step Graph (git-graph layout)

Process steps are laid out like `git log --graph`: each step is a circle in a
**lane** (column), steps are stacked in **rows** along the process's main axis,
and connector lines are drawn between steps following their dependencies. A step
shifts to a higher lane (further right) when a connector needs to **bypass** its
row, so the bypassing connector keeps a straight vertical line in a lower lane.
Step labels are left-aligned in a single text column to the right of the lanes.

This layout and its connectors are **separate** from the thing/tag flow layout
and the edge router described in `edge_paths.md`.


## Inputs

The layout builds on three IR fields (see `crate/ir_model/src/process/`):

- [`ProcessStepRanks`](crate/ir_model/src/process/process_step_ranks.rs) -- the
  rank of each step (used to order steps into rows).
- [`ProcessStepEdges`](crate/ir_model/src/process/process_step_edges.rs) -- the
  connector edges, derived from `process_step_dependencies` (linear-by-default
  when a process declares none).
- [`ProcessStepGraphs`](crate/ir_model/src/process/process_step_graphs.rs) -- the
  computed per-process [`ProcessStepGraph`](crate/ir_model/src/process/process_step_graph.rs):
  `lane_count`, each step's `ProcessStepPlacement { row, lane }`, and each
  connector's travel `lane`.


## Lane assignment (`ProcessStepGraphCalculator`)

Computed in
[`crate/input_ir_rt/src/process_step_graph_calculator.rs`](crate/input_ir_rt/src/process_step_graph_calculator.rs),
per process:

1. Order steps into rows by `(process_step_rank, declaration index)`.
2. Walk rows top-to-bottom maintaining `active_lanes: Vec<Option<NodeId>>` -- one
   reserved lane per pending connector, keyed by the step it heads to. At each
   step:
   - take the leftmost incoming-connector lane, or the first free lane if there
     is no incoming connector;
   - the first outgoing connector reuses the step's lane; additional outgoing
     connectors (branches) each claim a new free lane;
   - lanes whose connector targets a later row stay reserved -- these are the
     bypass lanes that push other steps to the right.
3. `lane_count` is the highest lane index used (by a step or connector) plus one.

Back connectors (target row at or before the source row, e.g. cycles) are
best-effort: they reuse the source step's lane and are not reserved.


## Taffy layout

Built in `TaffyDiagramNodeBuilder::process_node_step_graph_build` /
`process_step_graph_leaf_build`
([`taffy_diagram_node_builder.rs`](crate/input_ir_rt/src/ir_to_taffy_builder/taffy_diagram_node_builder.rs)).
A process with a `ProcessStepGraph` skips the usual rank-container path. Its
`wrapper_node` (flex column) holds the process label followed by one **step row**
per step, ordered by row:

```yaml
process_wrapper_node:        # flex column
  process_text_node: {}
  step_row:                  # flex row, align center -- one per step
    lane_gutter:             # fixed width = lane_count * LANE_WIDTH
      circle_node: {}        # margin-left offsets the circle to its lane
    text_node: {}            # step label -- starts after the gutter (aligned column)
  # ...
```

The fixed-width `lane_gutter` makes every step's `text_node` begin at the same x,
keeping labels left-aligned. The circle is centred within its lane via
`margin_left = (LANE_WIDTH - diameter) / 2 + lane * LANE_WIDTH`, so the circle
centre sits at `gutter_x + lane * LANE_WIDTH + LANE_WIDTH / 2`. `LANE_WIDTH` is a
constant in `disposition_taffy_model`. Each step is recorded as a
`NodeToTaffyNodeIds::ProcessStepGraphLeaf`; because the circle is nested inside
the gutter (not a direct sibling of the text), `HighlightedSpansComputer`
computes the text-span offset from the text node's own `location.x` so labels
stay aligned across lanes. Steps are not wrapped in envelopes (their connectors
are not drawn via envelope label slots).


## Connector router (`ProcessStepGraphEdgesBuilder`)

Built in
[`process_step_graph_edges_builder.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/process_step_graph_edges_builder.rs)
and appended to `svg_edge_infos` in `TaffyToSvgElementsMapper::map`. For each
connector it emits an `SvgEdgeInfo` with an orthogonal, arc-rounded `path_d`:

- the travel-lane x is `from_circle_centre_x + (travel_lane - from_lane) * LANE_WIDTH`;
- forward connectors exit the bottom of the `from` circle, run down the travel
  lane, and enter the top of the `to` circle; if the from/to/travel lanes all
  match it is a single straight vertical line;
- back connectors (cycles) bulge one lane to the right to avoid overlapping the
  steps between their endpoints.

The connector path is built as a `kurbo::BezPath` running from the `from` step to
the `to` step (the convention the shared `ArrowHeadBuilder` /
`EdgePathLocusCalculator` expect, placing the arrow at the path's final point),
so each connector also carries a positioned arrowhead at the `to` step and a
locus path for the focus indicator -- just like dependency edges. No spacers or
protrusions are used.

Connector tailwind classes are resolved during IR mapping (not in the router):
`TailwindClassesBuilder::build_process_step_connector_classes` styles connectors
like dependency edges -- the theme's base `edge_defaults` overlaid with
`type_dependency_edge_sequence_default` -- and the resulting string is stored in
`IrDiagram::tailwind_classes` keyed by `ProcessStepGraphEdge::edge_id` (prefixed
`edge_ps_`). Per-lane colouring is a possible future enhancement.
