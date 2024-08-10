
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::helpers::helpers::send_message_to_server;

pub async fn handle_replconf_request<'a>(stream: Arc<Mutex<TcpStream>>) -> () {
    println!("Handling REPLCONF request");
    let message: String = "+OK\r\n".to_string();
    send_message_to_server(stream, &message, false).await.unwrap();
    return
}