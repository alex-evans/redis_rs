
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::helpers::helpers::{
    get_next_element,
    send_message_to_server
};
use crate::SharedState;

pub async fn handle_replconf_request<'a>(
    stream: &'a mut TcpStream, 
    lines: &'a mut std::str::Lines<'a>,
    state: &'a Arc<Mutex<SharedState>>
) -> () {

    println!("Handling REPLCONF request");
    let sub_command: String = get_next_element(lines);
    let sub_value: String = get_next_element(lines);

    if sub_command.to_uppercase() == "LISTENING-PORT" {
        let current_host = "127.0.0.1".to_string(); 
        let mut state = state.lock().await;
        state.store.insert("repl1-listening-host".to_string(), current_host);
        state.store.insert("repl1-listening-port".to_string(), sub_value);
    }

    let message: String = "+OK\r\n".to_string();
    send_message_to_server(stream, &message, false).await.unwrap();
    return

}