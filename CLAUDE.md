# Project: `disposition`

Rust SVG diagram generation library with a dioxus web frontend.

## Architecture

1. The main `disposition` crate is a wrapper around sub-crates.
2. Subcrates (e.g. `disposition_input_model`) are placed in `crate/input_model`.
3. The `disposition_playground` crate is in `app/playground`, a dioxus 0.7 application.


## Code Style / Patterns

1. Variable names should usually be the snake case version of their type, e.g. for the `NodeId`, use `node_id`.
2. When working with multiple variables of the same type, favour a common prefix so that they are grouped when sorted by name, e.g. `node_id_from`, `node_id_to`.
3. When creating functions, favour naming them in `noun_verb` order, so they are discoverable when searching by name, and grouped together when sorted. e.g. `thing_rename`, `thing_delete`.
4. When creating functions, favour adding them within a type's `impl` block instead of a non-scoped function, This helps group related functions together. e.g. `impl ThingBuilder { fn node_coordinates(thing: &Thing) -> Point<f32, f32> { .. } }`.
5. When extracting functions to keep function bodies small, if it is in the same type, the calling function name should be a prefix of the called function name, so that it is easy to identify functions that are easy to extract. If the extracted function is used by multiple calling functions, often that indicates it should be extracted to a separate type.
6. Favour using strong types, e.g. `NodeId` when handling Node IDs, but `Id` when handling different kinds of IDs (e.g. `NodeId`, `EdgeId`, etc.), instead of using `String` / `&str`, so that it is clear what concept the variable should represent, and the constraints built into those types are guaranteed.
7. This means do NOT map a type into a `String` / `&str` unless necessary to compile.
8. If you must use "stringly typed" / standard data types (e.g. primitive types, `String`, `Map`, `Vec`, `Set`, etc.), then instead of using tuples, create a new type with field names that indicate what information those types hold. e.g. instead of `(String, Vec<String>)`, use `struct NodeIdToCssClasses { node_id: String, css_classes: Vec<String>, }`
9. Each type usually is defined in its own snake_case module, e.g. `NodeId` would be in `node_id.rs`.
10. "Data model" types are usually separate from "logic" types -- so data models can be published without publishing logic. Sometimes logic modules contain supporting data types to pass parameters.
11. Try and keep functions under 200 lines by extracting logic / components with meaningful names.
12. Avoid non-ascii characters, e.g. "—". Use "--" for elaboration or "`param`: description" in parameter documentation.
13. Section comments should be written as `// === Section Name === //` instead of a 3 line comment.
14. Documentation should include example valid values.
15. Unless a type / component is small, it should be placed in its own module -- often a submodule of the current module.
16. A module and its submodules would be `foo.rs` and `foo/bar.rs`, not `foo/mod.rs` and `foo/bar.rs`.


## Additional Context

1. When editing the `disposition_playground` crate, see <@agent/dioxus.md> if you need context on working with dioxus `0.7`.
2. See `<@doc/src/diagram_generation.md>` for a high level overview of the diagram generation process.
3. See `<@doc/src/edge_descriptions.md>` for how edge labels are computed from the `InputDiagram` through to the `SvgElements`.
4. See `<@doc/src/edge_paths.md>` for context about how edge paths are calculated, including node rank concepts and offset/protrusion routing.
5. See `<@doc/src/node_nesting_info.md>` for how `NodeNestingInfo` and `NodeNestingInfos` are built -- covers `NodeNestingInfosBuilder`, the `ancestor_chain` / `nesting_path` fields, and how they are used by rank computation and edge spacer insertion.
6. See `<@doc/src/node_ranks.md>` for how `NodeRank` and `NodeRanksNested` are computed per hierarchy level -- covers `NodeRanksCalculator`, LCA edge lifting, SCC-based cycle handling, and the full worked example.
7. See `<@doc/src/taffy_node_hierarchy.md>` for how the `taffy` layout node tree is structured -- covers inbuilt containers, rank containers, leaf and container diagram nodes (rect and circle shapes), taffy node styles, and `TaffyNodeCtx` variants.
8. See `<@doc/src/edge_spacers.md>` for how edge spacer taffy nodes are inserted to help route edges around diagram nodes -- covers same-level cross-rank spacers and cross-container spacers, including the `EdgeSpacerBuildDecider` decision logic and insertion-index accounting.
9. See `<@doc/src/edge_description_containers_plan.md>` for the step-by-step plan to render edge descriptions as container nodes interleaved between rank containers.
10. See `<@doc/src/md_node_content_plan.md>` for the step-by-step plan to render node and edge description text as syntax-highlighted markdown with inline images.
11. See `<@doc/src/process_step_graph.md>` for the git-graph layout of process steps -- covers `ProcessStepGraphCalculator` lane packing, the lane/text-column taffy grid, and the `ProcessStepGraphEdgesBuilder` connector router.


## Diagnosing Diagram Generation

To diagnose / check the values generated at each stage of the diagram generation
pipeline, run the `disposition_cli` and output the desired intermediate data to
stdout:

```bash
cargo run -q -p disposition_cli -- --structure-only --data $intermediate_data --stdout $input_diagram_yaml_file 2>&1
```

* `$intermediate_data` is one of `ir-diagram`, `taffy-tree`, `edge-routing`, `svg-elements`, or
  `svg`. The pipeline only computes up to the requested stage.
* `--structure-only` omits styles / colors so the structural values are easier
  to read; drop it to see styled values.
* `--stdout` writes the selected data straight to stdout instead of files, and
  `2>&1` merges any mapping issues (written to stderr) into the output.
* `$input_diagram_yaml_file` is the input diagram, e.g.
  `workspace_tests/src/input_diagram/0018_images_animated.yaml`.

Omit `--data` / `--stdout` and pass an output directory to instead write all
stages (`ir_diagram.yaml`, `taffy_tree.txt`, `edge_routing.yaml`, `svg_elements.yaml`, `diagram.svg`)
to files.


## Tests

Tests for all crates are placed inside the `workspace_tests` crate, and are run using the command:

```bash
cargo nextest run --workspace --all-features --no-tests warn
```

documentation is checked using the commands:

```bash
cargo test --doc --workspace --all-features
cargo doc --workspace --all-features --no-deps
```
