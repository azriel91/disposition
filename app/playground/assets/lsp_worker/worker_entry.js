// Web Worker entry for the disposition LSP server.
//
// Loads the wasm-bindgen module produced by:
//
//     cargo run -p xtask -- build-worker
//
// The module's `start` function (`worker_start`) installs the worker's message
// handler, so initializing the module is all that is needed here.
import init from "./disposition_lsp_worker.js";

init();
