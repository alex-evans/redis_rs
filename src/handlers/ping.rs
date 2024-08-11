use tokio::net::TcpStream;

use crate::helpers::helpers::send_message_to_server;

pub async fn handle_ping_request<'a>(stream: &'a mut TcpStream) -> () {
    println!("Handling PING request");
    let ping_response: String = "+PONG\r\n".to_string();
    send_message_to_server(stream, &ping_response, false).await.unwrap();
    println!("Sent PONG response to client");
}
