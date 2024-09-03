use tokio::net::{TcpListener, TcpStream};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::io::AsyncReadExt;

use crate::{Config, SharedState};
use crate::command_types::list::list_request;
use crate::helpers::helpers::send_message_to_server;

pub async fn handle_replica_connections(
    listener: TcpListener, 
    state: Arc<Mutex<SharedState>>,
    config: Config
) -> Result<(), Box<dyn std::error::Error>> {
    store_replication(&config, &state).await;
    
    // Connect to the master server
    let master_address = {
        let state_lock = state.lock().await;
        if let Some(replicaof) = state_lock.store.get("replicaof") {
            let parts: Vec<&str> = replicaof.split_whitespace().collect();
            if parts.len() == 2 {
                format!("{}:{}", parts[0], parts[1])
            } else {
                return Err("Invalid replicaof configuration".into());
            }
        } else {
            return Err("replicaof configuration is missing".into());
        }
    };

    println!("Connecting to master at: {}", master_address);
    let mut master_stream = TcpStream::connect(master_address).await?;
    println!("Connected to master server");
    
    // Establish handshake with the master
    if let Err(e) = establish_handshake(&mut master_stream).await {
        eprintln!("Failed to establish handshake with master: {}", e);
        return Err(e.into());
    }

    println!("Handling Replica Streams");
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let state_clone = state.clone();
                tokio::spawn(async move {
                    if let Err(err) = handle_connection(stream, state_clone).await {
                        eprintln!("Failed to handle connection: {}", err);
                    }
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
            }
        }
    }
}

async fn store_replication(
    config_ref: &Config, 
    ref_state: &Arc<Mutex<SharedState>>
) -> () {
    println!("Storing Replication Config");
    let mut state = ref_state.lock().await;
    state.store.insert("replicaof".to_string(), config_ref.replicaof.clone());
}

async fn handle_connection(
    stream: TcpStream,
    state: Arc<Mutex<SharedState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Handling Connection - For Replica");
    process_request(stream, state).await
}

async fn establish_handshake(
    stream: &mut TcpStream
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Establishing Replication Handshake");
    
    // Send PING message to Master
    {
        let message = "*1\r\n$4\r\nPING\r\n".to_string();
        match send_message_to_server(stream, &message, true).await {
            Ok(response) => println!("Received response: {}", response),
            Err(e) => println!("Failed to send PING message: {}", e),
        }
    }

    // Send Listening Port to Master
    {
        // let message = format!("*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n${}\r\n{}\r\n", repl_port.len(), repl_port);
        let message = "*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n$4\r\n6380\r\n".to_string();
        match send_message_to_server(stream, &message, true).await {
            Ok(response) => println!("Received Listening Port response: {}", response),
            Err(e) => println!("Failed to send Listening Port message: {}", e),
        }
    }

    // Send Capabilities to Master
    {
        let message = "*3\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\n$6\r\npsync2\r\n".to_string();
        match send_message_to_server(stream, &message, true).await {
            Ok(response) => println!("Received Capabilities response: {}", response),
            Err(e) => println!("Failed to send Capabilities message: {}", e),
        }
    }

    // Send PSYNC message to Master
    {
        let message = "*3\r\n$5\r\nPSYNC\r\n$1\r\n?\r\n$2\r\n-1\r\n".to_string();
        match send_message_to_server(stream, &message, true).await {
            Ok(response) => println!("Received PSYNC response: {}", response),
            Err(e) => println!("Failed to send PSYNC message: {}", e),
        }
    }
    println!("Finished Replication Handshake");
    return Ok(());
}

async fn process_request(
    stream: TcpStream,
    state: Arc<Mutex<SharedState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Handling process request - For Replica");
    let stream = Arc::new(Mutex::new(stream));
    let mut buf = [0; 1024];

    loop {
        println!("Waiting for data");
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
                    drop(stream_lock);
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

