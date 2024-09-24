use bytes::BytesMut;
use tokio::net::{TcpListener, TcpStream};
use std::sync::Arc;
use tokio::sync::Mutex;
// use std::os::unix::io::{AsRawFd, FromRawFd};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{Config, SharedState};
use crate::command_types::list::list_request;
// use crate::command_types::bulk_string::bulk_string_request;
// use crate::helpers::helpers::send_message_to_server;

pub async fn handle_replica_connections(
    listener: TcpListener, 
    state: Arc<Mutex<SharedState>>,
    config: Config
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    store_replication(&config, &state).await;
    
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

    println!("Master address: {}", master_address);

    // Spawn a task to handle master connection and handshake
    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Err(e) = handle_master_connection(&master_address, state_clone).await {
            eprintln!("Error processing master connection: {:?}", e);
        }
    });

    // Continue listening for client connections
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let state_clone = state.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, state_clone).await {
                        eprintln!("Error handling client: {:?}", e);
                    }
                });
            }
            Err(e) => eprintln!("Error accepting connection: {:?}", e),
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

async fn handle_master_connection(
    master_address: &str,
    state: Arc<Mutex<SharedState>>
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let master_stream = connect_and_handshake(master_address).await?;
    let stream: Arc<Mutex<TcpStream>> = Arc::new(Mutex::new(master_stream));
    
    let mut buffer = Vec::new();
    let mut rdb_processed = false;

    loop {
        let mut chunk = [0u8; 1024];
        let mut stream_lock = stream.lock().await;
        match stream_lock.read(&mut chunk).await {
            Ok(0) => {
                println!("Master connection closed");
                return Ok(());
            }
            Ok(n) => {
                buffer.extend_from_slice(&chunk[..n]);
                drop(stream_lock); // Release the lock before processing

                while !buffer.is_empty() {
                    if !rdb_processed {
                        if buffer.starts_with(b"$") {
                            // Process RDB file
                            if let Some(end) = buffer.windows(2).position(|w| w == b"\r\n") {
                                let length: usize = std::str::from_utf8(&buffer[1..end])
                                    .unwrap()
                                    .parse()
                                    .unwrap();
                                if buffer.len() >= end + 2 + length + 2 {
                                    println!("Received RDB file of length {}", length);
                                    // Skip the RDB data
                                    buffer = buffer[end + 2 + length + 2..].to_vec();
                                    rdb_processed = true;
                                    println!("RDB processing complete. Remaining buffer length: {}", buffer.len());
                                } else {
                                    break; // Incomplete data, wait for more
                                }
                            } else {
                                break; // Incomplete length, wait for more data
                            }
                        } else {
                            // Unexpected data before RDB file
                            buffer.remove(0);
                        }
                    } else {
                        match parse_resp_message(&buffer) {
                            Ok((command, rest, bytes_processed)) => {
                                println!("Received command: {:?}", command);
                                if !command.is_empty() {
                                    let cmd = format!("*{}\r\n", command.len());
                                    let cmd = command.iter().fold(cmd, |acc, arg| {
                                        format!("{}${}\r\n{}\r\n", acc, arg.len(), arg)
                                    });
                                    list_request(&cmd, &state, stream.clone()).await;
                                }
                                buffer = rest.to_vec();

                                // Lock the state to get the replica offset
                                let master_repl_offset = {
                                    let state_lock = state.lock().await;
                                    match state_lock.store.get("master_repl_offset") {
                                        Some(offset) => offset.clone(),
                                        None => "2".to_string(),
                                    }
                                };
                                let updated_offset = (master_repl_offset.parse::<i64>().unwrap() + bytes_processed as i64).to_string();

                                {
                                    state.lock().await.store.insert("master_repl_offset".to_string(), updated_offset.to_string());
                                }
                            }
                            Err(e) => {
                                println!("Incomplete command, wait for more data: {:?}", e);
                                break;
                            }
                        }
                    }
                }
            }
            Err(e) => return Err(e.into()),
        }
    }
}

async fn connect_and_handshake(
    master_address: &str
) -> Result<TcpStream, Box<dyn std::error::Error + Send + Sync>> {
    let mut master_stream = TcpStream::connect(master_address)
        .await
        .map_err(|e| format!("Failed to connect to master at {}: {}", master_address, e))?;
    
    println!("Connected to master server");
    
    establish_handshake(&mut master_stream)
        .await
        .map_err(|e| format!("Handshake failed with master at {}: {}", master_address, e))?;
    
    Ok(master_stream)
}

