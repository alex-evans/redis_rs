use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpStream;

use crate::helpers::helpers::send_message_to_server;

pub async fn handle_ping_request<'a>(stream: Arc<Mutex<TcpStream>>) -> () {
    println!("Handling PING request");
    
    let mut stream = stream.lock().await;

    let ping_response: String = "+PONG\r\n".to_string();
    
    send_message_to_server(&mut stream, &ping_response, true).await.unwrap();
    
    println!("Sent PONG response to client");

    return
}
