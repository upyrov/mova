use dashmap::DashMap;
use mova::{error::MovaError, lexer::tokenize, parser::node::parse};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
    document_map: DashMap<String, String>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Mova LSP server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("File opened: {}", params.text_document.uri),
            )
            .await;
        
        let uri = params.text_document.uri.to_string();
        let text = params.text_document.text;
        
        self.document_map.insert(uri.clone(), text.clone());
        self.validate_document(params.text_document.uri, text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().last() {
            let uri = params.text_document.uri.to_string();
            let text = change.text;
            
            self.document_map.insert(uri.clone(), text.clone());
            self.validate_document(params.text_document.uri, text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.document_map.remove(&params.text_document.uri.to_string());
    }
}

impl Backend {
    async fn validate_document(&self, uri: Url, text: String) {
        let mut diagnostics = Vec::new();

        match tokenize(&text) {
            Ok(tokens) => {
                if let Err(MovaError::Parser { error: e, position }) = parse(tokens) {
                    let pos = Position::new(position.line as u32 - 1, position.character as u32);
                    diagnostics.push(Diagnostic {
                        range: Range::new(pos, pos),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: e.to_string(),
                        ..Default::default()
                    });
                }
            }
            Err(MovaError::Lexer { character, position }) => {
                let pos = Position::new(position.line as u32 - 1, position.character as u32);
                diagnostics.push(Diagnostic {
                    range: Range::new(pos, pos),
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("Unexpected character: '{character}'"),
                    ..Default::default()
                });
            }
            _ => {}
        }

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        document_map: DashMap::new(),
    });
    
    Server::new(stdin, stdout, socket).serve(service).await;
}
