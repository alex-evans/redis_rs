use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpStream;

use crate::helpers::helpers::send_message_to_server;

pub async fn handle_ping_request<'a>(stream: Arc<Mutex<TcpStream>>) {
    println!("Handling PING request");

    // Prepare the response message
    let ping_response = "+PONG\r\n".to_string();

    // Lock the stream only to send the message
    {
        let mut stream = stream.lock().await;
        if let Err(e) = send_message_to_server(&mut stream, &ping_response, false).await {
            eprintln!("Failed to send PONG response: {}", e);
        }
    }

    println!("Sent PONG response to client");
}