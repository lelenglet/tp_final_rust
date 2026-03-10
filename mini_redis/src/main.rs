mod handler;
mod model;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

use crate::handler::{process_command, Store};
use crate::model::Request;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    // Initialiser tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let store: Store = Arc::new(tokio::sync::RwLock::new(HashMap::new()));

    let addr = "127.0.0.1:7878";
    let listener = TcpListener::bind(addr).await?;
    println!("MiniRedis écoute sur {}", addr);

    loop {
        let (socket, _) = listener.accept().await?;
        let store_clone = Arc::clone(&store);
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, store_clone).await {
                eprintln!("Erreur client: {}", e);
            }
        });
    }
}

async fn handle_client(mut socket: TcpStream, store: Store) -> tokio::io::Result<()> {
    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        if reader.read_line(&mut line).await? == 0 {
            break;
        }

        let response = match serde_json::from_str::<Request>(&line) {
            Ok(req) => process_command(req, &store).await,
            Err(_) => crate::model::Response::error("invalid json"),
        };

        let mut resp_json = serde_json::to_string(&response).unwrap();
        resp_json.push('\n');
        writer.write_all(resp_json.as_bytes()).await?;
    }
    Ok(())
}
