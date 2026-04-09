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
10. Between the first and second pass, offsets from where the edge path exits the from-node, and where it enters the to-node, are computed, so that multiple edges do not all touch the from-node / to-node at the same coordinate, for visual clarity.
11. Also, for orthogonal (strictly horizontal / vertical) edge paths, a "protrusion" length is calculated so that the paths exit the node perpendicular to the node face for some length, so that the path is not drawn directly on the node face as a tangential line.
12. Offsets are the coordinate shift to where the edge path contacts the node face.
13. Protrusion is the length that the edge path extends out of the node face, so that an edge path isn't drawn directly on a node face.
14. The second pass is defined in [`edge_path_builder_pass_2.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/edge_path_builder_pass_2.rs)
15. The second pass computes the edge paths with the offsets and protrusion, which should result in paths that are visually non-overlapping with other paths and node content, creating visual clarity.

## Spacer Coordinate Direction Awareness

16. Spacer nodes are 5x5 px taffy leaf nodes inserted at intermediate ranks. After taffy computes the layout, each spacer's absolute position is resolved into a `SpacerCoordinates { entry_x, entry_y, exit_x, exit_y }`, representing the entry and exit points that the edge path passes through.
17. The entry and exit points of a spacer depend on the diagram's `RankDir`. This is implemented in `fn spacer_absolute_coordinates` in both [`svg_edge_infos_builder.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/svg_edge_infos_builder.rs) and [`ortho_protrusion_calculator.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/ortho_protrusion_calculator.rs):
    - `TopToBottom` -- entry at the top midpoint (smallest y), exit at the bottom midpoint (largest y). The path passes vertically downward through the spacer.
    - `BottomToTop` -- entry at the bottom midpoint (largest y), exit at the top midpoint (smallest y). The path passes vertically upward through the spacer.
    - `LeftToRight` -- entry at the left midpoint (smallest x), exit at the right midpoint (largest x). The path passes horizontally rightward through the spacer.
    - `RightToLeft` -- entry at the right midpoint (largest x), exit at the left midpoint (smallest x). The path passes horizontally leftward through the spacer.
18. When cross-container spacers are merged with rank-based spacers, they are sorted by the main-axis coordinate (`entry_y` for vertical flows, `entry_x` for horizontal flows) so that the spacers appear in the correct visual order along the edge path. This sorting is implemented in `fn spacer_coordinates_from_spacers` in [`svg_edge_infos_builder.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/svg_edge_infos_builder.rs) and `fn spacer_coordinates_resolve` in [`ortho_protrusion_calculator.rs`](crate/input_ir_rt/src/taffy_to_svg_elements_mapper/ortho_protrusion_calculator.rs).
