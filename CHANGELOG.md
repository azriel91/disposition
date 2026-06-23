# Changelog

## unreleased

* Replace `dioxus-clipboard` with custom clipboard support. ([#42][#42])
* Make `DarkModeCssSelector::MediaQuery` the default. ([#43][#43])
* Update dependency versions. ([#44][#44])
* Make insertion order consistent with declared order when `RankDir` is `BottomToTop` / `RightToLeft`. ([#45][#45])
* Update self-loop edges to select `NodeFace` based on `RankDir`. ([#45][#45])
* Arrow protrusions for the `to` end of an edge take into account arrow head length. ([#45][#45])
* Distribute the orthogonal edge protrusion gap between the `from` and `to` ends proportionally to each side's edge count, instead of reserving the full gap fraction per side. ([#45][#45])
* Reduce extra gap between edge description and its edge path. ([#45][#45])
* Markdown: Support nested lists, alpha / roman numeral ordered lists. ([#46][#46])
* Markdown: Single line spacing around lists. ([#46][#46])
* Markdown: Fix unintentional wrapping of last character. ([#46][#46])
* Markdown: Improve inline code background position and rounded corner. ([#46][#46])
* Markdown: Style inline code background and link text via Tailwind classes backed by light / dark theme variables, improving link contrast in dark mode. ([#46][#46])
* Markdown: Render edge labels as markdown, so they support inline styling (bold, italic, inline code, links) and images like node and edge descriptions. ([#46][#46])
* Support generating a diagram per process step / tag. ([#47][#47])
* Animate interaction edges at a constant pixel speed regardless of edge length, with a constant pause at the end of each cycle. ([#48][#48])
* Make interaction edges visible and animated by default when a diagram has no processes. ([#48][#48])
* Add "Interaction Timing" playground example diagram. ([#48][#48])
* Add rank stacking container to change rank container flex direction based on `RankDir`. ([#49][#49])
* Update taffy tree fmt labels to indicate the role of each taffy node (envelope, rank container, edge wrapper, etc.). ([#49][#49])
* Update `disposition_json_schema` to work. ([#50][#50])
* Update docs and JSON schema to explain how styling is applied to nodes and edges. ([#50][#50])
* Stagger orthogonal edge protrusions for nested-to-nested edges that clear the same divergent-ancestor sibling row, so their lateral routing segments no longer overlap. ([#51][#51])
* Surface `MAX_GAP_FRACTION`, `MIN_PROTRUSION_PX`, `TO_PROTRUSION_MIN_PX`, `ARC_RADIUS` constants for orthogonal edge geometry in `disposition_model_common::edge`. ([#51][#51])
* Order each node face's edge label slots by the opposite endpoint's nesting path, so edge contact points are arranged to minimise crossings. ([#51][#51])
* Increase `MIN_PROTRUSION_PX` to `5.0` and `MAX_GAP_FRACTION` to `0.9` to make staggered orthogonal edge protrusions more visually distinct. ([#51][#51])
* Build cross-container edge spacers based on the target's rank inside the container rather than the root-level distance between divergent ancestors, so an edge into a deeply-ranked nested child routes around the container's lower-rank siblings instead of overshooting back on itself. ([#52][#52])
* Calculate edge protrusions independently per node instead of using common protrusion for all nodes in a given rank. ([#53][#53])
* Ensure edges that exit a nested node spacer have different exit protrusions so their paths don't overlap. ([#53][#53])
* Split `RenderOptions.edge_curvature` into `RenderOptions.dependencies_edge_curvature` and `RenderOptions.interactions_edge_curvature`, so dependency and interaction edge curvature can be configured independently. ([#54][#54])
* Add `EdgeCurvature::DirectStraight`, which draws edges as straight lines directly between nodes, bypassing edge spacers (whose nodes collapse to zero size). ([#54][#54])
* Add `EdgeCurvature::DirectCurved`, which draws edges as bezier curves directly between nodes, bypassing edge spacers. ([#54][#54])
* Default `RenderOptions.interactions_edge_curvature` to `EdgeCurvature::DirectCurved`, so interaction edges are drawn as direct curves. ([#54][#54])
* Update the LSP JSON schema to offer the split edge curvature render options and the `direct_straight` / `direct_curved` values. ([#54][#54])
* Add the split dependency / interaction edge curvature controls, including the `Direct (Straight)` / `Direct (Curved)` options, to the playground render options editor. ([#54][#54])
* Edges are routed through spacers at every intermediate ancestor based on ranks to avoid overlapping higher `from` and lower `to` rank nodes. ([#55][#55])
* Separate edge contact points that exit the same face direction of different nodes at the same coordinate (e.g. a container and a node centered within it), so their protrusion stubs no longer overlap. ([#55][#55])
* Nest the approach legs of edges that enter the same node face from different rank-gap buckets (a cross-container edge and a local edge into the same nested node), so their paths no longer cross. ([#55][#55])
* Keep a container's face contact on the from-node's side of edges that transit the same inter-rank gap to reach a node nested inside it, so their legs no longer touch or cross. ([#55][#55])
* Markdown: Support rendering code blocks. ([#56][#56])
* Markdown: Support rendering blockquotes. ([#56][#56])

[#42]: https://github.com/azriel91/disposition/pull/42
[#43]: https://github.com/azriel91/disposition/pull/43
[#44]: https://github.com/azriel91/disposition/pull/44
[#45]: https://github.com/azriel91/disposition/pull/45
[#46]: https://github.com/azriel91/disposition/pull/46
[#47]: https://github.com/azriel91/disposition/pull/47
[#48]: https://github.com/azriel91/disposition/pull/48
[#49]: https://github.com/azriel91/disposition/pull/49
[#50]: https://github.com/azriel91/disposition/pull/50
[#51]: https://github.com/azriel91/disposition/pull/51
[#52]: https://github.com/azriel91/disposition/pull/52
[#53]: https://github.com/azriel91/disposition/pull/53
[#54]: https://github.com/azriel91/disposition/pull/54
[#55]: https://github.com/azriel91/disposition/pull/55
[#56]: https://github.com/azriel91/disposition/pull/56


## 0.3.0 (2026-06-07)

* Add `EdgeLabel` to allow different labels at each end of an edge. ([#32][#32])
* Render edge descriptions in the middle of the edge path. ([#32][#32])
* Replace `EntityDescs` with `ThingDescs` and `EdgeDescs`. ([#33][#33])
* Support rendering markdown in node and edge descriptions, including images. ([#34][#34])
* Include `ThemeAttr::Extra` classes in `Node` and `Edge` rendering. ([#35][#35])
* Use `<g transform="translate(x, y)">` to position markdown images for more intuitive transform origin coordinates. ([#35][#35])
* Add `ProcessRenderCollapse` render option to control whether processes are collapsed by default. ([#36][#36])
* Render process step edges using git-like layout. ([#37][#37])
* Serve playground example diagrams as static assets fetched at runtime instead of embedding them in the wasm binary. ([#38][#38])
* Refresh and expand the playground example diagrams to introduce features incrementally -- including edge labels and descriptions, markdown, and inline `data:` URL images. ([#38][#38])
* Add LSP server, and use CodeMirror for text editor. ([#39][#39])
* Rename `InputDiagram::things` to `InputDiagram::thing_names`, and `InputDiagram::thing_hierarchy` to `InputDiagram::things`. ([#40][#40])
* Improve LSP suggestions for map keys and theme styles. ([#40][#40])
* Large refactor to change diagram generation to to use smaller, functional blocks. ([#41][#41])

[#32]: https://github.com/azriel91/disposition/pull/32
[#33]: https://github.com/azriel91/disposition/pull/33
[#34]: https://github.com/azriel91/disposition/pull/34
[#35]: https://github.com/azriel91/disposition/pull/35
[#36]: https://github.com/azriel91/disposition/pull/36
[#37]: https://github.com/azriel91/disposition/pull/37
[#38]: https://github.com/azriel91/disposition/pull/38
[#39]: https://github.com/azriel91/disposition/pull/39
[#40]: https://github.com/azriel91/disposition/pull/40
[#41]: https://github.com/azriel91/disposition/pull/41


## 0.2.0 (2026-05-22)

* Support rendering tooltips. ([#26][#26])
* Add focus outlines around nodes and edges. ([#27][#27])
* Update `stroke_style: dashed` to mean `dasharray:4`. ([#27][#27])
* Fix duplication of tailwind classes on edges. ([#28][#28])
* Fix edge path routing issues regarding cross-container edges, spacers, and nested `NodeRank`s. ([#29][#29])
* Include `InputDiagram` source in generated SVG. ([#30][#30])
* Support rendering edge descriptions. ([#31][#31])

[#26]: https://github.com/azriel91/disposition/pull/26
[#27]: https://github.com/azriel91/disposition/pull/27
[#28]: https://github.com/azriel91/disposition/pull/28
[#29]: https://github.com/azriel91/disposition/pull/29
[#30]: https://github.com/azriel91/disposition/pull/30
[#31]: https://github.com/azriel91/disposition/pull/31


## 0.1.0 (2026-04-11)

* Add `playground`. ([#16][#16], [#17][#17])
* Support specifying `thing_layouts` in `InputDiagram`. ([#16][#16])
* Apply margin and padding to leaf nodes. ([#16][#16])
* Support laying out things by ranks, based on Thing Dependencies. ([#17][#17], [#18][#18])
* Offset edges so they don't overlap where they contact the node. ([#19][#19])
* Support light and dark mode diagrams. ([#20][#20], [#21][#21])
* Route edge path between nodes. ([#22][#22], [#23][#23])
* Support orthogonal edge paths. ([#22][#22])
* Set default `FlexDirection` based on `RankDir`. ([#22][#22])
* Reduce edge overlapping. ([#23][#23])
* Support generating JSON schema through `schemars`. ([#24][#24])

[#16]: https://github.com/azriel91/disposition/pull/16
[#17]: https://github.com/azriel91/disposition/pull/17
[#18]: https://github.com/azriel91/disposition/pull/18
[#19]: https://github.com/azriel91/disposition/pull/19
[#20]: https://github.com/azriel91/disposition/pull/20
[#21]: https://github.com/azriel91/disposition/pull/21
[#22]: https://github.com/azriel91/disposition/pull/22
[#23]: https://github.com/azriel91/disposition/pull/23
[#24]: https://github.com/azriel91/disposition/pull/24


## 0.0.4 (2026-02-22)

* Support `InputDiagram` merging. ([#10][#10])
* Add edges to diagram. ([#11][#11])
* Animate edges evenly by generating CSS animation keyframes. ([#12][#12])
* Add arrow heads to edges. ([#13][#13])
* Allow edge animations to begin on process step focus. ([#14][#14])
* Add support for circle node shapes. ([#15][#15])

[#10]: https://github.com/azriel91/disposition/pull/10
[#11]: https://github.com/azriel91/disposition/pull/11
[#12]: https://github.com/azriel91/disposition/pull/12
[#13]: https://github.com/azriel91/disposition/pull/13
[#14]: https://github.com/azriel91/disposition/pull/14
[#15]: https://github.com/azriel91/disposition/pull/15


## 0.0.3 (2026-01-20)

* Add `disposition_playground` dioxus app. ([#7][#7])
* Improve diagram rendering performance. ([#8][#8])
* Remove `syntect` syntax highlighting. ([#8][#8])
* Replace `cosmic-text` font width measurement with simple multiplication. ([#8][#8])
* Implement collapsible process steps. ([#9][#9])

[#7]: https://github.com/azriel91/disposition/pull/7
[#8]: https://github.com/azriel91/disposition/pull/8
[#9]: https://github.com/azriel91/disposition/pull/9


## 0.0.2 (2026-01-06)

* Rename `disposition_model` to `disposition_input_model`. ([#3][#3])
* Rename `disposition_ir` to `disposition_ir_model`. ([#3][#3])
* Add `disposition_model_common` for common data structures. ([#3][#3])
* Add `disposition_input_ir_rt` for runtime logic. ([#3][#3])
* Implement `InputToIrMapper` to transform input model to intermediate representation. ([#3][#3])
* Add `IrToTaffyBuilder` which maps an `IrDiagram` to `TaffyNodeMappings`. ([#4][#4])
* Compute `syntect` highlighted spans. ([#5][#5])
* Add `TaffyToSvgMapper` which maps `IrDiagram` and `TaffyNodeMapping` to an SVG. ([#6][#6])

[#3]: https://github.com/azriel91/disposition/pull/3
[#4]: https://github.com/azriel91/disposition/pull/4
[#5]: https://github.com/azriel91/disposition/pull/5
[#6]: https://github.com/azriel91/disposition/pull/6


## 0.0.1 (2025-12-06)

* Add `disposition_model` data structures. ([#1][#1], [#2][#2])
* Add `disposition_ir` data structures. ([#2][#2])

[#1]: https://github.com/azriel91/disposition/pull/1
[#2]: https://github.com/azriel91/disposition/pull/2


## 0.0.0 (2025-11-02)

* Publish empty `disposition` and `disposition_model` crates.
