//! Language server for editing `disposition` [`InputDiagram`] YAML.
//!
//! Provides completion of YAML keys and values (struct fields, fixed enum
//! values, and IDs already present in the document) over the Language Server
//! Protocol, driven by an [`async_lsp`] `MainLoop`. Designed to compile to wasm
//! and run in the browser alongside the `disposition_playground` app.
//!
//! The host wires the server's framed byte streams to the editor's LSP client
//! and drives [`server_run`] to completion on an async executor (e.g.
//! `wasm_bindgen_futures::spawn_local`).
//!
//! [`InputDiagram`]: disposition_input_model::InputDiagram

pub mod completion;
pub mod language_server;
pub mod transport;

use async_lsp::{
    server::LifecycleLayer,
    MainLoop,
};
use futures::io::{AsyncRead, AsyncWrite};
use tower_layer::Layer;

pub use crate::language_server::DispositionLanguageServer;

/// Builds the language server `MainLoop` and drives it over the given byte
/// streams until the client disconnects.
///
/// `input` / `output` carry `Content-Length`-framed LSP messages (the standard
/// LSP wire format). The host adapts the editor's per-message transport to these
/// streams. Returns when the client sends `exit` or a stream closes.
pub async fn server_run<I, O>(input: I, output: O) -> async_lsp::Result<()>
where
    I: AsyncRead,
    O: AsyncWrite,
{
    let (main_loop, _client) = MainLoop::new_server(|client| {
        LifecycleLayer::default()
            .layer(async_lsp::router::Router::from_language_server(
                DispositionLanguageServer::new(client),
            ))
    });

    main_loop.run_buffered(input, output).await
}
