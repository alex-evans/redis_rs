
use std::sync::{Arc, Mutex};
use std::net::TcpStream;

use crate::Config;
use crate::SharedState;
use crate::helpers::helpers::send_message_to_server;

pub fn store_replication(config_ref: &Config, ref_state: &Arc<Mutex<SharedState>>) -> () {
    println!("Storing Replication Config");
    let mut state = ref_state.lock().unwrap();
    state.store.insert("replicaof".to_string(), config_ref.replicaof.clone());
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
            match TcpStream::connect(&address) {
                Ok(mut stream) => {

                    // Send PING message to Master
                    let message = "*1\r\n$4\r\nPING\r\n".to_string();
                    let msg_response = send_message_to_server(&mut stream, &message).unwrap();
                    println!("Received PING response: {}", msg_response);

                    // Send Listening Port to Master
                    let message = format!("*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n${}\r\n{}\r\n", repl_port.len(), repl_port);
                    let msg_response: String = send_message_to_server(&mut stream, &message).unwrap();
                    println!("Received Listening Port response: {}", msg_response);

                    // Send Capabilities to Master
                    let message = "*3\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\n$6\r\npsync2\r\n".to_string();
                    let msg_response: String = send_message_to_server(&mut stream, &message).unwrap();
                    println!("Received Capabilities response: {}", msg_response);

                }
                Err(e) => {
                    println!("Failed to connect to Master: {}", e);
                }
            }

        }
        None => {
            println!("No Master to send replication data to");
        }
    }
}