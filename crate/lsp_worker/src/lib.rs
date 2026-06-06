//! Web Worker host for the `disposition` LSP server.
//!
//! Compiled to wasm and run inside a dedicated Web Worker (see the
//! `app/playground/assets/lsp_worker/worker_entry.js` entry point), this drives
//! a real [`async_lsp::MainLoop`] off the main thread. Messages arrive from the
//! page via `postMessage`, are framed and fed into the loop, and the loop's
//! framed output is unframed and posted back.
//!
//! Build it with `cargo run -p xtask -- build-worker`, which runs
//! `wasm-bindgen --target web` into the playground's `lsp_worker` asset folder.

use disposition_lsp::transport::{byte_pipe, frame, read_message, WORKER_READY};
use futures::io::BufReader;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};

/// Worker entry point, invoked automatically once the wasm module is
/// initialized (`wasm-bindgen` `start`).
///
/// Wires the worker's `postMessage` transport to an [`async_lsp::MainLoop`]
/// running on the worker's own event loop.
#[wasm_bindgen(start)]
pub fn worker_start() {
    let scope: DedicatedWorkerGlobalScope = js_sys::global().unchecked_into();

    let (server_input, main_loop_input) = byte_pipe();
    let (main_loop_output, server_output) = byte_pipe();

    // Drive the language server `MainLoop` on the worker's event loop.
    spawn_local(async move {
        let _ = disposition_lsp::server_run(main_loop_input, main_loop_output).await;
    });

    // Drain the `MainLoop`'s framed output, unframe it, and post each message
    // back to the main thread.
    spawn_local({
        let scope = scope.clone();
        async move {
            let mut reader = BufReader::new(server_output);
            while let Some(json) = read_message(&mut reader).await {
                if scope.post_message(&JsValue::from_str(&json)).is_err() {
                    return;
                }
            }
        }
    });

    // Editor -> server messages arrive as JSON strings via `postMessage`; frame
    // each and feed it into the `MainLoop`.
    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
        if let Some(json) = event.data().as_string() {
            server_input.send_bytes(frame(&json));
        }
    });
    scope.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    // The closure outlives this function: it is the worker's message handler.
    onmessage.forget();

    // Tell the main thread we are ready to receive messages.
    let _ = scope.post_message(&JsValue::from_str(WORKER_READY));
}
