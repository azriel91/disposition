use std::fmt;

/// Errors that can occur when writing text to the clipboard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClipboardError {
    /// The text to copy could not be sent into the JS eval context.
    ///
    /// The inner `String` is the underlying `EvalError` message.
    Send(String),
    /// The result could not be received back from the JS eval context.
    ///
    /// The inner `String` is the underlying `EvalError` message.
    Recv(String),
    /// The browser reported that the copy operation failed.
    ///
    /// The inner `String` is the message reported by the JS snippet, e.g.
    /// `"error: execCommand returned false"`.
    CopyFailed(String),
}

impl fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Send(message) => {
                write!(f, "Failed to send clipboard text to JS: {message}")
            }
            Self::Recv(message) => {
                write!(f, "Failed to receive clipboard result from JS: {message}")
            }
            Self::CopyFailed(message) => {
                write!(f, "Failed to write text to clipboard: {message}")
            }
        }
    }
}

impl std::error::Error for ClipboardError {}
