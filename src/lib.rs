
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task;

mod command_types {
    pub mod list;
}

mod handlers {
    pub mod echo;
    pub mod get;
    pub mod info;
    pub mod set;
}

mod helpers {
    pub mod helpers;
}

use command_types::list::list_request;

pub struct SharedState {
    pub store: HashMap<String, String>,
}

pub async fn run(port: String) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    let state = Arc::new(Mutex::new(SharedState {
        store: HashMap::new(),
    }));

    loop {
        let (socket, _) = listener.accept().await?;
        let state_clone = state.clone();
        task::spawn(async move {
            process_request(socket, state_clone).await;
        });
    }
}

async fn process_request(mut socket: tokio::net::TcpStream, state: Arc<Mutex<SharedState>>) {
    let mut buf = [0; 1024];

    loop {
        match socket.read(&mut buf).await {
            Ok(0) => {
                println!("Connection closed");
                return;
            }
            Ok(_) => {
                let request = std::str::from_utf8(&buf).unwrap();

                if request.starts_with('+') {
                    // Means it's a Simple String
                    let response = &request[1..request.len()-2];
                    println!("Received Simple String: {}", response);
                    // Process the response here
                } else if request.starts_with('-') {
                    // Means it's an Error
                    let error = &request[1..request.len()-2];
                    println!("Received Error: {}", error);
                    // Process the error here
                } else if request.starts_with(':') {
                    // Means it's an Integer
                    let integer = &request[1..request.len()-2];
                    println!("Received Integer: {}", integer);
                    // Process the integer here
                } else if request.starts_with('$') {
                    // Means it's a Bulk String
                    let bulk_string = &request[1..request.len()-2];
                    println!("Received Bulk String: {}", bulk_string);
                    // Process the bulk string here
                } else if request.starts_with('*') {
                    // Means it's a List
                    let response_string = list_request(&request, &state);
                    let response_bytes = response_string.as_bytes().try_into().unwrap();
                    if let Err(e) = socket.write_all(response_bytes).await {
                        println!("Failed to write to connection: {}", e);
                        return;
                    }
                } else {
                    println!("Unknown request format");
                }
                buf.fill(0);  // Clear the buffer
            }
            Err(e) => {
                println!("Failed to read from connection: {}", e);
                return;
            }
        }
    }
}