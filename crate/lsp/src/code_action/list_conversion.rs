//! A single sequence flow/block conversion code action.

use async_lsp::lsp_types::TextEdit;

/// A proposed conversion of a YAML sequence between its inline and block forms.
///
/// URI-agnostic: the language server wraps `edit` into a `WorkspaceEdit` keyed
/// by the document's URI when responding to a `textDocument/codeAction`
/// request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListConversion {
    /// Human-readable action title, e.g. `` Convert `things` to a block list
    /// ``.
    pub title: String,
    /// The single edit that rewrites the sequence in place.
    pub edit: TextEdit,
}
