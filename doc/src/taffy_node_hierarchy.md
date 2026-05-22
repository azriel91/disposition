# Taffy Node Hierarchy

The taffy tree represents the entire diagram layout. It is built by `IrToTaffyBuilder` in
[`crate/input_ir_rt/src/ir_to_taffy_builder.rs`](crate/input_ir_rt/src/ir_to_taffy_builder.rs) and
is used to compute SVG node and text coordinates (`TaffyNodeMappings`).

The tree has one root and two major kinds of node: **inbuilt container nodes** (always-present
structural nodes) and **diagram nodes** (one taffy sub-tree per node in the user's `IrDiagram`).


## Inbuilt Container Nodes

The following structural nodes are always present at the top of the tree regardless of diagram
content:

```yaml
Root:
  TagsContainer:
    # [rank containers for tags]
    #   [tag diagram nodes]
  ThingsAndProcessesContainer:
    ProcessesContainer:
      # [rank containers for processes]
      #   [process diagram nodes]
    ThingsContainer:
      # [rank containers for things]
      #   [thing diagram nodes]
```

`TagsContainer`, `ThingsContainer`, and `ProcessesContainer` each have their flex direction
**inverted** from the direction the user configured. This inversion is intentional: rank
sub-containers carry the user's configured direction (e.g. `Row`), so the parent that stacks those
rank containers must use the perpendicular direction (`Column`) to make them stack correctly.


## Rank Containers

Top-level nodes are grouped by their `NodeRank`. For each distinct rank value a separate **rank
container** taffy node is created. All diagram nodes at rank `n` are placed as direct children of
the rank `n` container, and rank containers are placed as children of the relevant inbuilt container
(`TagsContainer`, `ProcessesContainer`, or `ThingsContainer`).

The same grouping applies recursively inside container diagram nodes: the direct children of a
container diagram node are grouped by their rank value, and a dedicated rank container is created
for each rank at that nesting level. See [Container diagram nodes](#container-diagram-nodes-rect-shape)
below for the exact sub-tree shape.

Edge spacer nodes may also be inserted into rank containers. See `edge_spacers.md` for details.


## Leaf Diagram Nodes (Rect Shape)

A diagram node that has no children and uses a rectangular shape maps to a single taffy leaf node:

    text_node    (leaf, DiagramNodeCtx context, measured for text size)

Stored as `NodeToTaffyNodeIds::Leaf { text_node_id }`.


## Leaf Diagram Nodes (Circle Shape)

A diagram node that has no children and uses a circular shape maps to a small sub-tree:

```yaml
wrapper_node:      # (flex row, align items center, gap 4px)
  circle_node: {}  # (fixed square size = diameter, flex_shrink 0)
  text_node: {}    # (leaf, DiagramNodeCtx context, measured for text size)
```

Stored as `NodeToTaffyNodeIds::LeafWithCircle { wrapper_node_id, circle_node_id, text_node_id }`.

The `wrapper_node` lays the circle and the text side-by-side in a row, vertically centred.


## Container Diagram Nodes (Rect Shape)

A diagram node that has children and uses a rectangular shape maps to:

```yaml
wrapper_node:              # (flex column, wrapper_style)
  text_node: {}            # (leaf, DiagramNodeCtx context)
  rank_container_0:        # (child_container_style, children at rank 0)
    # [child diagram nodes at rank 0]
    # [edge spacer nodes at rank 0, if any]
  rank_container_1:        # (child_container_style, children at rank 1)
    # [child diagram nodes at rank 1]
    # [edge spacer nodes at rank 1, if any]
  # ...
```

Stored as `NodeToTaffyNodeIds::Wrapper { wrapper_node_id, text_node_id }`.

The `wrapper_node` is always `flex_direction: Column` so the label and the rank sub-containers
stack vertically. Each `rank_container_n` uses the user-configured flex direction for its children.


## Container Diagram Nodes (Circle Shape)

A diagram node that has children and uses a circular shape maps to:

```yaml
wrapper_node:              # (flex column, wrapper_style)
  label_wrapper_node:      # (flex row, align items center, gap 4px)
    circle_node: {}        # (fixed square size = diameter, flex_shrink 0)
    text_node: {}          # (leaf, DiagramNodeCtx context)
  rank_container_0:        # (child_container_style, children at rank 0)
    # [child diagram nodes at rank 0]
  rank_container_1:        # (child_container_style, children at rank 1)
    # [child diagram nodes at rank 1]
  # ...
```

Stored as
`NodeToTaffyNodeIds::WrapperCircle { wrapper_node_id, label_wrapper_node_id, circle_node_id, text_node_id }`.

The `label_wrapper_node` mirrors the leaf-with-circle layout (circle + text in a row, centred), and
the rank containers below it follow the same pattern as the rect-shape container.


## Envelope Node

Every diagram node is wrapped in an **envelope node** -- a CSS Grid container that reserves
face-wrapper slots for edge label text on each of the four sides of the node.  The envelope
uses a 3x3 grid, with the center cell holding the node's own content and four edge-wrapper
cells on the cardinal faces.  The four corner cells are left empty.

```yaml
envelope_node:               # (grid 3x3, auto-sized columns and rows)
  edge_wrapper_top:          # row 1, col 2 -- flex row, label leaves for Top edges
  edge_wrapper_left:         # row 2, col 1 -- flex column, label leaves for Left edges
  diagram_node_wrapper_node: # row 2, col 2 -- the node's own content sub-tree
  edge_wrapper_right:        # row 2, col 3 -- flex column, label leaves for Right edges
  edge_wrapper_bottom:       # row 3, col 2 -- flex row, label leaves for Bottom edges
```

All four corner cells (row 1 col 1, row 1 col 3, row 3 col 1, row 3 col 3) are empty and
occupy zero space.  When no edges attach to a face the corresponding `edge_wrapper_*` node
has no children and collapses to zero size.

The `diagram_node_wrapper_node` is given explicit `grid_row: line(2)` / `grid_column: line(2)`
placement (via `set_style`) after it is created, so it always lands in the center cell
regardless of the order in which children are appended to `envelope_node`.

Because grid tracks are `auto`-sized, the center cell expands to accommodate whichever
adjacent cell is largest.  For example, if the `edge_wrapper_top` label is wide, column 2
grows to fit it, and `diagram_node_wrapper_node` stretches to fill that wider column.

Each label leaf inside an `edge_wrapper_*` node is created with `TaffyNodeCtx::EdgeLabel`
context so it is measured for text during layout.  Faces that have no edges still produce an
`edge_wrapper_*` node (but with zero children), keeping the grid structure consistent.


## Taffy Node Styles

### Container Style (`taffy_container_style`)

Used for inbuilt containers, rank containers at the top level, and leaf diagram nodes.

- `display: Flex`
- `border: 1px`
- `align_items: Stretch`, `align_content: Start`, `justify_items: Start`, `justify_content: Start`
- `flex_direction`: from node layout
- `flex_wrap`: from node layout
- `gap`: from node layout
- `margin`, `padding`: from `NodeLayout::Flex` or `NodeLayout::Leaf`

### Wrapper Style (`taffy_wrapper_node_styles` -- `wrapper_style` field)

Used for the outer `wrapper_node` of container diagram nodes (both rect and circle shapes).

- `display: Flex`
- `flex_direction: Column` (always column -- text label and rank sub-containers stack vertically)
- `flex_wrap: NoWrap`
- `align_items: FlexStart` (and related alignment fields)
- `margin`, `padding`, `border`: from node layout

### Text Style (`taffy_wrapper_node_styles` -- `text_style` field)

Used for the `text_node` that lives inside a `wrapper_node`.

- `padding` left/right: from node layout
- `padding` top/bottom: zero (to avoid double-counting with the wrapper node's padding)

### Child Container Style (`taffy_wrapper_node_styles` -- `child_container_style` field)

Used for rank sub-containers that sit inside a container diagram node.

- `display: Flex`
- `flex_shrink: 0` (prevents the container from compressing below its content size)
- `flex_direction`: from node layout (the user-configured direction for children)
- `flex_wrap`: from node layout
- `gap`: from node layout


## TaffyNodeCtx Variants

Each taffy node optionally carries a context value. The context is used during layout measurement
and post-layout coordinate extraction.

- `TaffyNodeCtx::DiagramNode(DiagramNodeCtx { entity_id, entity_type })` -- attached to `text_node`
  and bare-leaf nodes that represent diagram nodes. Used during measurement to determine text
  content and sizing.
- `TaffyNodeCtx::EdgeSpacer(EdgeSpacerCtx { edge_id, rank })` -- attached to spacer leaf nodes
  inserted to help route edge paths.
- `None` -- structural nodes carry no context. This includes `wrapper_node`, `circle_node`,
  `label_wrapper_node`, rank containers, and all inbuilt containers.


## Text Measurement

Text nodes are measured during `taffy_tree.compute_layout_with_measure(...)` via
`IrToTaffyBuilder::node_size_measure`.

The text content depends on the diagram level of detail (`DiagramLod`):

- `DiagramLod::Simple`: the node name only.
- `DiagramLod::Normal`: `"# {name}\n\n{description}"` when a description exists; otherwise just the
  name.

Width is estimated using a monospace character width ratio
(`MONOSPACE_CHAR_WIDTH_RATIO * TEXT_FONT_SIZE`). The estimator computes wrapped line widths and
line counts from the available width constraint, then converts them to pixel dimensions that taffy
uses to size the node.

Syntax highlighting spans are computed **after** layout is complete, not during measurement, to
avoid redundant work during the iterative constraint-solving phase.


## Node ID Maps

Two maps are maintained alongside the taffy tree to allow fast look-up in both directions:

- `node_id_to_taffy`: given a diagram `NodeId`, returns the `NodeToTaffyNodeIds` variant for that
  node (containing all taffy `NodeId` values for its sub-tree).
- `taffy_id_to_node`: given a taffy `NodeId`, returns the diagram `NodeId` it belongs to. This
  reverse map is used when traversing the computed layout to extract positions.
