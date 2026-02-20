# 📐 disposition

[![Crates.io](https://img.shields.io/crates/v/disposition.svg)](https://crates.io/crates/disposition)
[![docs.rs](https://img.shields.io/docsrs/disposition)](https://docs.rs/disposition)
[![CI](https://github.com/azriel91/disposition/workflows/CI/badge.svg)](https://github.com/azriel91/disposition/actions/workflows/ci.yml)
[![Coverage Status](https://codecov.io/gh/azriel91/disposition/branch/main/graph/badge.svg)](https://codecov.io/gh/azriel91/disposition)

SVG diagram generator.

> [!NOTE]
>
> 🚧 This crate is a work in progress.
>
> See design notes at <https://peace.mk/book/side_projects/disposition.html>.


## Features

<details open>

* [x] Pure Rust.
* [x] SVG for lossless resolution.
* [x] Stable layout.
* [x] Sensible default styling.
* [x] Customizable styling.
* [x] Interactive highlighting.
* [x] CSS only interactivity.
* [x] Edges between nodes.
* [x] Edge animations representing requests/responses.
* [x] Arrows on edges.
* [x] Circle as node shape.
* [ ] Dependencies between process steps.
* [ ] Tooltips.
* [ ] Images in nodes.
* [ ] Responsive layout.
* [ ] Light and dark modes.

</details>


## Usage

Add the following to `Cargo.toml`:

```toml
[dependencies]
disposition = "0.0.3"
```

In code:

```rust
use disposition;

todo!("This crate is a work in progress.")
```


## Development

### Playground

Install `dx` (dioxus build tool):

```bash
cargo install cargo-binstall
cargo binstall dioxus-cli
```

Run `dx serve` in `app/playground`:

```bash
cd app/playground
dx serve
```


## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.


### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
