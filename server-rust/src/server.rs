use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::lsp_types::*;
use tower_lsp_server::{ Client, LanguageServer, LspService, Server };
use tokio::fs;
use anyhow::Context;

#[derive(Debug)]
struct Backend {
    client: Client,
    dictionary: Vec<String>,
}

impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "language-server-rust".to_string(),
                version: Some("0.0.1".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(
                    TextDocumentSyncCapability::Kind(TextDocumentSyncKind::INCREMENTAL)
                ),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    ..Default::default()
                }),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["dummy.do_something".to_string()],
                    work_done_progress_options: Default::default(),
                }),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "initialized!").await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn execute_command(&self, _: ExecuteCommandParams) -> Result<Option<LSPAny>> {
        self.client.log_message(MessageType::INFO, "command executed!").await;

        match self.client.apply_edit(WorkspaceEdit::default()).await {
            Ok(res) if res.applied => self.client.log_message(MessageType::INFO, "applied").await,
            Ok(_) => self.client.log_message(MessageType::INFO, "rejected").await,
            Err(err) => self.client.log_message(MessageType::ERROR, err).await,
        }

        Ok(None)
    }

    async fn did_open(&self, _: DidOpenTextDocumentParams) {
        self.client.log_message(MessageType::INFO, "file opened!").await;
    }

    async fn did_change(&self, _: DidChangeTextDocumentParams) {
        self.client.log_message(MessageType::INFO, "file changed!").await;
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {
        self.client.log_message(MessageType::INFO, "file saved!").await;
    }

    async fn did_close(&self, _: DidCloseTextDocumentParams) {
        self.client.log_message(MessageType::INFO, "file closed!").await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let prefix = params.context
            .as_ref()
            .and_then(|ctx| ctx.trigger_character.as_deref())
            .unwrap_or_default();

        let items = self.dictionary
            .iter()
            .filter(|word| word.starts_with(prefix))
            .take(100)
            .map(|word| CompletionItem::new_simple(word.clone(), word.clone()))
            .collect();

        Ok(Some(CompletionResponse::Array(items)))
    }
}

async fn load_word_list(path: impl AsRef<std::path::Path>) -> anyhow::Result<Vec<String>> {
    let content = fs
        ::read_to_string(&path).await
        .with_context(|| format!("Failed to read word list from {}", path.as_ref().display()))?;

    Ok(
        content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect()
    )
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

    let dictionary = load_word_list("/tmp/keywords.dict").await.unwrap_or_else(|e| {
        eprintln!("Error loading word list: {e}");
        Vec::new()
    });

    let (service, socket) = LspService::new(|client| Backend { client, dictionary });
    Server::new(stdin, stdout, socket).serve(service).await;
}
