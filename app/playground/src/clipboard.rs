//! Clipboard write helper that works in non-secure browser contexts.
//!
//! The browser's async Clipboard API (`navigator.clipboard`) is only available
//! in "secure contexts" -- pages served over HTTPS or from `localhost`. When
//! the playground is served over plain HTTP from a LAN IP (e.g.
//! `http://192.168.88.6:8080`), `navigator.clipboard` is `undefined`, so
//! calling `navigator.clipboard.writeText(...)` throws:
//!
//! ```text
//! can't access property "writeText", arg0 is undefined
//! ```
//!
//! [`Clipboard::clipboard_text_set`] uses `navigator.clipboard` when it is
//! available, and otherwise falls back to a temporary `<textarea>` plus the
//! deprecated `document.execCommand("copy")`, which works in non-secure
//! contexts.

use dioxus::document;

pub use self::clipboard_error::ClipboardError;

mod clipboard_error;

/// Writes text to the system clipboard from the browser.
pub struct Clipboard;

/// JavaScript that reads the text to copy from the eval channel, writes it to
/// the clipboard (with a non-secure-context fallback), then sends back `"ok"`
/// on success or an `"error: ..."` message on failure.
///
/// The text is passed in via `dioxus.recv()` rather than being interpolated
/// into the snippet, so arbitrary content (quotes, newlines, backslashes in
/// the generated SVG) is handled safely without manual escaping.
const JS_CLIPBOARD_TEXT_SET: &str = r#"
(async () => {
    let text = await dioxus.recv();

    // === Preferred path: async Clipboard API (secure contexts only) === //
    if (window.isSecureContext && navigator.clipboard && navigator.clipboard.writeText) {
        try {
            await navigator.clipboard.writeText(text);
            dioxus.send("ok");
            return;
        } catch (e) {
            // Fall through to the `execCommand` fallback below.
        }
    }

    // === Fallback: temporary <textarea> + execCommand("copy") === //
    // Works on pages served over plain HTTP from a non-localhost host, where
    // `navigator.clipboard` is unavailable.
    try {
        const textarea = document.createElement("textarea");
        textarea.value = text;
        textarea.setAttribute("readonly", "");
        textarea.style.position = "fixed";
        textarea.style.top = "-9999px";
        textarea.style.left = "-9999px";
        document.body.appendChild(textarea);

        // Preserve any existing selection so copying does not disrupt the user.
        const selection = document.getSelection();
        const previousRange =
            selection && selection.rangeCount > 0 ? selection.getRangeAt(0) : null;

        textarea.select();
        const ok = document.execCommand("copy");
        document.body.removeChild(textarea);

        if (previousRange && selection) {
            selection.removeAllRanges();
            selection.addRange(previousRange);
        }

        dioxus.send(ok ? "ok" : "error: execCommand returned false");
    } catch (e) {
        dioxus.send("error: " + (e && e.message ? e.message : String(e)));
    }
})()
"#;

impl Clipboard {
    /// Copies `text` to the system clipboard.
    ///
    /// Uses the async Clipboard API when running in a secure context, and falls
    /// back to `document.execCommand("copy")` otherwise.
    ///
    /// Returns a [`ClipboardError`] if the text could not be written.
    pub async fn clipboard_text_set(text: String) -> Result<(), ClipboardError> {
        let mut eval = document::eval(JS_CLIPBOARD_TEXT_SET);
        eval.send(text)
            .map_err(|error| ClipboardError::Send(error.to_string()))?;
        let result = eval
            .recv::<String>()
            .await
            .map_err(|error| ClipboardError::Recv(error.to_string()))?;

        if result == "ok" {
            Ok(())
        } else {
            Err(ClipboardError::CopyFailed(result))
        }
    }
}
