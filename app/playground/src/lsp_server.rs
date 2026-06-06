//! Hosts the `disposition_lsp` language server for the YAML editor.
//!
//! [`DispositionLspServer`] implements the editor component's async LSP server
//! trait by driving an [`async_lsp::MainLoop`], bridged to the editor's
//! per-message transport. There are two hosts, selected by the `lsp-worker`
//! feature:
//!
//! * **default** -- runs the loop in-page on the main thread
//!   (`crate::lsp_server::disposition_lsp_server::DispositionLspServer`).
//! * **`lsp-worker`** -- runs the loop in a dedicated Web Worker
//!   (`crate::lsp_server::disposition_lsp_server_worker::DispositionLspServer`),
//!   keeping it off the main thread.
//!
//! Both expose a `DispositionLspServer` with the same `new()` constructor, so
//! the editor wiring is identical either way.
//!
//! [`async_lsp::MainLoop`]: https://docs.rs/async-lsp/latest/async_lsp/struct.MainLoop.html

#[cfg(not(feature = "lsp-worker"))]
pub mod disposition_lsp_server;
#[cfg(feature = "lsp-worker")]
pub mod disposition_lsp_server_worker;

#[cfg(not(feature = "lsp-worker"))]
pub use self::disposition_lsp_server::DispositionLspServer;
#[cfg(feature = "lsp-worker")]
pub use self::disposition_lsp_server_worker::DispositionLspServer;
