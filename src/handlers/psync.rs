
use tokio::net::TcpStream;

use crate::helpers::helpers::send_message_to_server;

pub async fn handle_psync_request<'a>(stream: &'a mut TcpStream) -> () {
    println!("Handling PSYNC request");
    let message: String = "+FULLRESYNC 8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb 0\r\n".to_string();
    send_message_to_server(stream, &message).await.unwrap();
    println!("Finished handling PSYNC initial request");
    let message: String = "$0\r\n".to_string();
    send_message_to_server(stream, &message).await.unwrap();
    println!("Finished sending PSYNC file");
    return
}
    