//! The `disposition` LSP language server.

use std::{collections::HashMap, ops::ControlFlow};

use async_lsp::{
    lsp_types::{
        CompletionOptions, CompletionParams, CompletionResponse, DidChangeTextDocumentParams,
        DidCloseTextDocumentParams, DidOpenTextDocumentParams, InitializeParams, InitializeResult,
        ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
    },
    ClientSocket, LanguageServer, ResponseError,
};
use futures::future::BoxFuture;

use crate::completion::CompletionEngine;

/// Language server providing completion for `InputDiagram` YAML.
///
/// Documents are tracked with full-text synchronization: the latest text of each
/// open document is stored and used to compute completions on demand.
pub struct DispositionLanguageServer {
    /// Socket back to the client. Held for future server-initiated messages
    /// (e.g. `textDocument/publishDiagnostics`).
    #[allow(dead_code)]
    client: ClientSocket,
    /// The latest full text of each open document, keyed by URI.
    documents: HashMap<Url, String>,
}

impl DispositionLanguageServer {
    /// Returns a new language server that can message the client over `client`.
    pub fn new(client: ClientSocket) -> Self {
        Self {
            client,
            documents: HashMap::new(),
        }
    }
}

impl LanguageServer for DispositionLanguageServer {
    type Error = ResponseError;
    type NotifyResult = ControlFlow<async_lsp::Result<()>>;

    fn initialize(
        &mut self,
        _params: InitializeParams,
    ) -> BoxFuture<'static, Result<InitializeResult, Self::Error>> {
        Box::pin(async move {
            Ok(InitializeResult {
                capabilities: ServerCapabilities {
                    text_document_sync: Some(TextDocumentSyncCapability::Kind(
                        TextDocumentSyncKind::FULL,
                    )),
                    completion_provider: Some(CompletionOptions {
                        trigger_characters: Some(vec![
                            ":".to_string(),
                            " ".to_string(),
                            "-".to_string(),
                        ]),
                        ..CompletionOptions::default()
                    }),
                    ..ServerCapabilities::default()
                },
                server_info: None,
            })
        })
    }

    fn completion(
        &mut self,
        params: CompletionParams,
    ) -> BoxFuture<'static, Result<Option<CompletionResponse>, Self::Error>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let items = self
            .documents
            .get(&uri)
            .map(|text| CompletionEngine::completions(text, position.line, position.character))
            .unwrap_or_default();

        Box::pin(async move { Ok(Some(CompletionResponse::Array(items))) })
    }

    fn did_open(&mut self, params: DidOpenTextDocumentParams) -> Self::NotifyResult {
        self.documents
            .insert(params.text_document.uri, params.text_document.text);
        ControlFlow::Continue(())
    }

    fn did_change(&mut self, params: DidChangeTextDocumentParams) -> Self::NotifyResult {
        // Full synchronization: the last change carries the entire document text.
        if let Some(change) = params.content_changes.into_iter().next_back() {
            self.documents.insert(params.text_document.uri, change.text);
        }
        ControlFlow::Continue(())
    }

    fn did_close(&mut self, params: DidCloseTextDocumentParams) -> Self::NotifyResult {
        self.documents.remove(&params.text_document.uri);
        ControlFlow::Continue(())
    }
}
