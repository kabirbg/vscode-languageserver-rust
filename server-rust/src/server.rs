use serde::{
    Serialize,
    Deserialize
};
use tower_lsp::{
    async_trait,
    lsp_types::{ ExecuteCommandOptions, InitializeParams, InitializeResult, ServerCapabilities },
    LspService,
    Server,
};

#[derive(Debug, Serialize, Deserialize)]
struct NotificationParams {
    title: String,
    message: String,
    description: String,
}

enum CustomNotification {}

impl Notification for CustomNotification {
    type Params = NotificationParams;

    const METHOD: &'static str = "custom/notification";
}

#[derive(Debug)]
struct Backend {
    client: Client,
}

#[async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                execute_command_provider = Some(ExecuteCommandOptions {
                    commands: vec![String::from("custom.notification")],
                    work_done_progress_options: Default::default(),
                }),
                ..ServerCapabilities::default()
            },
            ..Default::default()
        })
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<Value>> {
        if params.command == "custom.notification" {
            self.client.send_notification::<CustomNotification>(NotificationParams {
                title: String::from("Hello Notification"),
                message: String::from("This is a test message"),
                description: String::from("This is a description"),
            }).await;

            self.client.log_message(
                MessageType::INFO,
                format!("Command executed Successfully with params : {params:?}")
            ).await;

            Ok(None)
        } else {
            Err(Error::invalid_request())
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();

    let (stream, _) = listener.accept().await.unwrap();

    let (read, write) = tokio::io::split(stream);

    let (service, socket) = LspService::new(|client: Client| Backend {
        client,
    });

    Server::new(read, write).serve(service).await;
}