
use tokio::net::TcpStream;

use crate::helpers::helpers::send_message_to_server;

pub async fn handle_replconf_request<'a>(stream: &'a mut TcpStream) -> () {
    println!("Handling REPLCONF request");
    let message: String = "+OK\r\n".to_string();
    send_message_to_server(stream, &message, false).await.unwrap();
    return
}