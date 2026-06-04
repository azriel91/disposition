//! In-memory byte pipe bridging the editor's message transport to the LSP
//! `MainLoop`'s framed byte streams.
//!
//! [`async_lsp::MainLoop`] reads and writes `Content-Length`-framed bytes over
//! [`AsyncRead`] / [`AsyncWrite`], whereas the editor's LSP client exchanges
//! whole JSON-RPC messages. A [`byte_pipe`] connects the two: one pipe carries
//! editor -> server bytes (written framed by the server glue, read by the
//! `MainLoop`), the other carries server -> editor bytes (written by the
//! `MainLoop`, drained and unframed back into messages).

use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    io::{AsyncRead, AsyncWrite},
    StreamExt,
};

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

impl AsyncWrite for PipeWriter {
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
