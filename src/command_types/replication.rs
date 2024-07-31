
use std::sync::{Arc, Mutex};

use crate::Config;
use crate::SharedState;
use std::net::TcpStream;
use std::io::{Read, Write};

// use crate::handlers::info::handle_info_request;


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
            let parts: Vec<&str> = full_value.split("\r\n").collect();
            let host = parts.get(0).unwrap_or(&"");
            let port = parts.get(1).unwrap_or(&"");
            let address = format!("{}:{}", host, port);
            println!("Address: {}", address);

            match TcpStream::connect(address) {
                Ok(mut stream) => {
                    let message = "*1\r\n$4\r\nPING\r\n";
                    stream.write_all(message.as_bytes()).unwrap();
                    
                    let mut response = String::new();
                    stream.read_to_string(&mut response).unwrap();
                    
                    println!("Ping response: {}", response);
                }
                Err(e) => {
                    println!("Failed to connect to {}:{}", host, port);
                    println!("Error: {}", e);
                }
            }
        }
        None => {
            println!("No Master to ping");
        }
    }
}