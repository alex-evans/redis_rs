
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task;

#[derive(Clone)]
pub struct Config {
    pub port: String,
    pub replicaof: String
}

impl Config {
    pub fn from_args(args: Vec<String>) -> Result<Config, &'static str> {
        let mut port = String::from("6379");
        let mut replicaof = String::from("");

        for i in 1..args.len() {
            if args[i] == "--port" || args[i] == "-p" {
                if i + 1 < args.len() {
                    port = args[i + 1].clone();
                } else {
                    return Err("No port provided");
                }
            } else if args[i] == "--replicaof" {
                if i + 1 < args.len() {
                    replicaof = args[i + 1].clone();
                } else {
                    return Err("No replicaof provided");
                }
            }
        }

        Ok(Config {
            port: port,
            replicaof: replicaof
        })

    }
}

mod command_types {
    pub mod list;
    pub mod replication;
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
use command_types::replication::{
    store_replication,
    ping_master
};

pub struct SharedState {
    pub store: HashMap<String, String>,
}

pub async fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.port)).await?;
    let state = Arc::new(Mutex::new(SharedState {
        store: HashMap::new(),
    }));

    loop {
        let (socket, _) = listener.accept().await?;
        let state_clone = state.clone();
        let config_clone = config.clone();
        task::spawn(async move {
            process_request(&config_clone, socket, state_clone).await;
        });
    }
}

async fn process_request(config_ref: &Config, mut socket: tokio::net::TcpStream, state: Arc<Mutex<SharedState>>) {
    println!("Handling process request");
    let mut buf = [0; 1024];

    if config_ref.replicaof != "" {
        println!("Replicaof is set to: {}", config_ref.replicaof);
        store_replication(config_ref, &state);
        ping_master(&state);
    }

    loop {
        match socket.read(&mut buf).await {
            Ok(0) => {
                println!("Connection closed");
                return;
            }
            Ok(_) => {
                let request = std::str::from_utf8(&buf).unwrap();
                println!("We got here: {}", request);

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
                    println!("Received List: {}", request);
                    let response_string = list_request(config_ref, &request, &state);
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