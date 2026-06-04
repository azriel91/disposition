//! End-to-end test driving the real [`async_lsp::MainLoop`] over in-memory,
//! `Content-Length`-framed byte streams -- the same wiring used in the browser,
//! minus the editor. Exercises `initialize` -> `didOpen` -> `completion`.

use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    executor::block_on,
    io::{AsyncReadExt, BufReader},
    AsyncBufReadExt, StreamExt,
};
use serde_json::{json, Value};

#[test]
fn completion_over_main_loop_returns_top_level_keys() {
    // client -> server and server -> client byte pipes.
    let (client_writer, server_reader) = byte_pipe();
    let (server_writer, mut client_reader) = client_pipe();

    let server = disposition_lsp::server_run(server_reader, server_writer);

    let client = async move {
        // initialize.
        write_message(
            &client_writer,
            &json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": { "capabilities": {} }
            }),
        );
        let initialize_response = read_response(&mut client_reader, 1).await;
        assert!(
            initialize_response["result"]["capabilities"]["completionProvider"].is_object(),
            "expected completionProvider capability, got {initialize_response}"
        );

        // initialized + didOpen (empty document).
        write_message(
            &client_writer,
            &json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }),
        );
        write_message(
            &client_writer,
            &json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": "file:///diagram.yaml",
                        "languageId": "yaml",
                        "version": 1,
                        "text": ""
                    }
                }
            }),
        );

        // completion at the start of the empty document.
        write_message(
            &client_writer,
            &json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "textDocument/completion",
                "params": {
                    "textDocument": { "uri": "file:///diagram.yaml" },
                    "position": { "line": 0, "character": 0 }
                }
            }),
        );
        let completion_response = read_response(&mut client_reader, 2).await;
        let labels = completion_response["result"]
            .as_array()
            .expect("expected completion result array")
            .iter()
            .map(|item| item["label"].as_str().unwrap_or_default().to_string())
            .collect::<Vec<String>>();
        assert!(
            labels.iter().any(|label| label == "things"),
            "expected `things` among completions, got {labels:?}"
        );

        // shutdown + exit so the server `MainLoop` returns.
        write_message(
            &client_writer,
            &json!({ "jsonrpc": "2.0", "id": 3, "method": "shutdown" }),
        );
        let _ = read_response(&mut client_reader, 3).await;
        write_message(
            &client_writer,
            &json!({ "jsonrpc": "2.0", "method": "exit" }),
        );
    };

    let (server_result, ()) = block_on(async { futures::join!(server, client) });
    server_result.expect("server main loop errored");
}

/// Writes `message` to `writer` with an LSP `Content-Length` frame.
fn write_message(writer: &PipeWriter, message: &Value) {
    let body = serde_json::to_string(message).expect("serialize message");
    let framed = format!("Content-Length: {}\r\n\r\n{body}", body.len());
    writer.send(framed.into_bytes());
}

/// Reads framed messages from `reader` until one with `id` arrives, returning it.
async fn read_response(reader: &mut BufReader<PipeReader>, id: i64) -> Value {
    loop {
        let message = read_message(reader).await.expect("expected a message");
        if message["id"].as_i64() == Some(id) {
            return message;
        }
    }
}

/// Reads a single `Content-Length`-framed JSON message from `reader`.
async fn read_message(reader: &mut BufReader<PipeReader>) -> Option<Value> {
    let mut content_length = None;
    loop {
        let mut line = Vec::new();
        if reader.read_until(b'\n', &mut line).await.ok()? == 0 {
            return None;
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

    let mut body = vec![0u8; content_length?];
    reader.read_exact(&mut body).await.ok()?;
    serde_json::from_slice(&body).ok()
}

/// Returns a connected ([`PipeWriter`], [`PipeReader`]) pair.
fn byte_pipe() -> (PipeWriter, PipeReader) {
    let (tx, rx) = unbounded();
    (
        PipeWriter { tx },
        PipeReader {
            rx,
            leftover: Vec::new(),
            pos: 0,
        },
    )
}

/// Returns a connected ([`PipeWriter`], buffered [`PipeReader`]) pair.
fn client_pipe() -> (PipeWriter, BufReader<PipeReader>) {
    let (writer, reader) = byte_pipe();
    (writer, BufReader::new(reader))
}

/// Write half of an in-memory byte pipe.
#[derive(Clone)]
struct PipeWriter {
    tx: UnboundedSender<Vec<u8>>,
}

impl PipeWriter {
    fn send(&self, bytes: Vec<u8>) -> bool {
        self.tx.unbounded_send(bytes).is_ok()
    }
}

impl futures::io::AsyncWrite for PipeWriter {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        if self.send(buf.to_vec()) {
            Poll::Ready(Ok(buf.len()))
        } else {
            Poll::Ready(Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed")))
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.tx.close_channel();
        Poll::Ready(Ok(()))
    }
}

/// Read half of an in-memory byte pipe.
struct PipeReader {
    rx: UnboundedReceiver<Vec<u8>>,
    leftover: Vec<u8>,
    pos: usize,
}

impl futures::io::AsyncRead for PipeReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        if self.pos >= self.leftover.len() {
            match self.rx.poll_next_unpin(cx) {
                Poll::Ready(Some(chunk)) => {
                    self.leftover = chunk;
                    self.pos = 0;
                }
                Poll::Ready(None) => return Poll::Ready(Ok(0)),
                Poll::Pending => return Poll::Pending,
            }
        }

        let available = &self.leftover[self.pos..];
        let count = available.len().min(buf.len());
        buf[..count].copy_from_slice(&available[..count]);
        self.pos += count;
        Poll::Ready(Ok(count))
    }
}
