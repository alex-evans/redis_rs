use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpStream;

use crate::helpers::helpers::send_message_to_server;
use crate::SharedState;

pub async fn handle_wait_request<'a>(
    stream: Arc<Mutex<TcpStream>>,
    state: &'a Arc<Mutex<SharedState>>
) {
    println!("Handling WAIT request");
    
    let current_number_of_replicas = {
        let state_lock = state.lock().await;
        match state_lock.store.get("number_of_replicas") {
            Some(replicas) => replicas.clone(),
            None => "0".to_string(),
        }
    };

    let wait_response = format!(":{}\r\n", current_number_of_replicas);

    // Lock the stream only to send the message
    {
        let mut stream = stream.lock().await;
        if let Err(e) = send_message_to_server(&mut stream, &wait_response, false).await {
            eprintln!("Failed to send WAIT response: {}", e);
        }
    }

    println!("Sent WAIT response to client");
}