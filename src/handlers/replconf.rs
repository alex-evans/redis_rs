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
    state: &'a Arc<Mutex<SharedState>>
) {
    println!("Handling REPLCONF request");

    let key = get_next_element(lines);
    let value = get_next_element(lines);
    let message: String;

    if key == "GETACK" {
        println!("REPLCONF - GETACK: {}", value);
        let master_repl_offset = {
            let state_lock = state.lock().await;
            match state_lock.store.get("master_repl_offset") {
                Some(offset) => offset.clone(),
                None => "0".to_string(),
            }
        };
        let length_offset = master_repl_offset.len();
        message = format!("*3\r\n$8\r\nREPLCONF\r\n$3\r\nACK\r\n${}\r\n{}\r\n", length_offset, master_repl_offset);
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