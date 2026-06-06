# Changelog

## unreleased

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
* Improve LSP suggestions for map keys and theme styles. ([#41][#41])

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
