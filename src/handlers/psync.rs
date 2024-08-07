
use tokio::net::TcpStream;

use crate::helpers::helpers::send_message_to_server;
use std::fs;
use base64;

pub async fn handle_psync_request<'a>(stream: &'a mut TcpStream) -> () {
    println!("Handling PSYNC request");
    let message: String = "+FULLRESYNC 8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb 0\r\n".to_string();
    send_message_to_server(stream, &message).await.unwrap();
    println!("Finished handling PSYNC initial request");

    let file_path = "data/fake.rdb";
    let file_contents = fs::read(file_path).unwrap();
    let file_length = file_contents.len();
    let binary_data = base64::decode(&file_contents).unwrap();
    let message = format!("${}\r\n{}", file_length, String::from_utf8_lossy(&binary_data));

    // Send the message to the server
    send_message_to_server(stream, &message).await.unwrap();

    println!("Finished sending PSYNC file");
    return
}
    