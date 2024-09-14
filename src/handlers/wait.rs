use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpStream;

use crate::helpers::helpers::send_message_to_server;

pub async fn handle_wait_request<'a>(
    stream: Arc<Mutex<TcpStream>>,
) {
    println!("Handling WAIT request");
    
    let wait_response = ":0\r\n".to_string();

    // Lock the stream only to send the message
    {
        let mut stream = stream.lock().await;
        if let Err(e) = send_message_to_server(&mut stream, &wait_response, false).await {
            eprintln!("Failed to send WAIT response: {}", e);
        }
    }

    println!("Sent WAIT response to client");
}