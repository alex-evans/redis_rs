use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::helpers::helpers::send_message_to_server_arc;

pub async fn handle_ping_request(stream: Arc<Mutex<TcpStream>>) -> () {
    println!("Handling PING request");
    let ping_response: String = "+PONG\r\n".to_string();
    send_message_to_server_arc(stream, &ping_response, false).await.unwrap();
    println!("Sent PONG response to client");
}
