use tokio::net::{TcpListener, TcpStream};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::AsyncReadExt;

use crate::{Config, SharedState};
use crate::command_types::list::list_request;
use crate::helpers::helpers::send_message_to_server_arc;

pub async fn handle_replica_connections(
    _listener: TcpListener, 
    state: Arc<Mutex<SharedState>>,
    config: Config
) -> Result<(), Box<dyn std::error::Error>> {
    store_replication(&config, &state).await;
    if let Ok(stream) = establish_handshake(&state).await {
        process_request(stream, state).await?;
    }
    return Ok(());
}

async fn store_replication(
    config_ref: &Config, 
    ref_state: &Arc<Mutex<SharedState>>
) -> () {
    println!("Storing Replication Config");
    let mut state = ref_state.lock().await;
    state.store.insert("replicaof".to_string(), config_ref.replicaof.clone());
}

async fn establish_handshake(
    ref_state: &Arc<Mutex<SharedState>>
) -> Result<Arc<Mutex<TcpStream>>, Box<dyn std::error::Error>> {
    println!("Establishing Replication Handshake");
    
    let state = ref_state.lock().await;
    
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
                    Ok(stream)
                }
                Err(e) => {
                    println!("Failed to connect to Master: {}", e);
                    Err(Box::new(e))
                }
            }
        }
        None => {
            println!("No Master Configured");
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "No master found")))
        }
    }
}

async fn process_request(
    stream: Arc<Mutex<TcpStream>>,
    state: Arc<Mutex<SharedState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Handling process request - For Replica");
    let mut buf = [0; 1024];

    loop {
        let mut stream_lock = stream.lock().await;
        match stream_lock.read(&mut buf).await {
            Ok(0) => {
                println!("Connection closed");
                return Ok(());
            }
            Ok(n) => {
                let request = std::str::from_utf8(&buf[..n])?;
                println!("Received request: {}", request);

                if request.starts_with('*') {
                    drop(stream_lock); // Release the lock before processing
                    list_request(&request, &state, stream.clone()).await;
                    println!("Finished processing list request");
                } else {
                    println!("Unhandled request format");
                }
            }
            Err(e) => {
                eprintln!("Failed to read from connection: {}", e);
                return Ok(());
            }
        }
    }
}