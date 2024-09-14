use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::SharedState;
use crate::helpers::helpers::{
    get_next_element,
    send_message_to_server,
};

pub async fn handle_replconf_request<'a>(
    stream: Arc<Mutex<TcpStream>>, 
    lines: &'a mut std::str::Lines<'a>,
    _state: &'a Arc<Mutex<SharedState>>
) {
    println!("Handling REPLCONF request");

    let key = get_next_element(lines);
    let value = get_next_element(lines);
    let message: String;

    if key == "GETACK" {
        println!("REPLCONF - GETACK: {}", value);
        message = "*3\r\n$8\r\nREPLCONF\r\n$3\r\nACK\r\n$1\r\n0\r\n".to_string();
    } else {
        message = "+OK\r\n".to_string();
    }

    // Lock the stream to send a message
    {
        let mut stream_lock = stream.lock().await;
        if let Err(e) = send_message_to_server(&mut stream_lock, &message, false).await {
            eprintln!("Failed to send response to client: {}", e);
        }
    }

    println!("REPLCONF - Successfully sent response to client");
}