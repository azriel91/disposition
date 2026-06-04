//! In-page LSP server host for the `InputDiagram` YAML editor.
//!
//! Implements [`LspServerAsync`] so the [`CodeMirror`] editor's LSP client can
//! drive a real [`async_lsp::MainLoop`] running on the main thread. Editor
//! messages are framed and fed into the loop; the loop's framed output is
//! drained, unframed, and pushed back to the editor -- including unprompted
//! server messages (e.g. future `textDocument/publishDiagnostics` diagnostics).
//!
//! [`CodeMirror`]: dioxus_codemirror::CodeMirror

use dioxus::prelude::spawn;
use dioxus_codemirror::{LspMessage, LspPusher, LspServerAsync};
use futures::{
    io::{AsyncReadExt, BufReader},
    AsyncBufReadExt,
};

use crate::lsp_server::lsp_stream_bridge::{byte_pipe, PipeReader, PipeWriter};

/// Hosts the `disposition_lsp` language server in the page and bridges it to the
/// editor's LSP client.
#[derive(Default)]
pub struct DispositionLspServer {
    /// Write half feeding framed editor -> server messages into the `MainLoop`.
    /// `Some` once [`LspServerAsync::lsp_pusher_set`] has started the server.
    server_input: Option<PipeWriter>,
}

impl DispositionLspServer {
    /// Returns a new, not-yet-started server.
    pub fn new() -> Self {
        Self::default()
    }
}

impl LspServerAsync for DispositionLspServer {
    fn lsp_pusher_set(&mut self, pusher: LspPusher) {
        let (server_input, main_loop_input) = byte_pipe();
        let (main_loop_output, server_output) = byte_pipe();
        self.server_input = Some(server_input);

        // Drive the language server `MainLoop` to completion on the main thread.
        spawn(async move {
            let _ = disposition_lsp::server_run(main_loop_input, main_loop_output).await;
        });

        // Drain the `MainLoop`'s framed output, unframe it, and push each
        // message to the editor's LSP client.
        spawn(messages_drain(server_output, pusher));
    }

    fn lsp_message_handle(&mut self, message: LspMessage) {
        let Some(server_input) = self.server_input.as_ref() else {
            return;
        };

        let json = message.json();
        let framed = format!("Content-Length: {}\r\n\r\n{json}", json.len());
        server_input.send_bytes(framed.into_bytes());
    }
}

/// Reads `Content-Length`-framed LSP messages from `server_output` and pushes
/// each unframed JSON message to the editor via `pusher`, until the stream ends.
async fn messages_drain(server_output: PipeReader, pusher: LspPusher) {
    let mut reader = BufReader::new(server_output);

    loop {
        // Read headers until the blank separator line, capturing Content-Length.
        let mut content_length = None;
        loop {
            let mut line = Vec::new();
            match reader.read_until(b'\n', &mut line).await {
                Ok(0) => return, // End of stream.
                Ok(_) => {}
                Err(_) => return,
            }

            let header = String::from_utf8_lossy(&line);
            let header = header.trim_end_matches(['\r', '\n']);
            if header.is_empty() {
                break;
            }
            if let Some(value) = header.strip_prefix("Content-Length:") {
                content_length = value.trim().parse::<usize>().ok();
            }
        }

        let Some(length) = content_length else {
            return;
        };

        let mut body = vec![0u8; length];
        if reader.read_exact(&mut body).await.is_err() {
            return;
        }

        let json = String::from_utf8_lossy(&body).into_owned();
        if !pusher.lsp_message_push(LspMessage::new(json)) {
            // Editor torn down; stop draining.
            return;
        }
    }
}
