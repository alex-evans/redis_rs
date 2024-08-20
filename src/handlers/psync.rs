use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use std::fs;
use std::path::Path;

use crate::helpers::helpers::send_message_to_server;
use crate::SharedState;

pub async fn handle_psync_request<'a>(
    stream: Arc<Mutex<TcpStream>>,
    state: &'a Arc<Mutex<SharedState>>
) {
    println!("Handling PSYNC request");

    // Lock the stream for sending the initial response
    {
        let mut stream = stream.lock().await;
        let message = "+FULLRESYNC 8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb 0\r\n";
        if let Err(e) = send_message_to_server(&mut stream, message, false).await {
            eprintln!("Failed to send initial PSYNC response: {}", e);
            return;
        }
        println!("Finished handling PSYNC initial request");
    } // Release lock

    let file_path = "data/fake.rdb";
    if !Path::new(file_path).exists() {
        println!("RDB file not found at path: {}", file_path);
        return;
    }

    let file_contents = match fs::read(file_path) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("Failed to read RDB file: {}", e);
            return;
        }
    };

    let rdb_file = hex::decode(&file_contents).unwrap();

    // If the file is binary, no need to decode it
    let rdb_length = rdb_file.len();

    // Lock the stream for sending the RDB file
    {
        let mut stream = stream.lock().await;
        let message = format!("${}\r\n", rdb_length);
        if let Err(e) = send_message_to_server(&mut stream, &message, false).await {
            eprintln!("Failed to send RDB length: {}", e);
            return;
        }

        if let Err(e) = stream.write_all(&rdb_file).await {
            eprintln!("Failed to send RDB file: {}", e);
            return;
        }

        if let Err(e) = stream.flush().await {
            eprintln!("Failed to flush stream: {}", e);
        }
    } // Release lock

    let mut receiver = {
        let state_guard = state.lock().await;
        state_guard.sender.subscribe()
    };

    // Clone the Arc to move into the task
    let stream_clone = Arc::clone(&stream);

    while let Ok(message) = receiver.recv().await {
        println!("Received message from sender: {}", message);
        let mut stream_lock = stream_clone.lock().await;
        if let Err(e) = send_message_to_server(&mut stream_lock, &message, false).await {
            eprintln!("Failed to send message to client: {}", e);
        }
    }

    println!("Finished sending PSYNC file");
}