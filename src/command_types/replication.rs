
use std::sync::{Arc, Mutex};

use crate::Config;
use crate::SharedState;
use crate::helpers::helpers::send_message_to_server;

pub fn store_replication(config_ref: &Config, ref_state: &Arc<Mutex<SharedState>>) -> () {
    println!("Storing Replication Config");
    let mut state = ref_state.lock().unwrap();
    state.store.insert("replicaof".to_string(), config_ref.replicaof.clone());
}

pub fn ping_master(ref_state: &Arc<Mutex<SharedState>>) -> () {
    println!("Pinging Master");
    let state = ref_state.lock().unwrap();
    match state.store.get("replicaof") {
        Some(full_value) => {
            let parts: Vec<&str> = full_value.split(" ").collect();
            let host = parts.get(0).unwrap_or(&"");
            let port = parts.get(1).unwrap_or(&"");
            let address = format!("{}:{}", host, port);
            println!("Ping - Connecting to Master: {}", address);
            let message = "*1\r\n$4\r\nPING\r\n".to_string();
            send_message_to_server(&address, message);
        }
        None => {
            println!("No Master to ping");
        }
    }
}

pub fn send_replication_data(ref_config: &Config, ref_state: &Arc<Mutex<SharedState>>) -> () {
    println!("Sending Replication Data");
    let state = ref_state.lock().unwrap();
    let repl_port: &str = &ref_config.port;

    println!("Replicaof: {:?}", state.store.get("replicaof"));
    match state.store.get("replicaof") {
        Some(full_value) => {
            let parts: Vec<&str> = full_value.split(" ").collect();
            let master_host = parts.get(0).unwrap_or(&"");
            let master_port = parts.get(1).unwrap_or(&"");
            let address = format!("{}:{}", master_host, master_port);
            println!("Connecting to Master: {}", address);
            
            // First Message: Listening Port
            let message = format!("*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n${}\r\n{}\r\n", repl_port.len(), repl_port);
            send_message_to_server(&address, message);

            // Second Message: Capabilities
            let message = "*3\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\npsync2\r\n".to_string();
            send_message_to_server(&address, message);
        }
        None => {
            println!("No Master to send replication data to");
        }
    }
}