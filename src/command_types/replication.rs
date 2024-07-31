
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
            let parts: Vec<&str> = full_value.split(" ").collect();
            let host = parts.get(0).unwrap_or(&"");
            let port = parts.get(1).unwrap_or(&"");
            let address = format!("{}:{}", host, port);
            println!("Ping - Connecting to Master: {}", address);

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

pub fn send_replication_data(ref_config: &Config, ref_state: &Arc<Mutex<SharedState>>) -> () {
    println!("Sending Replication Data");
    let state = ref_state.lock().unwrap();
    let repl_port: &str = &ref_config.port;

    match state.store.get("replicaof") {
        Some(full_value) => {
            let parts: Vec<&str> = full_value.split(" ").collect();
            let master_host = parts.get(0).unwrap_or(&"");
            let master_port = parts.get(1).unwrap_or(&"");
            let address = format!("{}:{}", master_host, master_port);
            println!("Connecting to Master: {}", address);

            match TcpStream::connect(address) {
                Ok(mut stream) => {
                    let message = format!("*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n${}\r\n{}\r\n", repl_port.len(), repl_port);
                    stream.write_all(message.as_bytes()).unwrap();
                    
                    let mut response = String::new();
                    stream.read_to_string(&mut response).unwrap();
                    
                    println!("Replication response: {}", response);
                }
                Err(e) => {
                    println!("Failed to connect to {}:{}", master_host, master_port);
                    println!("Error: {}", e);
                }
            }

            // match TcpStream::connect(address) {
            //     Ok(mut stream) => {
            //         let message = "*3\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\npsync2\r\n".to_string();
            //         stream.write_all(message.as_bytes()).unwrap();
                    
            //         let mut response = String::new();
            //         stream.read_to_string(&mut response).unwrap();
                    
            //         println!("Replication response: {}", response);
            //     }
            //     Err(e) => {
            //         println!("Failed to connect to {}:{}", master_host, master_port);
            //         println!("Error: {}", e);
            //     }
            // }
        }
        None => {
            println!("No Master to send replication data to");
        }
    }
}