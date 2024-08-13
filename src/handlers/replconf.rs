
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
        let master_host: Option<String>;
        {
            let state = state.lock().await;
            master_host = state.store.get("replicaof").map(|full_value| {
                let parts: Vec<&str> = full_value.split(" ").collect();
                parts.get(0).unwrap_or(&"").to_string()
            });
        }

        if let Some(master_host) = master_host {
            let mut state = state.lock().await;
            state.store.insert("repl1-listening-host".to_string(), master_host);
            state.store.insert("repl1-listening-port".to_string(), sub_value);
        } else {
            println!("No Master to send replication data to");
        }
    }

    let message: String = "+OK\r\n".to_string();
    send_message_to_server(stream, &message, false).await.unwrap();
    return

}