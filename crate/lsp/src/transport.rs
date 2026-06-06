//! In-memory byte-stream transport bridging a per-message LSP client to the
//! framed byte streams an [`async_lsp::MainLoop`] reads and writes.
//!
//! The `MainLoop` speaks `Content-Length`-framed bytes over [`AsyncRead`] /
//! [`AsyncWrite`]; an editor's LSP client exchanges whole JSON-RPC messages. A
//! [`byte_pipe`] connects the two halves, and [`frame`] / [`read_message`]
//! translate between framed bytes and individual JSON messages. This is shared
//! by every host -- the in-page server, the Web Worker server, and tests.

use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    io::{AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt},
    StreamExt,
};

/// Sentinel message a Web Worker host posts to the main thread once its wasm
/// has initialized and is ready to receive LSP messages.
///
/// The main thread queues outgoing messages until it sees this, then flushes
/// them -- the editor's first `initialize` request is typically sent before the
/// worker has finished loading.
pub const WORKER_READY: &str = "__disposition_lsp_worker_ready__";

/// Returns a connected ([`PipeWriter`], [`PipeReader`]) pair backed by an
/// unbounded channel of byte chunks.
pub fn byte_pipe() -> (PipeWriter, PipeReader) {
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

/// Wraps a single JSON-RPC message in an LSP `Content-Length` frame.
pub fn frame(json: &str) -> Vec<u8> {
    format!("Content-Length: {}\r\n\r\n{json}", json.len()).into_bytes()
}

/// Reads one `Content-Length`-framed JSON message from `reader`.
///
/// Returns `None` at end of stream or on a malformed frame.
pub async fn read_message<R>(reader: &mut R) -> Option<String>
where
    R: AsyncBufRead + Unpin,
{
    let mut content_length = None;
    loop {
        let mut line = Vec::new();
        match reader.read_until(b'\n', &mut line).await {
            Ok(0) => return None, // End of stream.
            Ok(_) => {}
            Err(_) => return None,
        }

        let header = String::from_utf8_lossy(&line);
        let header = header.trim_end_matches(['\r', '\n']);
        if header.is_empty() {
            break; // End of headers.
        }
        if let Some(value) = header.strip_prefix("Content-Length:") {
            content_length = value.trim().parse::<usize>().ok();
        }
    }

    let mut body = vec![0u8; content_length?];
    reader.read_exact(&mut body).await.ok()?;
    Some(String::from_utf8_lossy(&body).into_owned())
}

/// Write half of an in-memory byte pipe.
#[derive(Clone)]
pub struct PipeWriter {
    /// Sends written byte chunks to the paired [`PipeReader`].
    tx: UnboundedSender<Vec<u8>>,
}

impl PipeWriter {
    /// Queues `bytes` to the reader synchronously.
    ///
    /// The channel is unbounded, so this never blocks. Returns `false` if the
    /// reader has been dropped.
    pub fn send_bytes(&self, bytes: Vec<u8>) -> bool {
        self.tx.unbounded_send(bytes).is_ok()
    }
}

impl futures::io::AsyncWrite for PipeWriter {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        if self.send_bytes(buf.to_vec()) {
            Poll::Ready(Ok(buf.len()))
        } else {
            Poll::Ready(Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "LSP pipe reader dropped",
            )))
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
pub struct PipeReader {
    /// Receives byte chunks from the paired [`PipeWriter`].
    rx: UnboundedReceiver<Vec<u8>>,
    /// The current chunk being drained into reads.
    leftover: Vec<u8>,
    /// Read offset into `leftover`.
    pos: usize,
}

impl AsyncRead for PipeReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        // Refill from the channel when the current chunk is exhausted.
        if self.pos >= self.leftover.len() {
            match self.rx.poll_next_unpin(cx) {
                Poll::Ready(Some(chunk)) => {
                    self.leftover = chunk;
                    self.pos = 0;
                }
                // All writers dropped: end of stream.
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
