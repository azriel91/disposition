//! Web Worker LSP server host for the `InputDiagram` YAML editor.
//!
//! Implements [`LspServerAsync`] by spawning a dedicated Web Worker that runs
//! the `disposition_lsp` [`async_lsp::MainLoop`] off the main thread (see
//! `disposition_lsp_worker`). Editor messages are forwarded to the worker with
//! `postMessage`; messages the worker posts back are pushed to the editor's LSP
//! client.
//!
//! Enabled by the `lsp-worker` feature. The worker's wasm must be built first
//! with `cargo run -p xtask -- build-worker`.
//!
//! [`CodeMirror`]: dioxus_codemirror::CodeMirror
//! [`async_lsp::MainLoop`]: https://docs.rs/async-lsp/latest/async_lsp/struct.MainLoop.html

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use dioxus::prelude::{asset, manganis, Asset, AssetOptions};
use dioxus_codemirror::{LspMessage, LspPusher, LspServerAsync};
use disposition_lsp::transport::WORKER_READY;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{MessageEvent, Worker, WorkerOptions, WorkerType};

/// The worker's asset folder, containing `worker_entry.js` plus the
/// wasm-bindgen output built by `cargo run -p xtask -- build-worker`.
const LSP_WORKER_DIR: Asset = asset!("/assets/lsp_worker", AssetOptions::folder());

/// Hosts the `disposition_lsp` language server in a Web Worker and bridges it
/// to the editor's LSP client.
#[derive(Default)]
pub struct DispositionLspServer {
    /// The spawned worker and its message plumbing. `Some` once
    /// [`LspServerAsync::lsp_pusher_set`] has started the worker.
    host: Option<WorkerHost>,
}

impl DispositionLspServer {
    /// Returns a new, not-yet-started server.
    pub fn new() -> Self {
        Self::default()
    }
}

/// A running worker and the state needed to talk to it.
struct WorkerHost {
    /// The dedicated Web Worker running the LSP `MainLoop`.
    worker: Worker,
    /// Whether the worker has signalled [`WORKER_READY`].
    ready: Rc<Cell<bool>>,
    /// Messages queued before the worker became ready, flushed on ready.
    outbox: Rc<RefCell<Vec<String>>>,
    /// Keeps the `message` handler alive for the worker's lifetime.
    _onmessage: Closure<dyn FnMut(MessageEvent)>,
}

impl LspServerAsync for DispositionLspServer {
    fn lsp_pusher_set(&mut self, pusher: LspPusher) {
        let worker_options = WorkerOptions::new();
        worker_options.set_type(WorkerType::Module);

        let worker_url = format!("{LSP_WORKER_DIR}/worker_entry.js");
        let Ok(worker) = Worker::new_with_options(&worker_url, &worker_options) else {
            return;
        };

        let ready = Rc::new(Cell::new(false));
        let outbox = Rc::new(RefCell::new(Vec::<String>::new()));

        // Worker -> main: ready signal flushes the outbox; everything else is an
        // LSP message pushed to the editor's client.
        let onmessage = {
            let worker = worker.clone();
            let ready = Rc::clone(&ready);
            let outbox = Rc::clone(&outbox);
            Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
                let Some(data) = event.data().as_string() else {
                    return;
                };

                if data == WORKER_READY {
                    ready.set(true);
                    for queued in outbox.borrow_mut().drain(..) {
                        let _ = worker.post_message(&JsValue::from_str(&queued));
                    }
                    return;
                }

                pusher.lsp_message_push(LspMessage::new(data));
            })
        };
        worker.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));

        self.host = Some(WorkerHost {
            worker,
            ready,
            outbox,
            _onmessage: onmessage,
        });
    }

    fn lsp_message_handle(&mut self, message: LspMessage) {
        let Some(host) = self.host.as_ref() else {
            return;
        };

        let json = message.json_into();
        if host.ready.get() {
            let _ = host.worker.post_message(&JsValue::from_str(&json));
        } else {
            // The worker is still loading; queue until it signals readiness.
            host.outbox.borrow_mut().push(json);
        }
    }
}
