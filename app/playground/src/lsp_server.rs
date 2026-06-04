//! Hosts the `disposition_lsp` language server in the page for the YAML editor.
//!
//! [`DispositionLspServer`] implements the editor component's async LSP server
//! trait by driving an [`async_lsp::MainLoop`], bridged to the editor's
//! per-message transport through an in-memory framed byte pipe
//! ([`lsp_stream_bridge`]).

pub mod disposition_lsp_server;
pub mod lsp_stream_bridge;

pub use self::disposition_lsp_server::DispositionLspServer;
