# Diagram Generation

Diagram generation from an `InputDiagram` into an SVG goes through the following steps:

1. Merge user `InputDiagram` over base `InputDiagram`.

    Source: `InputDiagramMerger` in `crate/input_ir_rt/src/input_diagram_merger.rs`.

2. Calculate diagram intermediate representation (`IrDiagram`) from the merged `InputDiagram`.

    Source: `InputToIrDiagramMapper` in `crate/input_ir_rt/src/input_to_ir_diagram_mapper.rs`.

3. Calculate SVG element node and text coordinates (`TaffyNodeMappings`) using `taffy`.

    Source: `IrToTaffyBuilder` in `crate/input_ir_rt/src/ir_to_taffy_builder.rs`.

    Key construction steps (per dimension, inside `build_taffy_trees_for_dimension`):

    - Diagram node taffy sub-trees are built recursively for all nodes. For each container node,
      `build_taffy_nodes_for_node_with_child_hierarchy` handles its children.

    - `EdgeSpacerBuilder::build` is called at the same three trigger points to insert rank-based
      spacers for cross-rank edges (once per entity type per container, plus once per entity type
      at the top level).

    - `EdgeDescriptionBuilder::build` is called at the same three trigger points (once per entity
      type per container with `lca_node_id = Some(&container_id)`, and once per entity type at the
      top level with `lca_node_id = None`). Each call creates `edge_description_container` taffy
      nodes interleaved among the rank containers for all described edges at that LCA level.

    - `EdgeSpacerBuilder::build_edge_desc_container_spacers` is called immediately after each
      `EdgeDescriptionBuilder::build`, inserting spacer leaves inside each
      `edge_description_container` for any other edge whose rank span crosses that container.

    - After layout is computed, `highlighted_spans_compute` builds `entity_highlighted_spans` for
      node and edge-label text, and `highlighted_spans_compute_edge_desc_containers` builds
      `edge_description_highlighted_spans` for edge description text.

4. Calculate SVG elements and edges including attributes (`SvgElements`) based on `IrDiagram` and `TaffyNodeMappings`.

    Source: `TaffyToSvgElementsMapper` in  `crate/input_ir_rt/src/taffy_to_svg_elements_mapper.rs`.

    In addition to node and edge SVG elements, this step now also calls
    `SvgEdgeDescriptionsBuilder::build`, which produces `SvgEdgeDescriptionInfo` values from
    `edge_description_taffy_nodes` and `edge_description_highlighted_spans`. The results are
    stored in `SvgElements::edge_description_infos`.

5. Write SVG string based on `SvgElements`.

    Source: `SvgElementsToSvgMapper` in `crate/input_ir_rt/src/svg_elements_to_svg_mapper.rs`.

    Edge descriptions are rendered as `<g id="{edge_id}__desc" class="edge-description">` elements
    with `<text>` children for each wrapped line, after edge labels and before the closing `</svg>`.
