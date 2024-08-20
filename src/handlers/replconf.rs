use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::helpers::helpers::send_message_to_server;
use crate::SharedState;

pub async fn handle_replconf_request<'a>(
    stream: Arc<Mutex<TcpStream>>, 
    _lines: &'a mut std::str::Lines<'a>,
    _state: &'a Arc<Mutex<SharedState>>
) {
    println!("Handling REPLCONF request");

    // Lock the stream to send a message
    {
        let mut stream_lock = stream.lock().await;
        let message = "+OK\r\n".to_string();
        if let Err(e) = send_message_to_server(&mut stream_lock, &message, false).await {
            eprintln!("Failed to send response to client: {}", e);
        }
    }

    println!("REPLCONF - Successfully sent response to client");
}