async fn establish_handshake(
    stream: &mut TcpStream
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Establishing Replication Handshake");
    
    // Send PING message to Master
    println!("Sending PING");
    send_and_receive(stream, "*1\r\n$4\r\nPING\r\n").await?;
    println!("PING acknowledged");

    // Send Listening Port to Master
    println!("Sending REPLCONF listening-port");
    send_and_receive(stream, "*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n$4\r\n6380\r\n").await?;
    println!("Listening port acknowledged");

    // Send Capabilities to Master
    println!("Sending REPLCONF capa");
    send_and_receive(stream, "*3\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\n$6\r\npsync2\r\n").await?;
    println!("Capabilities acknowledged");

    // Send PSYNC message to Master
    println!("Sending PSYNC");
    let psync_response = send_and_receive(stream, "*3\r\n$5\r\nPSYNC\r\n$1\r\n?\r\n$2\r\n-1\r\n").await?;
    println!("PSYNC response: {}", psync_response);

    if !psync_response.starts_with("+FULLRESYNC") {
        return Err(format!("Unexpected PSYNC response: {}", psync_response).into());
    }

    println!("Finished Replication Handshake");
    Ok(())
}

async fn send_and_receive(
    stream: &mut TcpStream, 
    message: &str
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    stream.write_all(message.as_bytes()).await?;
    let mut buffer = BytesMut::with_capacity(1024);
    loop {
        let mut byte = [0u8; 1];
        stream.read_exact(&mut byte).await?;
        buffer.extend_from_slice(&byte);
        if byte[0] == b'\n' {
            break;
        }
    }
    let response = std::str::from_utf8(&buffer)?.trim().to_string();
    println!("Sent: {}, Received: {}", message.trim(), response);
    Ok(response)
}

fn parse_resp_message(
    buffer: &[u8]
) -> Result<(Vec<String>, &[u8], usize), Box<dyn std::error::Error + Send + Sync>> {
    let mut rest = buffer;
    let mut result = Vec::new();
    let mut bytes_processed = 0;

    // Skip any whitespace between bulk strings
    while rest.starts_with(b"\r\n") {
        rest = &rest[2..];
        bytes_processed += 2;
    }

    if rest.is_empty() {
        return Err("Empty buffer after trimming".into());
    }

    if rest.starts_with(b"*") {
        // Handle array
        let newline = rest.iter().position(|&b| b == b'\r').ok_or("Incomplete data")?;
        let count: usize = std::str::from_utf8(&rest[1..newline])?.parse()?;
        rest = &rest[newline + 2..]; // Skip \r\n
        bytes_processed += newline + 2;

        for _ in 0..count {
            let (bulk_string, remaining, bulk_bytes) = parse_bulk_string(rest)?;
            result.push(bulk_string);
            rest = remaining;
            bytes_processed += bulk_bytes;
        }
    } else if rest.starts_with(b"$") {
        // Handle sequence of bulk strings (for the initial SET command)
        while rest.starts_with(b"$") {
            let (bulk_string, remaining, bulk_bytes) = parse_bulk_string(rest)?;
            result.push(bulk_string);
            rest = remaining;
            bytes_processed += bulk_bytes;

            // Skip any whitespace between bulk strings
            while rest.starts_with(b"\r\n") {
                rest = &rest[2..];
                bytes_processed += 2;
            }
        }
    } else {
        return Err(format!("Unexpected message format: {:?}", rest).into());
    }

    Ok((result, rest, bytes_processed))
}

fn parse_bulk_string(
    buffer: &[u8]
) -> Result<(String, &[u8], usize), Box<dyn std::error::Error + Send + Sync>> {
    let mut bytes_processed = 1; // Count the initial '$'
    if let Some(pos) = buffer[1..].iter().position(|&b| b == b'\r') {
        bytes_processed += pos + 2; // +2 for "\r\n"
        let length: i64 = std::str::from_utf8(&buffer[1..pos+1])?.parse()?;
        if length == -1 {
            Ok((String::new(), &buffer[pos + 3..], bytes_processed + 1))
        } else if length >= 0 {
            let start = pos + 3;
            let end = start + length as usize;
            if buffer.len() >= end + 2 {
                let content = std::str::from_utf8(&buffer[start..end])?;
                bytes_processed += length as usize + 2; // +2 for final "\r\n"
                Ok((content.to_string(), &buffer[end + 2..], bytes_processed))
            } else {
                Err("Incomplete bulk string data".into())
            }
        } else {
            Err("Invalid bulk string length".into())
        }
    } else {
        Err("Incomplete bulk string".into())
    }
}

async fn handle_client(
    stream: TcpStream,
    state: Arc<Mutex<SharedState>>
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Processing client messages");
    let stream: Arc<Mutex<TcpStream>> = Arc::new(Mutex::new(stream));
    
    let mut buf: [u8; 1024] = [0; 1024];

    loop {
        let mut stream_lock = stream.lock().await;
        match stream_lock.read(&mut buf).await {
            Ok(0) => {
                println!("Client connection closed");
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
