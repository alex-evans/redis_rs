use tokio::net::{TcpListener, TcpStream};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::AsyncReadExt;

use crate::SharedState;
use crate::command_types::list::list_request;

pub async fn handle_master_connections(
    listener: TcpListener, 
    state: Arc<Mutex<SharedState>>
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Handling Master Connections");
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let state_clone = state.clone();

                tokio::spawn(async move {
                    if let Err(err) = process_request(stream, state_clone).await {
                        eprintln!("Failed to process request: {}", err);
                    }
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}

async fn process_request(
    stream: TcpStream,
    state: Arc<Mutex<SharedState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Handling process request - For Master");
    let stream = Arc::new(Mutex::new(stream));
    let mut buf = [0; 1024];

    loop {
        let mut stream_lock = stream.lock().await;
        match stream_lock.read(&mut buf).await {
            Ok(0) => {
                println!("Connection closed");
                return Ok(());
            }
            Ok(n) => {
                let request = std::str::from_utf8(&buf[..n])?;
                println!("Received request: {}", request);

                if request.starts_with('*') {
                    drop(stream_lock); // Release the lock before processing
                    list_request(&request, &state, stream.clone()).await;
                    println!("Finished processing list request");
                } else {
                    println!("Unhandled request format");
                }
            }
            Err(e) => {
                eprintln!("Failed to read from connection: {}", e);
                return Ok(());
            }
        }
    }
}