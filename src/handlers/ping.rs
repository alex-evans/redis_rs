
use tokio::net::TcpStream;

use crate::helpers::helpers::send_message_to_server;

pub async fn handle_ping_request(stream: &mut TcpStream) -> () {
    let ping_response = "+PONG\r\n".to_string();
    send_message_to_server(stream, &ping_response).await.unwrap();
    println!("Sent PONG response to client");
    return
}
