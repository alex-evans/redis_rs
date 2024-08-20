
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpStream;

use crate::Config;
use crate::SharedState;
use crate::helpers::helpers::send_message_to_server_arc;

pub async fn store_replication(config_ref: &Config, ref_state: &Arc<Mutex<SharedState>>) -> () {
    println!("Storing Replication Config");
    let mut state = ref_state.lock().await;
    state.store.insert("replicaof".to_string(), config_ref.replicaof.clone());
}

pub async fn send_replication_data(_ref_config: &Config, ref_state: &Arc<Mutex<SharedState>>) -> () {
    println!("Sending Replication Data");
    let state = ref_state.lock().await;
    // let repl_port: &str = &ref_config.port;

    println!("Replicaof: {:?}", state.store.get("replicaof"));
    match state.store.get("replicaof") {
        Some(full_value) => {
            let parts: Vec<&str> = full_value.split(" ").collect();
            let master_host = parts.get(0).unwrap_or(&"");
            let master_port = parts.get(1).unwrap_or(&"");
            let address: String = format!("{}:{}", master_host, master_port);
            
            println!("Connecting to Master: {}", address);
            match TcpStream::connect(&address).await {
                Ok(stream) => {
                    let stream = Arc::new(Mutex::new(stream));
                    
                    // Send PING message to Master
                    {
                        let message = "*1\r\n$4\r\nPING\r\n".to_string();
                        match send_message_to_server_arc(stream.clone(), &message, true).await {
                            Ok(Some(response)) => println!("Received response: {}", response),
                            Ok(None) => println!("Received no response"),
                            Err(e) => println!("Failed to send PING message: {}", e),
                        }
                    }

                    // Send Listening Port to Master
                    {
                        // let message = format!("*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n${}\r\n{}\r\n", repl_port.len(), repl_port);
                        let message = "*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n$4\r\n6380\r\n".to_string();
                        match send_message_to_server_arc(stream.clone(), &message, true).await {
                            Ok(Some(response)) => println!("Received Listening Port response: {}", response),
                            Ok(None) => println!("Received no response for Listening Port"),
                            Err(e) => println!("Failed to send Listening Port message: {}", e),
                        }
                    }

                    // Send Capabilities to Master
                    {
                        let message = "*3\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\n$6\r\npsync2\r\n".to_string();
                        match send_message_to_server_arc(stream.clone(), &message, true).await {
                            Ok(Some(response)) => println!("Received Capabilities response: {}", response),
                            Ok(None) => println!("Received no response for Capabilities"),
                            Err(e) => println!("Failed to send Capabilities message: {}", e),
                        }
                    }

                    // Send PSYNC message to Master
                    {
                        let message = "*3\r\n$5\r\nPSYNC\r\n$1\r\n?\r\n$2\r\n-1\r\n".to_string();
                        match send_message_to_server_arc(stream.clone(), &message, true).await {
                            Ok(Some(response)) => println!("Received PSYNC response: {}", response),
                            Ok(None) => println!("Received no response for PSYNC"),
                            Err(e) => println!("Failed to send PSYNC message: {}", e),
                        }
                    }
                
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