
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::helpers::helpers::{get_next_element, send_message_to_server};
use crate::SharedState;

pub async fn handle_replconf_request<'a>(
    stream: Arc<Mutex<TcpStream>>, 
    lines: &'a mut std::str::Lines<'a>,
    state: &'a Arc<Mutex<SharedState>>
) -> () {
    println!("Handling REPLCONF request");
    
    let sub_command: String = get_next_element(lines);
    // let sub_value: String = get_next_element(lines);

    if sub_command.to_uppercase() == "LISTENING-PORT" {
        let mut receiver = {
            let state_guard = state.lock().await;
            state_guard.sender.subscribe()
        };

        tokio::spawn(async move {
            loop {
                let message = receiver.recv().await.unwrap();
                println!("Received message: {}", message);
            }
        });
    }

    // Lock the stream to send a message
    {
        let mut stream_lock = stream.lock().await;
        let message: String = "+OK\r\n".to_string();
        send_message_to_server(&mut stream_lock, &message, false).await.unwrap();
    }

    println!("REPLCONF - Successfully sent response to client");

    return

}