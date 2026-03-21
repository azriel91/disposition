# Diagram Generation

Diagram generation from an `InputDiagram` into an SVG goes through the following steps:

1. Merge user `InputDiagram` over base `InputDiagram`.

    Source: `InputDiagramMerger` in `crate/input_ir_rt/src/input_diagram_merger.rs`.

2. Calculate diagram intermediate representation (`IrDiagram`) from the merged `InputDiagram`.

    Source: `InputToIrDiagramMapper` in `crate/input_ir_rt/src/input_to_ir_diagram_mapper.rs`.

3. Calculate SVG element node and text coordinates (`TaffyNodeMappings`) using `taffy`.

    Source: `IrToTaffyBuilder` in `crate/input_ir_rt/src/ir_to_taffy_builder.rs`.

4. Calculate SVG elements and edges including attributes (`SvgElements`) based on `IrDiagram` and `TaffyNodeMappings`.

    Source: `TaffyToSvgElementsMapper` in  `crate/input_ir_rt/src/taffy_to_svg_elements_mapper.rs`.

5. Write SVG string based on `SvgElements`.

    Source: `SvgElementsToSvgMapper` in `crate/input_ir_rt/src/svg_elements_to_svg_mapper.rs`.
