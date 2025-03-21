use std::error::Error;
use serde_json::json;

use tokio::io::{ AsyncReadExt, AsyncWriteExt };
use tower_lsp::lsp_types::request::Initialize;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = tokio::net::TcpStream::connect("127.0.0.1:8080").await?;

    let initialize_request =
        json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params" : {
            "processid" : null,
            "rootUri" : null,
            "capabilities" : {}
        }
    });

    let initialize_request_str = initialize_request.to_string();

    let initialize_request_formatted: String = format!(
        "Content-Length: {}\r\n\r\n{}",
        initialize_request_str.len(),
        initialize_request_str
    );

    stream.write_all(initialize_request_formatted.as_bytes()).await?;

    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await;

    let response = String::from_utf8_lossy(&buffer[..n]);

    println!("Received initialize response: {}", response);

    let execute_command_request =
        json!({
        "jsonrpc" : "2.0",
        "id" : 2,
        "method" : "workspace/execute_command",
        "params" : {
            "command" : "custom.notification",
            "arguments" : [{
                "title" : "Hello",
                "message" : "Hello from client",
                "description" : "This is a custom notification from client"
            }]
        }
        });

    let execute_command_str = execute_command_request.to_string();

    let formatted_command_request = format!(
        "Content-Length: {}\r\n\r\n{}",
        execute_command_str.len(),
        execute_command_str
    );

    stream.write_all(formatted_command_request.as_bytes()).await?;

    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await;

    let response = String::from_utf8_lossy(&buffer[..n]);

    println!("Received execute command response: {}", response);

    Ok(())
}
