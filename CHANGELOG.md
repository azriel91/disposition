# Changelog

## unreleased

* Support `InputDiagram` merging. ([#10][#10])
* Add edges to diagram. ([#11][#11])
* Animate edges evenly by generating CSS animation keyframes. ([#12][#12])

[#10]: https://github.com/azriel91/disposition/pull/10
[#11]: https://github.com/azriel91/disposition/pull/11
[#12]: https://github.com/azriel91/disposition/pull/12


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
